use super::super::errors::Result;
use crate::fio::IOManager;

use super::super::errors::Errors;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::sync::Arc;

use log::error;
use parking_lot::RwLock;

pub struct FileIO {
    fd: Arc<RwLock<File>>,
}

impl FileIO {
    pub fn new(file_name: PathBuf) -> Result<Self> {
        match OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(file_name)
        {
            Ok(file) => Ok(Self {
                fd: Arc::new(RwLock::new(file)),
            }),
            Err(e) => {
                error!("failed to open data file: {e}");
                Err(Errors::OpenDataFileErr)
            }
        }
    }
}

impl IOManager for FileIO {
    fn read(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let read_guard = self.fd.read();
        match read_guard.read_at(buf, offset) {
            Ok(n) => Ok(n),
            Err(e) => {
                error!("read from the data file err: {e}");
                Err(Errors::ReadFromDataFileErr)
            }
        }
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        let mut write_guard = self.fd.write();

        match write_guard.write(buf) {
            Ok(n) => Ok(n),
            Err(e) => {
                error!("write to data file err:{e}");
                Err(Errors::WriteToDataFileErr)
            }
        }
    }

    fn sync(&self) -> Result<()> {
        let read_guard = self.fd.read();
        if let Err(e) = read_guard.sync_all() {
            error!("sync data file err:{e}");
            return Err(Errors::SyncDataFileErr);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;
    #[test]
    fn test_file_io_write() {
        let path = PathBuf::from("/tmp/a.data");
        let fio_res = FileIO::new(path.clone());
        assert!(fio_res.is_ok());

        let fio = fio_res.ok().unwrap();
        let res1 = fio.write("key-a".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let res1 = fio.write("key-b".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let res2 = fs::remove_file(path);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_file_io_read() {
        let path = PathBuf::from("/tmp/b.data");
        let fio_res = FileIO::new(path.clone());
        assert!(fio_res.is_ok());

        let fio = fio_res.ok().unwrap();
        let res1 = fio.write("key-a".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let res1 = fio.write("key-b".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let mut buf1 = [0u8; 5];
        let read_res1 = fio.read(&mut buf1, 0);
        assert!(read_res1.is_ok());
        assert_eq!(read_res1.unwrap(), 5);

        let mut buf2 = [0u8; 5];
        let read_res2 = fio.read(&mut buf2, 5);
        assert!(read_res2.is_ok());
        assert_eq!(read_res2.unwrap(), 5);

        let res2 = fs::remove_file(path);
        assert!(res2.is_ok());
    }

    #[test]
    fn test_file_io_sync() {
        let path = PathBuf::from("/tmp/c.data");
        let fio_res = FileIO::new(path.clone());
        assert!(fio_res.is_ok());

        let fio = fio_res.ok().unwrap();
        let res1 = fio.write("key-a".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let res1 = fio.write("key-b".as_bytes());
        assert!(res1.is_ok());
        assert_eq!(res1.ok().unwrap(), 5);

        let sync_res1 = fio.sync();
        assert!(sync_res1.is_ok());

        let res2 = fs::remove_file(path);
        assert!(res2.is_ok());
    }
}
