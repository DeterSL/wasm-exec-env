
use super::KVRefCellMut;

pub trait KvView: Send {
    fn kv(&mut self) -> &KVRefCellMut;
}
