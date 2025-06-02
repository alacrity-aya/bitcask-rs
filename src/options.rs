use std::path::PathBuf;

///recording the configuraiton set by user
#[derive(Clone)]
pub struct Options {
    pub dir_path: PathBuf,

    //size of data file
    pub data_file_size: u64,

    //whether to sync every write operation
    pub sync_writes: bool,

    pub index_type: IndexType,
}

#[derive(Clone)]
pub enum IndexType {
    Btree,
    SkipList,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            dir_path: std::env::temp_dir().join("biscask-rs"),
            data_file_size: 256 * 1024 * 1024,
            sync_writes: false,
            index_type: IndexType::Btree,
        }
    }
}
