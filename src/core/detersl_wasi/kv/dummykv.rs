use std::sync::{Arc, Mutex};

use crate::{core::{bindings, detersl_wasi::kv::KVType}};

#[derive(Clone, Default)]
pub struct DummyKV {
    inner: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
}

impl DummyKV {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(Default::default())) }
    }
}

impl bindings::detersl::kv_api::kv::Host for DummyKV {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        self.inner.lock().unwrap().get(&key).cloned()
    }

    fn set(&mut self, key: String, value: Vec<u8>) {
        self.inner.lock().unwrap().insert(key.to_string(), value);
    }

    fn delete(&mut self, key: String) -> bool {
        self.inner.lock().unwrap().remove(&key).is_some()
    }
}

impl KVType for DummyKV {}
