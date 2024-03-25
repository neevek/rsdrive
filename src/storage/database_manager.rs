use super::{database::Database, sqlite_database::SqliteDatabase};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub uri: String,
}

#[derive(Debug, Clone)]
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
