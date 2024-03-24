use std::path::PathBuf;

use super::{database::Database, sqlite_database::SqliteDatabase};
use anyhow::Result;

pub struct DatabaseConfig {
    uri: String,
}

pub struct DatabaseManager {
    config: DatabaseConfig,
}

impl DatabaseManager {
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }

    pub fn get_database(&self) -> Result<Box<dyn Database>> {
        let path = PathBuf::from(&self.config.uri);
        Ok(Box::new(SqliteDatabase::open(path)?))
    }
}
