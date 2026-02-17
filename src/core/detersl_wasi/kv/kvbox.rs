use crate::core::detersl_wasi::kv::KVType;

/// KvBox is a wrapper around any `KVType` instance, allowing dynamic dispatch.
#[allow(dead_code)]
pub struct KvBox {
    pub inner: Box<dyn KVType>,
}

#[allow(dead_code)]
impl KvBox {
    pub fn new(inner: Box<dyn KVType>) -> Self {
        Self { inner }
    }

    pub fn into_inner(self: Box<Self>) -> Box<dyn KVType> {
        self.inner
    }
}

