use crate::errors::Result;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

use crate::fio;

pub struct DataFile {
    file_id: Arc<RwLock<u32>>,   //current file id
    write_off: Arc<RwLock<u64>>, //writing position offset
    io_manager: Box<dyn fio::IOManager>,
}

impl DataFile {
    pub fn new(dir_path: PathBuf, file_id: u32) -> Result<DataFile> {
        todo!()
    }
    pub fn new(dir_path: PathBuf) -> Result<DataFile> {
        todo!()
    }

    pub fn get_write_off(&self) -> u64 {
        let read_guard = self.write_off.read();
        *read_guard
    }

    pub fn get_file_id(&self) -> u32 {
        *self.file_id.read()
    }

    pub fn sync(&self) -> Result<()> {
        todo!()
    }
}
