pub mod file_io;
use super::errors::Result;

pub trait IOManager: Sync + Send {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    fn write(&self, buf: &[u8]) -> Result<usize>;

    /// 持久化数据
    fn sync(&self) -> Result<()>;
}
