use crate::data::data_file::DataFile;
use crate::data::log_record::LogRecord;
use crate::data::log_record::LogRecordType;
use crate::errors::Errors;
use crate::errors::Result;
use crate::options::Options;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

///bitcask存储引擎
pub struct Engine {
    options: Arc<Options>,
    active_file: Arc<RwLock<DataFile>>,
    older_files: Arc<RwLock<HashMap<u32, DataFile>>>,
}

//面向用户的接口
impl Engine {
    pub fn put(&self, key: Bytes, value: Bytes) -> Result<()> {
        if key.is_empty() {
            return Err(Errors::EmptyKey);
        }

        let record = LogRecord {
            key: key.to_vec(),
            value: value.to_vec(),
            rec_type: LogRecordType::Normal,
        };

        todo!();
        Ok(())
    }

    fn append_log_record(&self, log_record: &mut LogRecord) -> Result<()> {
        let path = &self.options.dir_path;
        let enc_record = log_record.encode();
        let record_len = enc_record.len() as u64;

        let mut active_file = self.active_file.write();

        if active_file.get_write_off() + record_len > self.options.data_file_size {
            //sync current using file
            //persist current active_file to older_files

            active_file.sync()?;
            let current_fid = active_file.get_file_id();
            let older_file = self.older_files.write();
            todo!();
            older_file.insert(current_fid, active_file);
        }
        todo!()
    }
}
