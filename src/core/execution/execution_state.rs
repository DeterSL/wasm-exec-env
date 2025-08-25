use std::rc::Rc;

use wasmtime_wasi::{p2::{WasiCtx, IoView, WasiView, WasiCtxBuilder}, ResourceTable};

use crate::core::{detersl_wasi::{self, kv::{KVType, KVRcMut, KvView, KVRefCellMut}}};


pub struct ExecutionState {
    ctx: WasiCtx,
    table: ResourceTable,
    kv: KVRcMut
}

unsafe impl Send for ExecutionState {}

impl IoView for ExecutionState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

impl WasiView for ExecutionState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

impl KvView for ExecutionState {
    fn kv(&mut self) -> &KVRefCellMut {
        &*self.kv
    }
}

impl ExecutionState {
    pub fn new(kv: KVRcMut) -> ExecutionState {
        let mut wasi = WasiCtxBuilder::new();
        
        wasi.monotonic_clock(detersl_wasi::clock::DeterSLMonotonicWallClock::new());
        wasi.wall_clock(detersl_wasi::clock::DeterSLWallClock::new());
        wasi.insecure_random_seed(42);
        wasi.insecure_random(detersl_wasi::random::ConstantRng::new(42));
        wasi.secure_random(detersl_wasi::random::ConstantRng::new(42));
        wasi.set_logger(detersl_wasi::logger::SimpleLogger::new());

        ExecutionState {
            ctx: wasi.build(),
            table: ResourceTable::new(),
            kv
        }
    }
}

