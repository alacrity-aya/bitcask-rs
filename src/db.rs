use crate::data::{
    data_file::{DataFile, DATA_FILE_NAME_SUFFIX},
    log_record::{LogRecord, LogRecordPos, LogRecordType},
};
use crate::options::Options;
use crate::{
    errors::{Errors, Result},
    index,
};
use bytes::Bytes;
use log::warn;
use parking_lot::RwLock;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

///bitcask engine
pub struct Engine {
    options: Arc<Options>,
    active_file: Arc<RwLock<DataFile>>,
    older_files: Arc<RwLock<HashMap<u32, DataFile>>>,
    index: Box<dyn index::Indexer>, //Btree Skiplist LSM-Tree...
    file_ids: Vec<u32>,
}

const INITAL_FILE_ID: u32 = 0;

//面向用户的接口
impl Engine {
    pub fn open(opts: Options) -> Result<Self> {
        if let Some(e) = check_options(&opts) {
            return Err(e);
        }
        let dir_path = &opts.dir_path;
        if dir_path.is_dir() {
            if let Err(e) = fs::create_dir_all(dir_path) {
                warn!("create database dir err:{e}\n");
                return Err(Errors::CreateDatabaseDirErr);
            }
        }
        let mut data_files = load_data_file(dir_path)?;

        let mut file_ids = Vec::new();
        for v in data_files.iter() {
            file_ids.push(v.get_file_id());
        }
        //put old files into older_files
        let mut older_files = HashMap::new();
        if data_files.len() > 1 {
            for _ in 0..data_files.len() - 2 {}
            let file = data_files.pop().unwrap();
            older_files.insert(file.get_file_id(), file);
        }

        //get the current active file that is the last ele in data_files
        let active_file = match data_files.pop() {
            Some(v) => v,
            None => DataFile::new(dir_path.to_path_buf(), INITAL_FILE_ID)?,
        };

        let engine = Self {
            options: Arc::new(opts.clone()),
            active_file: Arc::new(RwLock::new(active_file)),
            older_files: Arc::new(RwLock::new(older_files)),
            index: Box::new(index::new_indexer(opts.index_type)),
            file_ids,
        };

        engine.load_index_from_data_files()?;

        Ok(engine)
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

    pub fn delete(&self, key: Bytes) -> Result<()> {
        if key.is_empty() {
            return Err(Errors::EmptyKey);
        }

        //look up whether the look is existed. if it doesn't exist, return
        let pos = self.index.get(key.to_vec());
        if pos.is_none() {
            return Ok(());
        }

        //after getting the vaild key, we should put it into active_file
        let mut record = LogRecord {
            key: key.to_vec(),
            value: Default::default(),
            rec_type: LogRecordType::Deleted,
        };
        self.append_log_record(&mut record)?;

        //remove key from memory index
        match self.index.delete(key.to_vec()) {
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
            true => active_file.read_log_record(pos.offset)?.record,
            false => {
                let data_file = older_file.get(&pos.file_id);
                if data_file.is_none() {
                    return Err(Errors::DataFileNotFound);
                }
                data_file.unwrap().read_log_record(pos.offset)?.record
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

    fn load_index_from_data_files(&self) -> Result<()> {
        if self.file_ids.is_empty() {
            return Ok(());
        }

        let active_file = self.active_file.read();
        let older_file = self.older_files.read();

        for (i, file_id) in self.file_ids.iter().enumerate() {
            let mut offset = 0;
            loop {
                let log_record_res = match *file_id == active_file.get_file_id() {
                    true => active_file.read_log_record(offset),
                    false => {
                        let data_file = older_file.get(&(i as u32)).unwrap();
                        data_file.read_log_record(offset)
                    }
                };
                let (log_record, size) = match log_record_res {
                    Ok(result) => (result.record, result.size),
                    Err(e) => {
                        if e == Errors::ReadDataFileEOF {
                            break;
                        }
                        return Err(e);
                    }
                };

                //setup memory index
                let log_record_pos = LogRecordPos {
                    file_id: *file_id,
                    offset,
                };
                let ok = match log_record.rec_type {
                    LogRecordType::Normal => {
                        self.index.put(log_record.key.to_vec(), log_record_pos)
                    }

                    LogRecordType::Deleted => self.index.delete(log_record.key.to_vec()),
                };

                if !ok {
                    return Err(Errors::UpdateIndexErr);
                }

                //updata offset
                offset += size;
            }
        }
        Ok(())
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
        } else {
            unreachable!("dir.unwrap() return Err\n"); //TODO: should return a Errors:: here
        }
    }
    if file_idx.is_empty() {
        return Ok(data_files);
    }

    //对文件id进行排序，从小到大进行加载
    file_idx.sort();

    for file_id in file_idx.iter() {
        let data_file = DataFile::new(dir_path.to_path_buf(), *file_id)?;
        data_files.push(data_file);
    }

    Ok(data_files)
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
