use std::path::PathBuf;

///recording the configuraiton set by user
pub struct Options {
    pub dir_path: PathBuf,

    //size of data file
    pub data_file_size: u64,

    //whether to sync every write operation
    pub sync_writes: bool,
}
