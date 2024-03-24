use crate::server::entity::{SyncFileInfo, User};
use anyhow::Result;

pub trait Database: Send {
    fn save_user(&self, user: &User) -> Result<()>;
    fn query_user(&self, username: &str, password: &str) -> Option<User>;
    fn query_file_info(&self, user_id: u32, file_dir: &str, file_name: &str) -> Option<SyncFileInfo>;
    fn save_file_info(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<()>;
    fn delete_file_info(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<bool>;
    fn update_sync_size(&self, user_id: u32, file_info: &SyncFileInfo) -> Result<()>;
}
