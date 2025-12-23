mod dummykv;
mod kv_view;
mod kvbox;

use crate::core::bindings;

pub type DynKVType = dyn bindings::detersl::kv_api::kv::Host;

pub trait KVType: bindings::detersl::kv_api::kv::Host + KVTypeClone {}

pub trait KVTypeClone {
    fn clone_to_box(&self) -> Box<dyn KVType>;
}

impl<T> KVTypeClone for T where T: KVType + Clone + 'static {
    fn clone_to_box(&self) -> Box<dyn KVType> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn KVType> {
    fn clone(&self) -> Self {
       self.clone_to_box() 
    }
}

impl<'a, T: bindings::detersl::kv_api::kv::Host> bindings::detersl::kv_api::kv::Host for Box<T> {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        T::get(self, key)
    }

    fn set(&mut self, key: String, value: Vec<u8>) {
        T::set(self, key, value)
    }

    fn delete(&mut self, key: String) -> bool {
        T::delete(self, key)
    }
}

pub use dummykv::DummyKV;
pub use kv_view::KvView;
pub use kvbox::KvBox;
