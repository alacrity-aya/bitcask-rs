pub mod file_io;
use file_io::FileIO;
use std::path::PathBuf;

use super::errors::Result;

pub trait IOManager: Sync + Send {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    fn write(&self, buf: &[u8]) -> Result<usize>;

    /// 持久化数据
    fn sync(&self) -> Result<()>;
}

pub fn new_io_manager(file_name: PathBuf) -> Result<impl IOManager> {
    FileIO::new(file_name)
}
