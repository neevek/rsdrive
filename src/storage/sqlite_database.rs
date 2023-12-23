use super::database::Database;
use crate::server::entity::SyncFileInfo;
use anyhow::bail;
use anyhow::Result;
use rs_utilities::log_and_bail;
use rusqlite::Connection;
use std::path::Path;
use tracing::debug;
use tracing::error;

pub struct SqliteDatabase {
    user_id: u32,
    conn: Connection,
}

impl SqliteDatabase {
    pub fn open<P: AsRef<Path>>(user_id: u32, path: P) -> Result<Self> {
        Ok(Self {
            user_id,
            conn: Self::init(path)?,
        })
    }

    fn init<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let conn = Connection::open(path.as_ref())?;
        let create_tables_sql = vec![
            "
            CREATE TABLE IF NOT EXISTS shared_files (
                file_hash TEXT PRIMARY KEY,
                ref_count INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                sync_size INTEGER NOT NULL,
                sync_completed INTEGER NOT NULL DEFAULT 0 CHECK (sync_completed IN (0, 1)),
                create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime'))
            )",
            "
            CREATE TABLE IF NOT EXISTS user_files (
                id INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_dir TEXT NOT NULL,
                file_meta TEXT NOT NULL DEFAULT '',
                file_create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                record_create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                UNIQUE (file_name, file_dir)
            )",
        ];

        for sql in create_tables_sql {
            match conn.execute(sql, ()) {
                Ok(_) => {}
                Err(e) => {
                    let path = path.as_ref().to_str().unwrap();
                    log_and_bail!("failed create sqlite database, path:{path}, error:{e:?}, sql:{sql}");
                }
            }
        }

        Ok(conn)
    }
}

impl Database for SqliteDatabase {
    fn query_file_info(&self, file_hash: &str) -> Option<SyncFileInfo> {
        let sql = "
            SELECT u.file_name, u.file_dir, s.sync_size, s.file_size, u.file_meta, s.ref_count
            FROM user_files AS u
            JOIN shared_files AS s ON u.file_hash = s.file_hash
            WHERE u.user_id = ? AND u.file_hash = ?";

        self.conn
            .query_row(sql, rusqlite::params![self.user_id, file_hash], |row| {
                Ok(SyncFileInfo {
                    file_hash: file_hash.to_string(),
                    file_name: row.get(0)?,
                    file_dir: row.get(1)?,
                    sync_size: row.get(2)?,
                    file_size: row.get(3)?,
                    file_meta: row.get(4)?,
                })
            })
            .ok()
    }

    fn save_file_info(&self, file_info: &SyncFileInfo) -> Result<()> {
        let i = &file_info;

        let sql = "
            INSERT INTO user_files (user_id, file_hash, file_name, file_dir, file_meta)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(file_name, file_dir)
            DO NOTHING";
        let rows_affected = self.conn.execute(
            sql,
            rusqlite::params![self.user_id, i.file_hash, i.file_name, i.file_dir, i.file_meta],
        )?;

        if rows_affected > 0 {
            let sql = "
                INSERT OR REPLACE INTO shared_files (file_hash, sync_size, file_size, ref_count)
                VALUES (?, ?, ?, COALESCE((SELECT ref_count + 1 FROM shared_files WHERE file_hash = ?), 1))";

            debug!("will insert new record:{}", i.file_hash);
            self.conn
                .execute(sql, rusqlite::params![i.file_hash, i.sync_size, i.file_size, i.file_hash])?;
        }

        Ok(())
    }

    fn delete_file_info(&self, file_hash: &str) -> Result<bool> {
        let sql = "
            DELETE FROM shared_files WHERE file_hash = ? AND ref_count = 1";
        let deleted = self.conn.execute(sql, rusqlite::params![file_hash])? > 0;
        debug!("deleting record:{file_hash}, deleted:{deleted}");
        Ok(deleted)
    }

    fn update_sync_size(&self, file_info: &SyncFileInfo) -> Result<()> {
        let sql = "UPDATE shared_files SET sync_size = ?, sync_completed = ? WHERE file_hash = ?";
        let sync_completed = file_info.sync_size >= file_info.file_size;
        self.conn
            .execute(sql, rusqlite::params![file_info.sync_size, sync_completed, file_info.file_hash])?;
        Ok(())
    }
}
