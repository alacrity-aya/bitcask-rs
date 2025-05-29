pub mod btree;

use crate::{data::log_record::LogRecordPos, options::IndexType};

pub trait Indexer: Send + Sync {
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> bool;
    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos>;
    fn delete(&self, key: Vec<u8>) -> bool;
}

pub fn new_indexer(index_type: IndexType) -> impl Indexer {
    match index_type {
        IndexType::Btree => btree::Btree::new(),
        IndexType::SkipList => todo!(),
        _ => unreachable!("unknow index type"),
    }
}
