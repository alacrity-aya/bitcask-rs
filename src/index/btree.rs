use std::{collections::BTreeMap, sync::Arc};

use parking_lot::RwLock;

use crate::{data::log_record::LogRecordPos, options::IteratorOptions};

use super::{IndexTypeIterator, Indexer};

// BTree 索引，主要封装了标准库中的 BTreeMap 结构
pub struct BTree {
    tree: Arc<RwLock<BTreeMap<Vec<u8>, LogRecordPos>>>,
}

impl BTree {
    pub fn new() -> Self {
        Self {
            tree: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl Indexer for BTree {
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
        let remove_res = write_guard.remove(&key);
        remove_res.is_some()
    }

    fn iterator(&self, options: IteratorOptions) -> Box<dyn IndexTypeIterator> {
        let read_guard = self.tree.read();
        let mut items = Vec::with_capacity(read_guard.len());
        for (key, value) in read_guard.iter() {
            items.push((key.clone(), *value));
        }
        if options.reverse {
            items.reverse();
        }

        Box::new(BtreeIterator {
            items,
            curr_index: 0,
            options,
        })
    }
}

///BTree iterator
pub struct BtreeIterator {
    items: Vec<(Vec<u8>, LogRecordPos)>, //Vec<&Vec<u8>, &LogRecordPos> may be better?
    curr_index: usize,
    options: IteratorOptions,
}

impl IndexTypeIterator for BtreeIterator {
    fn rewind(&mut self) {
        self.curr_index = 0;
    }

    fn seek(&mut self, key: Vec<u8>) {
        self.curr_index = match self.items.binary_search_by(|(x, _)| {
            if self.options.reverse {
                x.cmp(&key).reverse()
            } else {
                x.cmp(&key)
            }
        }) {
            Ok(equal_val) => equal_val,
            Err(insert_val) => insert_val,
        };
    }

    fn next(&mut self) -> Option<(&Vec<u8>, &LogRecordPos)> {
        if self.curr_index >= self.items.len() {
            return None;
        }
        while let Some(item) = self.items.get(self.curr_index) {
            self.curr_index += 1;
            let prefix = &self.options.prefix;
            if prefix.is_empty() || item.0.starts_with(prefix) {
                return Some((&item.0, &item.1));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_btree_put() {
        let bt = BTree::new();
        let res1 = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        assert!(res1);

        let res2 = bt.put(
            "aa".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 22,
            },
        );
        assert!(res2);
    }

    #[test]
    fn test_btree_get() {
        let bt = BTree::new();
        let res1 = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        assert!(res1);
        let res2 = bt.put(
            "aa".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 22,
            },
        );
        assert!(res2);

        let pos1 = bt.get("".as_bytes().to_vec());
        assert!(pos1.is_some());
        assert_eq!(pos1.unwrap().file_id, 1);
        assert_eq!(pos1.unwrap().offset, 10);

        let pos2 = bt.get("aa".as_bytes().to_vec());
        assert!(pos2.is_some());
        assert_eq!(pos2.unwrap().file_id, 11);
        assert_eq!(pos2.unwrap().offset, 22);
    }

    #[test]
    fn test_btree_delete() {
        let bt = BTree::new();
        let res1 = bt.put(
            "".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        assert!(res1);
        let res2 = bt.put(
            "aa".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 11,
                offset: 22,
            },
        );
        assert!(res2);

        let del1 = bt.delete("".as_bytes().to_vec());
        assert!(del1);

        let del2 = bt.delete("aa".as_bytes().to_vec());
        assert!(del2);

        let del3 = bt.delete("not exist".as_bytes().to_vec());
        assert!(!del3);
    }

    #[test]
    fn test_btree_iterator_seek() {
        let bt = BTree::new();

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("aa".as_bytes().to_vec());
        let res = iter.next();
        assert!(res.is_none());

        bt.put(
            "ccde".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("aa".as_bytes().to_vec());
        let res = iter.next();
        assert!(res.is_some());

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("zz".as_bytes().to_vec());
        let res = iter.next();
        assert!(res.is_none());

        bt.put(
            "bbde".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        bt.put(
            "aade".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        bt.put(
            "cadd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("b".as_bytes().to_vec());
        while let Some(item) = iter.next() {
            assert!(!item.0.is_empty());
        }

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("cadd".as_bytes().to_vec());
        while let Some(item) = iter.next() {
            assert!(!item.0.is_empty());
        }

        let mut iter = bt.iterator(IteratorOptions::default());
        iter.seek("zzzzzzzz".as_bytes().to_vec());
        let res = iter.next();
        assert!(res.is_none());
    }

    #[test]
    fn test_btree_iterator_next() {
        let bt = BTree::new();

        bt.put(
            "ccde".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );

        bt.put(
            "bbde".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        bt.put(
            "aade".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );
        bt.put(
            "cadd".as_bytes().to_vec(),
            LogRecordPos {
                file_id: 1,
                offset: 10,
            },
        );

        let ops = IteratorOptions {
            prefix: Default::default(),
            reverse: true,
        };

        let mut iter = bt.iterator(ops);
        while let Some(item) = iter.next() {
            // println!("{:?}", String::from_utf8(item.0.to_vec()));
            assert!(!item.0.is_empty());
        }

        let ops = IteratorOptions {
            prefix: "c".as_bytes().to_vec(),
            reverse: false,
        };

        let mut iter = bt.iterator(ops);
        while let Some(item) = iter.next() {
            // println!("{:?}", String::from_utf8(item.0.to_vec()));
            assert!(!item.0.is_empty());
        }
    }
}
