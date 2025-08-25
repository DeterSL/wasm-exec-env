use std::cell::RefMut;

use super::{KVType, KVRcMut, KVRefCellMut};

pub trait KvView: Send {
    fn kv(&mut self) -> &KVRefCellMut;
}
