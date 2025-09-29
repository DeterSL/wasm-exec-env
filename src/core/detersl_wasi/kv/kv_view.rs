
use crate::core::detersl_wasi::kv::KVType;

pub trait KvView: Send {
    fn kv(&mut self) -> &mut dyn KVType;
}
