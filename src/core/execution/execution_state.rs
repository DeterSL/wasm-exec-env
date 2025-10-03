
use wasmtime::EventHandler;
use wasmtime_wasi::{p2::{WasiCtx, IoView, WasiView, WasiCtxBuilder, event_handler::EventHandlerImpl}, ResourceTable};

use crate::{config::{make_filters, FuncExecutionPolicy, FuncInitValue}, core::detersl_wasi::{self, http::{DeterSLHttpCtx, DeterSLHttpView}, kv::{KVType, KvView}}};


pub struct ExecutionState {
    ctx: WasiCtx,
    table: ResourceTable,
    kv: Box<dyn KVType>,
    http_ctx: DeterSLHttpCtx
}

unsafe impl Send for ExecutionState {}

impl IoView for ExecutionState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

impl WasiView for ExecutionState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

impl KvView for ExecutionState {
    fn kv(&mut self) -> &mut dyn KVType {
        &mut *self.kv
    }
}

impl DeterSLHttpView for ExecutionState {
    fn ctx(&mut self) -> &mut DeterSLHttpCtx {
       &mut self.http_ctx 
    }
}

impl ExecutionState {
    pub fn new(kv: Box<dyn KVType>, execution_policy: &FuncExecutionPolicy, inital_values: &FuncInitValue) -> ExecutionState {
        let mut wasi = WasiCtxBuilder::new();

        #[cfg(not(feature = "noop-logger"))]
        {
            wasi.set_logger(detersl_wasi::logger::SimpleLogger::new());
        }

        ExecutionState::apply_initial_values(&mut wasi, inital_values);

        #[cfg(feature = "noop-filter")]
        {
            use wasmtime_wasi::p2::event_handler::NoopHandler;

            let handler = NoopHandler{};
            // set_event_handler accepts impl EventHandler + 'static
            wasi.set_event_handler(handler);
        }

        #[cfg(not(feature = "noop-filter"))]
        {
            let mut event_filter = EventHandlerImpl::new();
            let filters = make_filters(execution_policy);

            for (td, filter_fn) in filters {
                event_filter.register(td, filter_fn);
            }

            wasi.set_event_handler(event_filter);
        }

        ExecutionState {
            ctx: wasi.build(),
            http_ctx: DeterSLHttpCtx::new(),
            table: ResourceTable::new(),
            kv
        }
    }

    fn apply_initial_values(wasi: &mut WasiCtxBuilder, inital_values: &FuncInitValue) {
        wasi.monotonic_clock(detersl_wasi::clock::DeterSLMonotonicWallClock::new(inital_values.init_clock));
        wasi.wall_clock(detersl_wasi::clock::DeterSLWallClock::from_nanos(inital_values.init_clock));
        wasi.insecure_random_seed(inital_values.random_seed);
        wasi.insecure_random(detersl_wasi::random::ConstantRng::new(inital_values.random_seed.try_into().unwrap()));
        wasi.secure_random(detersl_wasi::random::ConstantRng::new(inital_values.random_seed.try_into().unwrap()));
    }
}

