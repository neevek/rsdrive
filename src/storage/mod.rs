pub mod database;
pub mod file_storage;
pub mod local_file_storage;
pub mod sqlite_database;

use self::{database::Database, file_storage::FileStorage};

pub struct StorageContext {
    pub db: Box<dyn Database>,
    pub file_storage: Box<dyn FileStorage>,
}
