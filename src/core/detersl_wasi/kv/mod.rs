mod dummykv;
mod kv_view;

use std::{rc::Rc, cell::{RefCell, RefMut}};

use crate::core::bindings;

pub type KVType = dyn bindings::detersl::kv_api::kv::Host + Send + 'static;
pub type KVRcMut = Rc<RefCell<Box<KVType>>>;
pub type KVRefCellMut = RefCell<Box<KVType>>;
pub type KVRefMut<'a> = RefMut<'a, Box<KVType>>;

impl<'a, T: bindings::detersl::kv_api::kv::Host + ?Sized> bindings::detersl::kv_api::kv::Host for RefMut<'a, Box<T>> {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        (&mut **self).get(key)
    }

    fn set(&mut self, key: String, value: Vec<u8>) {
        (&mut **self).set(key, value)
    }

    fn delete(&mut self, key: String) -> bool {
        (&mut **self).delete(key)
    }
}

pub use dummykv::DummyKV;
pub use kv_view::KvView;


