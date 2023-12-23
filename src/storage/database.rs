use crate::server::entity::SyncFileInfo;
use anyhow::Result;

pub trait Database: Send {
    fn query_file_info(&self, file_hash: &str) -> Option<SyncFileInfo>;
    fn save_file_info(&self, file_info: &SyncFileInfo) -> Result<()>;
    fn delete_file_info(&self, file_hash: &str) -> Result<bool>;
    fn update_sync_size(&self, file_info: &SyncFileInfo) -> Result<()>;
}
