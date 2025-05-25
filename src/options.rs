use std::path::PathBuf;

///recording the configuraiton set by user
pub struct Options {
    pub dir_path: PathBuf,
    pub data_file_size: u64,
}
