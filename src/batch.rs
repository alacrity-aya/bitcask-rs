use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use parking_lot::Mutex;

use crate::data::log_record::LogRecordType;
use crate::errors::{Errors, Result};
use crate::{data::log_record::LogRecord, db::Engine, options::WriteBatchOptions};

pub struct WriteBatch<'a> {
    pending_writes: Arc<Mutex<HashMap<Vec<u8>, LogRecord>>>,
    engine: &'a Engine,
    options: WriteBatchOptions,
}

impl Engine {
    pub fn new_write_batch(&self, options: WriteBatchOptions) -> Result<WriteBatch> {
        Ok(WriteBatch {
            pending_writes: Arc::new(Mutex::new(HashMap::new())),
            engine: self,
            options,
        })
    }
}

impl WriteBatch<'_> {
    pub fn put(&self, key: Bytes, value: Bytes) -> Result<()> {
        if key.is_empty() {
            return Err(Errors::KeyIsEmpty);
        }

        let record = LogRecord {
            key: key.to_vec(),
            value: value.to_vec(),
            rec_type: LogRecordType::NORMAL,
        };
        let mut pending_writes = self.pending_writes.lock();
        pending_writes.insert(key.to_vec(), record);

        Ok(())
    }

    pub fn delete(&self, key: Bytes, value: Bytes) -> Result<()> {
        if key.is_empty() {
            return Err(Errors::KeyIsEmpty);
        }

        let mut pending_writes = self.pending_writes.lock();
        let index_pos = self.engine.index.get(key.to_vec());
        if index_pos.is_none() {
            if pending_writes.contains_key(&key.to_vec()) {
                pending_writes.remove(&key.to_vec());
            }
            return Ok(());
        }

        let record = LogRecord {
            key: key.to_vec(),
            value: Default::default(),
            rec_type: LogRecordType::DELETED,
        };

        pending_writes.insert(key.to_vec(), record);
        Ok(())
    }

    pub fn commit(&self) -> Result<()> {
        let pending_write = self.pending_writes.lock();
        if pending_write.is_empty() {
            return Ok(());
        }

        if pending_write.len() > self.options.max_batch_num {
            return Err(Errors::ExceedMaxBatchNum);
        }

        //保证全局的串行化
        let _lock = self.engine.batch_commit_lock.lock();

        Ok(())
    }
}
