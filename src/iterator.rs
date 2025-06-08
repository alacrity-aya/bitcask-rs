use bytes::Bytes;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::{db::Engine, errors::Result, index::IndexTypeIterator, options::IteratorOptions};

/// iterator interface
pub struct Iterator<'a> {
    index_iter: Arc<RwLock<Box<dyn IndexTypeIterator>>>,
    engine: &'a Engine,
}

impl Engine {
    pub fn iter(&self, options: IteratorOptions) -> Iterator {
        Iterator {
            index_iter: Arc::new(RwLock::new(self.index.iterator(options))),
            engine: self,
        }
    }

    pub fn list_keys(&self) -> Result<Vec<bytes::Bytes>> {
        // may be optional there is better?
        self.index.list_keys()
    }

    pub fn fold<F>(&self, f: F) -> Result<()>
    //there is no need to Result<()> i think
    where
        Self: Sized,
        F: Fn(Bytes, Bytes) -> bool,
    {
        let iter = self.iter(IteratorOptions::default());
        while let Some((key, value)) = iter.next() {
            if !f(key, value) {
                break;
            }
        }
        Ok(())
    }
}

impl Iterator<'_> {
    pub fn rewind(&self) {
        let mut index_iter = self.index_iter.write();
        index_iter.rewind();
    }

    pub fn seek(&self, key: Vec<u8>) {
        let mut index_iter = self.index_iter.write();
        index_iter.seek(key);
    }

    pub fn next(&self) -> Option<(Bytes, Bytes)> {
        let mut index_iter = self.index_iter.write();
        if let Some(item) = index_iter.next() {
            let value = self
                .engine
                .get_value_by_position(item.1)
                .expect("failed to get value from data file");
            return Some((Bytes::from(item.0.to_vec()), value));
        }
        None
    }
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use crate::{options::Options, util::rand_kv::get_test_value};

    use super::*;

    #[test]
    fn test_iterator_seek() {
        let mut opts = Options::default();

        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
        opts.data_file_size = 64 * 1024 * 1024;
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let iter = engine.iter(IteratorOptions::default());
        iter.seek("aa".as_bytes().to_vec());
        assert!(iter.next().is_none());

        let put_res = engine.put(Bytes::from("aacc"), get_test_value(10));
        assert!(put_res.is_ok());

        let iter = engine.iter(IteratorOptions::default());
        iter.seek("a".as_bytes().to_vec());
        assert!(iter.next().is_some());

        let put_res = engine.put(Bytes::from("eecc"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("bbac"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("ccde"), get_test_value(10));
        assert!(put_res.is_ok());

        let iter = engine.iter(IteratorOptions::default());
        iter.seek("a".as_bytes().to_vec());
        assert_eq!(iter.next().unwrap().0, Bytes::from("aacc"));

        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_list_keys() {
        let opts = Options {
            dir_path: PathBuf::from("/tmp/bitcask-rs-list-key"),
            ..Default::default()
        };

        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let keys = engine.list_keys();
        assert_eq!(keys.ok().unwrap().len(), 0);

        let put_res = engine.put(Bytes::from("eecc"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("bbac"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("ccaa"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("aabb"), get_test_value(10));
        assert!(put_res.is_ok());

        let keys = engine.list_keys();
        assert_eq!(keys.ok().unwrap().len(), 4);

        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }

    #[test]
    fn test_fold() {
        let opts = Options {
            dir_path: PathBuf::from("/tmp/bitcask-rs-fold"),
            ..Default::default()
        };
        let engine = Engine::open(opts.clone()).expect("failed to open engine");

        let put_res = engine.put(Bytes::from("eecc"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("bbac"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("ccaa"), get_test_value(10));
        assert!(put_res.is_ok());
        let put_res = engine.put(Bytes::from("aabb"), get_test_value(10));
        assert!(put_res.is_ok());

        engine
            .fold(|key, value| {
                assert!(!key.is_empty());
                assert!(!value.is_empty());
                if key.ge(&"cc") {
                    return false;
                }
                true
            })
            .unwrap();

        std::fs::remove_dir_all(opts.clone().dir_path).expect("failed to remove path");
    }
}
