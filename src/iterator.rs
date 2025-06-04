use std::sync::Arc;

use parking_lot::RwLock;

use crate::{db::Engine, index::IndexTypeIterator, options::IteratorOptions};

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
}
