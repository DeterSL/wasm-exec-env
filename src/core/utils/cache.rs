use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct Cache<V> {
    inner: Arc<RwLock<HashMap<String, V>>>,
}

impl<V> Clone for Cache<V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<V> Cache<V> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
        }
    }

    pub fn insert(&self, key: impl Into<String>, value: V) -> Option<V> {
        let mut write = self.inner.write().expect("RwLock poisoned");
        write.insert(key.into(), value)
    }

    pub fn get(&self, key: &str) -> Option<V>
    where
        V: Clone,
    {
        let read = self.inner.read().expect("RwLock poisoned");
        read.get(key).cloned()
    }

    pub fn remove(&self, key: &str) -> Option<V> {
        let mut write = self.inner.write().expect("RwLock poisoned");
        write.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        let read = self.inner.read().expect("RwLock poisoned");
        read.contains_key(key)
    }

    pub fn len(&self) -> usize {
        let read = self.inner.read().expect("RwLock poisoned");
        read.len()
    }

    pub fn clear(&self) {
        let mut write = self.inner.write().expect("RwLock poisoned");
        write.clear()
    }
}

impl<V> Default for Cache<V> {
    fn default() -> Self {
        Self::new()
    }
}
