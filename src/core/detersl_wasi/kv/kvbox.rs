use crate::core::detersl_wasi::kv::KVType;

/// KvBox is a wrapper around any `KVType` instance, allowing dynamic dispatch.
pub struct KvBox {
    pub inner: Box<dyn KVType>,
}

impl KvBox {
    pub fn new(inner: Box<dyn KVType>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self: Box<Self>) -> Box<dyn KVType> {
        self.inner
    }
}

