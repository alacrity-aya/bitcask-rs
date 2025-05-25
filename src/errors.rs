use std::result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("fail to read from data file")]
    ReadFromDataFileErr,

    #[error("fail to write to data file")]
    WriteToDataFileErr,

    #[error("fail to sync data file")]
    SyncDataFileErr,

    #[error("fail to open data file")]
    OpenDataFileErr,
}

pub type Result<T> = result::Result<T, Errors>;
