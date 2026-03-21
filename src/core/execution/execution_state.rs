use log::warn;
use wasmtime::EventHandler;
use wasmtime_wasi::{
    p2::{WasiCtx, IoView, WasiView, WasiCtxBuilder, event_handler::EventHandlerImpl},
    ResourceTable,
};

use crate::{
    config::{make_filters, FuncExecutionPolicy, FuncInitValue},
    core::detersl_wasi::{
        self,
        http::{DeterSLHttpCtx, DeterSLHttpView},
        kv::{KVType, KvView},
    },
};

pub struct ExecutionState {
    ctx: WasiCtx,
    table: ResourceTable,
    kv: Box<dyn KVType>,
    http_ctx: DeterSLHttpCtx,
}

unsafe impl Send for ExecutionState {}

impl IoView for ExecutionState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for ExecutionState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
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
    pub fn new(
        kv: Box<dyn KVType>,
        execution_policy: &FuncExecutionPolicy,
        initial_values: &FuncInitValue,
    ) -> ExecutionState {
        ExecutionState {
            ctx: Self::build_wasi_ctx(execution_policy, initial_values),
            table: ResourceTable::new(),
            kv,
            http_ctx: DeterSLHttpCtx::new(),
        }
    }

    pub fn reset(
        &mut self,
        execution_policy: &FuncExecutionPolicy,
        initial_values: &FuncInitValue,
    ) {
        self.ctx = Self::build_wasi_ctx(execution_policy, initial_values);

        // The language runtime might place some resource on the resource table
        if !self.table.is_empty() {
            warn!("Resource table is not empty upon reseting the execution state!")
        }
        
        self.table = ResourceTable::new();

        // TODO: is it safe to replace the http ctx?
        // We need to answer this when we have outbox.
        self.http_ctx = DeterSLHttpCtx::new();
        // keep existing kv as-is
    }

    pub fn reset_with_kv(
        &mut self,
        kv: Box<dyn KVType>,
        execution_policy: &FuncExecutionPolicy,
        initial_values: &FuncInitValue,
    ) {
        self.kv = kv;
        self.reset(execution_policy, initial_values);
    }

    fn build_wasi_ctx(
        execution_policy: &FuncExecutionPolicy,
        initial_values: &FuncInitValue,
    ) -> WasiCtx {
        let mut wasi = WasiCtxBuilder::new();

        #[cfg(not(feature = "noop-logger"))]
        {
            wasi.set_logger(detersl_wasi::logger::SimpleLogger::new());
        }

        Self::apply_initial_values(&mut wasi, initial_values);

        #[cfg(feature = "noop-filter")]
        {
            use wasmtime_wasi::p2::event_handler::NoopHandler;

            wasi.set_event_handler(NoopHandler {});
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

        wasi.build()
    }

    fn apply_initial_values(wasi: &mut WasiCtxBuilder, initial_values: &FuncInitValue) {
        wasi.monotonic_clock(
            detersl_wasi::clock::DeterSLMonotonicWallClock::new(initial_values.init_clock),
        );
        wasi.wall_clock(
            detersl_wasi::clock::DeterSLWallClock::from_nanos(initial_values.init_clock),
        );
        wasi.insecure_random_seed(initial_values.random_seed);
        wasi.insecure_random(
            detersl_wasi::random::ConstantRng::new(initial_values.random_seed.try_into().unwrap()),
        );
        wasi.secure_random(
            detersl_wasi::random::ConstantRng::new(initial_values.random_seed.try_into().unwrap()),
        );
    }
}
