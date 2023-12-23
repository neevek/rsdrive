use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use super::file_storage::{FileReader, FileStorage, FileWriter};
use crate::server::entity::SyncFileInfo;
use anyhow::{Context, Result};

pub struct LocalFileStorage {
    base_dir: PathBuf,
}

pub struct LocalFileWriter {
    file: File,
}
pub struct LocalFileReader {
    file: File,
}

impl FileWriter for LocalFileWriter {
    fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.file.write_all(data).map(|_| Ok(data.len()))?
    }

    fn close(&mut self) {
        // do nothing
    }
}

impl FileReader for LocalFileReader {
    fn read(&mut self, data: &mut [u8]) -> Result<usize> {
        self.file.read(data).map(|_| Ok(data.len()))?
    }

    fn close(&mut self) {
        // do nothing
    }
}

impl LocalFileStorage {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

impl LocalFileStorage {
    fn open_file(&self, file_hash: &str) -> Result<File> {
        let mut path = PathBuf::new();
        path.push(&self.base_dir);
        path.push(&file_hash[..2]);

        if !path.exists() {
            fs::create_dir_all(&path).context(format!("failed to create dir:{path:?}"))?;
        }

        path.push(&file_hash[2..]);
        Ok(File::create(&path).context(format!("failed to create file:{path:?}"))?)
    }
}

impl FileStorage for LocalFileStorage {
    fn open_writer(&self, file_info: &SyncFileInfo) -> Result<Box<dyn FileWriter>> {
        Ok(Box::new(LocalFileWriter {
            file: self.open_file(&file_info.file_hash)?,
        }))
    }

    fn open_reader(&self, file_info: &SyncFileInfo) -> Result<Box<dyn FileReader>> {
        Ok(Box::new(LocalFileReader {
            file: self.open_file(&file_info.file_hash)?,
        }))
    }

    fn delete_file(&self, file_hash: &str) -> Result<()> {
        let mut path = PathBuf::new();
        path.push(&self.base_dir);
        path.push(&file_hash[..2]);
        path.push(&file_hash[2..]);
        Ok(fs::remove_file(path)?)
    }
}
