use crate::data::log_record::LogRecord;
use crate::data::log_record::LogRecordType;
use crate::errors::Errors;
use crate::errors::Result;
use bytes::Bytes;

///bitcask存储引擎
pub struct Engine {}

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

        Ok(())
    }
}
