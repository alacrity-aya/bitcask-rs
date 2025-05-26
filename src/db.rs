use crate::data::data_file::DataFile;
use crate::data::data_file::DATA_FILE_NAME_SUFFIX;
use crate::data::log_record::LogRecord;
use crate::data::log_record::LogRecordPos;
use crate::data::log_record::LogRecordType;
use crate::errors::Errors;
use crate::errors::Result;
use crate::index;
use crate::options::Options;
use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

///bitcask engine
pub struct Engine {
    options: Arc<Options>,
    active_file: Arc<RwLock<DataFile>>,
    older_files: Arc<RwLock<HashMap<u32, DataFile>>>,
    index: Box<dyn index::Indexer>, //Btree Skiplist LSM-Tree...
}

//面向用户的接口
impl Engine {
    pub fn open(opts: Options) -> Result<Self> {
        if let Some(e) = check_options(&opts) {
            return Err(e);
        }
        todo!()
    }

    pub fn put(&self, key: Bytes, value: Bytes) -> Result<()> {
        if key.is_empty() {
            return Err(Errors::EmptyKey);
        }

        let mut record = LogRecord {
            key: key.to_vec(),
            value: value.to_vec(),
            rec_type: LogRecordType::Normal,
        };

        let log_record_pos = self.append_log_record(&mut record)?;

        match self.index.put(key.to_vec(), log_record_pos) {
            true => Ok(()),
            false => Err(Errors::UpdateIndexErr),
        }
    }

    //NOTE::why not use Option<Bytes> here?
    pub fn get(&self, key: Bytes) -> Result<Bytes> {
        if key.is_empty() {
            return Err(Errors::EmptyKey);
        }
        let pos = self.index.get(key.to_vec());

        if pos.is_none() {
            return Err(Errors::KeyNotFound);
        }

        //find log_record in corresponding datafiles(active_file,older_files)
        let pos = pos.unwrap();
        let active_file = self.active_file.read();
        let older_file = self.older_files.read();
        let log_record = match active_file.get_file_id() == pos.file_id {
            true => active_file.read_log_record(pos.offset)?,
            false => {
                let data_file = older_file.get(&pos.file_id);
                if data_file.is_none() {
                    return Err(Errors::DataFileNotFound);
                }
                data_file.unwrap().read_log_record(pos.offset)?
            }
        };

        if log_record.rec_type == LogRecordType::Deleted {
            return Err(Errors::ValueNotFound);
        }

        Ok(log_record.value.into())
    }

    fn append_log_record(&self, log_record: &mut LogRecord) -> Result<LogRecordPos> {
        let path = &self.options.dir_path;
        let enc_record = log_record.encode();
        let record_len = enc_record.len() as u64;

        let mut active_file = self.active_file.write();

        if active_file.get_write_off() + record_len > self.options.data_file_size {
            //sync current using file
            //persist current active_file to older_files

            active_file.sync()?;
            let current_fid = active_file.get_file_id();

            let mut older_files = self.older_files.write();
            let old_file = DataFile::new(path.clone(), current_fid)?;
            older_files.insert(current_fid, old_file);

            let new_file = DataFile::new(path.clone(), current_fid + 1)?;
            *active_file = new_file;
        }

        let write_off = active_file.get_write_off();
        active_file.write(&enc_record)?;

        if self.options.sync_writes {
            active_file.sync()?;
        }

        Ok(LogRecordPos {
            file_id: active_file.get_file_id(),
            offset: active_file.get_write_off(),
        })
    }
}

///load data file from dir_path
fn load_data_file(dir_path: &PathBuf) -> Result<Vec<DataFile>> {
    let dir = fs::read_dir(dir_path);
    if dir.is_err() {
        return Err(Errors::ReadDatabaseDirErr);
    }

    let mut file_idx: Vec<u32> = Vec::new();
    let mut data_files: Vec<DataFile> = Vec::new();

    //get file id
    for file in dir.unwrap() {
        if let Ok(entry) = file {
            let os_file_name = entry.file_name();
            let file_name = os_file_name.to_str().unwrap();

            if !file_name.ends_with(DATA_FILE_NAME_SUFFIX) {
                continue;
            }
            let split_name: Vec<&str> = file_name.split(".").collect();

            let file_id = match split_name[0].parse::<u32>() {
                Ok(fid) => fid,
                Err(_) => return Err(Errors::DataDirCorrupted),
            };
            file_idx.push(file_id);
        }
    }

    todo!()
}

fn check_options(opts: &Options) -> Option<Errors> {
    let dir_path = opts.dir_path.to_str();
    if dir_path.is_none() || dir_path.unwrap().is_empty() {
        return Some(Errors::DirPathIsEmpty);
    }
    if opts.data_file_size == 0 {
        return Some(Errors::DataFileSizeTooSmall);
    }

    None
}
