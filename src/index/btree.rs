use std::collections::BTreeMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::data::log_record::LogRecordPos;

use super::Indexer;

pub struct Btree {
    tree: Arc<RwLock<BTreeMap<Vec<u8>, LogRecordPos>>>,
}

impl Btree {
    fn new() -> Self {
        Self {
            tree: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Indexer for Btree {
    fn put(&self, key: Vec<u8>, pos: LogRecordPos) -> bool {
        let mut write_guard = self.tree.write();
        write_guard.insert(key, pos);
        true
    }

    fn get(&self, key: Vec<u8>) -> Option<LogRecordPos> {
        let read_guard = self.tree.read();
        read_guard.get(&key).copied()
    }

    fn delete(&self, key: Vec<u8>) -> bool {
        let mut write_guard = self.tree.write();
        write_guard.remove(&key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_put() {
        let bt = Btree::new();
        let mut res = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 1,
            },
        );

        assert!(res);

        res = bt.put(
            "123".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 11,
            },
        );

        assert!(res);
    }

    #[test]
    fn test_btree_get() {
        let bt = Btree::new();

        let mut res = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 1,
            },
        );

        assert!(res);

        res = bt.put(
            "123".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 11,
            },
        );

        assert!(res);

        let mut pos = bt.get("".as_bytes().to_vec());
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().file_id, 1);
        assert_eq!(pos.unwrap().offset, 1);

        pos = bt.get("123".as_bytes().to_vec());
        assert!(pos.is_some());
        assert_eq!(pos.unwrap().file_id, 11);
        assert_eq!(pos.unwrap().offset, 11);

        pos = bt.get("not exist".as_bytes().to_vec());
        assert!(pos.is_none());
    }

    #[test]
    fn test_btree_delete() {
        let bt = Btree::new();
        let mut res = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 1,
            },
        );

        assert!(res);

        res = bt.put(
            "123".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 11,
            },
        );

        assert!(res);

        let mut del = bt.delete("".as_bytes().to_vec());
        assert!(del);

        del = bt.delete("".as_bytes().to_vec());
        assert!(!del);
    }
}
