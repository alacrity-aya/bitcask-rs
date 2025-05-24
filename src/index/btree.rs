use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::data::log_record::LogRecordPos;

pub struct Btree {
    tree: Arc<RwLock<BTreeMap<Vec<u8>, LogRecordPos>>>,
}
