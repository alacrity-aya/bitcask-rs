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

    #[error("key is empty")]
    EmptyKey,

    #[error("fail to update memory index")]
    UpdateIndexErr,

    #[error("key is not found in database")]
    KeyNotFound,

    #[error("data file is not found in database")]
    DataFileNotFound,

    #[error("database dir path can not be empty")]
    DirPathIsEmpty,

    #[error("corresponding value is not found")]
    ValueNotFound,

    #[error("data file should larger than zero")]
    DataFileSizeTooSmall,

    #[error("fail to create database dir")]
    CreateDatabaseDirErr,

    #[error("fail to read database dir")]
    ReadDatabaseDirErr,

    #[error("data dir may be corrupted")]
    DataDirCorrupted,
}

pub type Result<T> = result::Result<T, Errors>;
