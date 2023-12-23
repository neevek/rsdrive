use crate::server::entity::SyncFileInfo;
use anyhow::Result;

pub trait FileWriter: Send {
    fn write(&mut self, data: &[u8]) -> Result<usize>;
    fn close(&mut self);
}

pub trait FileReader: Send {
    fn read(&mut self, data: &mut [u8]) -> Result<usize>;
    fn close(&mut self);
}

pub trait FileStorage: Send {
    fn open_writer(&self, file_info: &SyncFileInfo) -> Result<Box<dyn FileWriter>>;
    fn open_reader(&self, file_info: &SyncFileInfo) -> Result<Box<dyn FileReader>>;
    fn delete_file(&self, file_hash: &str) -> Result<()>;
}
