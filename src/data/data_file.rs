use crate::errors::Errors;
use crate::errors::Result;
use bytes::{Buf, BytesMut};
use parking_lot::RwLock;
use prost::decode_length_delimiter;
use prost::length_delimiter_len;
use std::path::PathBuf;
use std::sync::Arc;

use super::log_record::LogRecord;
use super::log_record::LogRecordType;
use super::log_record::CRC_LENGTH;
use super::log_record::TYPE_LENGTH;

use crate::fio::{self, new_io_manager};

use super::log_record::{max_log_record_header_size, ReadLogRecord};

pub const DATA_FILE_NAME_SUFFIX: &str = ".data";

pub struct DataFile {
    file_id: Arc<RwLock<u32>>,   //current file id
    write_off: Arc<RwLock<u64>>, //writing position offset
    io_manager: Box<dyn fio::IOManager>,
}

impl DataFile {
    pub fn new(dir_path: PathBuf, file_id: u32) -> Result<DataFile> {
        let file_name = get_data_file_name(dir_path, file_id);
        let io_manager = new_io_manager(file_name)?;

        Ok(Self {
            file_id: Arc::new(RwLock::new(file_id)),
            write_off: Arc::new(RwLock::new(0)),
            io_manager: Box::new(io_manager),
        })
    }

    pub fn get_write_off(&self) -> u64 {
        let read_guard = self.write_off.read();
        *read_guard
    }

    pub fn set_write_off(&mut self, offset: u64) {
        let mut write_guard = self.write_off.write();
        *write_guard = offset;
    }

    pub fn get_file_id(&self) -> u32 {
        *self.file_id.read()
    }

    pub fn sync(&self) -> Result<()> {
        self.io_manager.sync()
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize> {
        let n_bytes = self.io_manager.write(buf)?;
        let mut write_off = self.write_off.write();
        *write_off += n_bytes as u64;

        Ok(n_bytes)
    }

    /*
     *+------+---------+---------+-----+-----+-----+
     *| type | keysize | valsize | key | val | CRC |
     *+------+---------+---------+-----+-----+-----+
     *
     *
     * */

    pub fn read_log_record(&self, offset: u64) -> Result<ReadLogRecord> {
        let mut header_buf = BytesMut::zeroed(max_log_record_header_size());
        self.io_manager.read(&mut header_buf, offset)?;

        let rec_type = header_buf.get_u8();

        let key_size = decode_length_delimiter(&mut header_buf).unwrap();
        let value_size = decode_length_delimiter(&mut header_buf).unwrap();
        if key_size == 0 && value_size == 0 {
            return Err(Errors::ReadDataFileEOF);
        }

        //get the real size of header
        let actual_header_size =
            length_delimiter_len(key_size) + length_delimiter_len(value_size) + TYPE_LENGTH;

        let mut kv_buf = BytesMut::zeroed(key_size + value_size + CRC_LENGTH);
        self.io_manager
            .read(&mut kv_buf, offset + actual_header_size as u64);

        //return LogRecord
        let mut log_record = LogRecord {
            key: kv_buf.get(..key_size).unwrap().to_vec(),
            value: kv_buf
                .get(key_size..kv_buf.len() - CRC_LENGTH)
                .unwrap()
                .to_vec(),
            rec_type: LogRecordType::from_u8(rec_type),
        };

        kv_buf.advance(key_size + value_size);
        if kv_buf.get_u32() != log_record.get_crc() {
            return Err(Errors::InvalidLogRecordCRC);
        }

        Ok(ReadLogRecord {
            record: log_record,
            size: actual_header_size + key_size + value_size + CRC_LENGTH,
        })
    }
}

fn get_data_file_name(dir_path: PathBuf, file_id: u32) -> PathBuf {
    let v = std::format!("{:09}", file_id) + DATA_FILE_NAME_SUFFIX;
    dir_path.join(v)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_data_file() {
        let dir_path = std::env::temp_dir();

        let data_file_res = DataFile::new(dir_path.clone(), 0);
        assert!(data_file_res.is_ok());

        let data_file = data_file_res.unwrap();
        assert_eq!(data_file.get_file_id(), 0);

        let data_file_res = DataFile::new(dir_path.clone(), 0);
        assert!(data_file_res.is_ok());

        let data_file = data_file_res.unwrap();
        assert_eq!(data_file.get_file_id(), 0);

        println!("temp dir : {:?}", dir_path.clone().as_os_str());
    }
}
