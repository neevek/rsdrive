use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub phone_number: String,
    pub email: String,
    pub create_time: u32,
}

#[derive(Serialize, Deserialize)]
pub struct UploadEntity {
    pub file_pos: usize,
    pub file_size: usize,
    pub file_name: String,
    pub dest_dir: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SyncFileInfo {
    pub file_hash: String,
    pub sync_size: usize,
    pub file_size: usize,
    pub file_name: String,
    pub file_dir: String,
    pub file_meta: String,
}
