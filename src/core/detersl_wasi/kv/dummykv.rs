use std::sync::Mutex;

use crate::core::bindings;

pub struct DummyKV {
    inner: Mutex<std::collections::HashMap<String, Vec<u8>>>,
}

unsafe impl Send for DummyKV {}

impl DummyKV {
    pub fn new() -> Self {
        Self { inner: Mutex::new(Default::default()) }
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
