use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
