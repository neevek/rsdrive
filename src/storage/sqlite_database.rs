use super::database::Database;
use crate::server::entity::SyncFileInfo;
use crate::server::entity::User;
use anyhow::bail;
use anyhow::Result;
use rs_utilities::log_and_bail;
use rusqlite::Connection;
use std::path::Path;
use tracing::debug;
use tracing::error;

pub struct SqliteDatabase {
    conn: Connection,
}

impl SqliteDatabase {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self { conn: Self::init(path)? })
    }

    fn init<P: AsRef<Path>>(path: P) -> Result<Connection> {
        let conn = Connection::open(path.as_ref())?;
        let sql = "
            CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                password TEXT NOT NULL,
                phone_number TEXT NOT NULL,
                email TEXT NOT NULL,
                create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime'))
            );

            CREATE TABLE IF NOT EXISTS shared_file (
                file_hash TEXT PRIMARY KEY,
                ref_count INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                sync_size INTEGER NOT NULL,
                sync_completed INTEGER NOT NULL DEFAULT 0 CHECK (sync_completed IN (0, 1)),
                create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime'))
            );
            
            CREATE TABLE IF NOT EXISTS user_file (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                file_hash TEXT NOT NULL,
                file_dir TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_meta TEXT NOT NULL DEFAULT '',
                file_create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                record_create_time DATETIME NOT NULL DEFAULT (datetime(CURRENT_TIMESTAMP, 'localtime')),
                UNIQUE (user_id, file_name, file_dir)
            );

            CREATE INDEX IF NOT EXISTS idx_user_file ON user_file (user_id, file_dir, file_name);

            ";

        match conn.execute_batch(sql) {
            Ok(_) => {}
            Err(e) => {
                let path = path.as_ref().to_str().unwrap();
                log_and_bail!("failed create sqlite database, path:{path}, error:{e:?}, sql:{sql}");
            }
        }

        Ok(conn)
    }
}

impl Database for SqliteDatabase {
    fn save_user(&self, user: &User) -> Result<()> {
        let sql = "
            INSERT OR IGNORE INTO user (username, password, phone_number, email)
            VALUES (?, ?, ?, ?)";
        self.conn
            .execute(sql, rusqlite::params![user.username, user.password, user.phone_number, user.email])?;
        Ok(())
    }

    fn query_user(&self, username: &str, password: &str) -> Option<User> {
        let sql = "
            SELECT id, username, password, phone_number, email, create_time
            FROM user WHERE username = ? AND password = ?";

        self.conn
            .query_row(sql, rusqlite::params![username, password], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password: row.get(2)?,
                    phone_number: row.get(3)?,
                    email: row.get(4)?,
                    create_time: row.get(5)?,
                })
            })
            .ok()
    }

    fn query_file_info(&self, user_id: u32, file_dir: &str, file_name: &str) -> Option<SyncFileInfo> {
        let sql = "
            SELECT u.file_dir, u.file_name, u.file_meta, s.file_hash, s.sync_size, s.file_size
            FROM user_file AS u
            JOIN shared_file AS s ON u.file_hash = s.file_hash
            WHERE u.user_id = ? AND u.file_dir = ? AND u.file_name = ?";

        self.conn
            .query_row(sql, rusqlite::params![user_id, file_dir, file_name], |row| {
                Ok(SyncFileInfo {
                    file_dir: row.get(0)?,
                    file_name: row.get(1)?,
                    file_meta: row.get(2)?,
                    file_hash: row.get(3)?,
                    sync_size: row.get(4)?,
                    file_size: row.get(5)?,
                })
            })
            .ok()
    }

    fn save_file_info(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<()> {
        let i = &file_info;

        let sql = "
            INSERT INTO user_file (user_id, file_hash, file_dir, file_name, file_meta)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(user_id, file_dir, file_name)
            DO NOTHING";
        let rows_affected = self
            .conn
            .execute(sql, rusqlite::params![user_id, i.file_hash, i.file_dir, i.file_name, i.file_meta])?;

        if rows_affected > 0 {
            let sql = "
                INSERT OR REPLACE INTO shared_file (file_hash, sync_size, file_size, ref_count)
                VALUES (?, ?, ?, COALESCE((SELECT ref_count + 1 FROM shared_file WHERE file_hash = ?), 1))";

            debug!("will insert new record:{}", i.file_hash);
            self.conn
                .execute(sql, rusqlite::params![i.file_hash, i.sync_size, i.file_size, i.file_hash])?;
        }

        Ok(())
    }

    fn delete_file_info(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<bool> {
        let sql = "
            DELETE FROM user_file WHERE user_id = ? AND file_dir = ? AND file_name = ?";

        let sql = "
            DELETE FROM shared_file WHERE file_hash = ? AND ref_count = 1";
        let deleted = self
            .conn
            .execute(sql, rusqlite::params![user_id, file_info.file_dir, file_info.file_name])?
            > 0;
        debug!("deleting record:{file_info:?}, deleted:{deleted}");
        Ok(deleted)
    }

    fn update_sync_size(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<()> {
        let sql = "UPDATE shared_file SET sync_size = ?, sync_completed = ? WHERE file_hash = ?";
        let sync_completed = file_info.sync_size >= file_info.file_size;
        self.conn
            .execute(sql, rusqlite::params![file_info.sync_size, sync_completed, file_info.file_hash])?;
        Ok(())
    }
}
