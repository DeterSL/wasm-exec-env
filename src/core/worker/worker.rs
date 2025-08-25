use std::{time::Instant, rc::Rc, cell::RefCell};

use tokio::sync::{mpsc, oneshot};

use wasmtime::{Engine, Config, component::{Linker, Component}, Store};

use crate::{core::{execution::ExecutionState, types::{self, Output, Event}, bindings, utils::Cache, detersl_wasi::kv::{DummyKV, KVRcMut, KVType}}, config::func_config::{FuncBinaryConfig, FuncExecutionPolicy}};

use super::{linker_builder::{encode_execution_policy, LinkerBuilder}, linker_opts::{get_linker_opts_from_execution_policy, AddKVToLinker, get_kv_as_opt}};

type WorkerError = Box<dyn std::error::Error + Send + Sync>;

pub struct FuncJob {
    pub config: FuncBinaryConfig,
    pub reply: oneshot::Sender<Result<types::Output, WorkerError>>,
}

pub struct Worker {
    engine: Engine,
    linker_cache: Cache<Linker<ExecutionState>>,
    kv: Rc<RefCell<Box<KVType>>>
}

impl Worker {
    pub fn new(engine_config: Config) -> Self {
        let engine = Engine::new(&engine_config).unwrap(); 
        let linker_cache = Cache::new();

        let dummy_kv: Box<KVType> = Box::new(DummyKV::new());
        let kv = Rc::new(RefCell::new(dummy_kv));
        Self { engine , linker_cache, kv: kv}
    }

    pub fn generate_or_get_linker_from(&mut self, execution_policy: FuncExecutionPolicy) -> Linker<ExecutionState> {
        let encoded_policy = encode_execution_policy(&execution_policy);
        let retrived_linker = self.linker_cache.get(&encoded_policy);
        match retrived_linker {
           Some(linker) => linker,
           None => {
               let mut linker_builder = LinkerBuilder::new(Linker::<ExecutionState>::new(&self.engine));
               let mut linker_opts = get_linker_opts_from_execution_policy(&execution_policy);
               linker_builder.add_opts(&mut linker_opts);
               linker_opts = get_kv_as_opt(self.kv.clone());
               linker_builder.add_opts(&mut linker_opts);
               let linker = linker_builder.build();
               
               self.linker_cache.insert(encoded_policy.clone(), linker);
               let retrived_linker = self.linker_cache.get(&encoded_policy).unwrap();
               retrived_linker
           }
        }
    }

    pub fn run_func(&mut self, func_config: FuncBinaryConfig) -> Result<types::Output, WorkerError> {
        let clone: KVRcMut = self.kv.clone();
        let mut store = Store::new(&self.engine, ExecutionState::new(clone));

        let mut start = Instant::now();
        let component = Component::from_file(&self.engine, func_config.func_binary_path)?;
        let mut end = Instant::now();
        log::info!("component built in {} microseconds", (end-start).as_micros());
    
        let linker_base_on_policy = self.generate_or_get_linker_from(func_config.func_execution_policy); start = Instant::now();
        let world = bindings::DeterslApi::instantiate(&mut store, &component, &linker_base_on_policy)?;
        end = Instant::now();
        log::info!("instantiate done in {} microseconds", (end-start).as_micros());

        let event: Event = func_config.func_input_event.into();

        start = Instant::now();
        let output = world.detersl_api_func_handler().call_handle(store, &event.into_binding())?;
        end = Instant::now();
        log::info!("call done in {} microseconds", (end-start).as_micros());

        Ok(Output::from(output))
    }

    pub fn run_forever(&mut self, mut rx: mpsc::Receiver<FuncJob>) {
        while let Some(FuncJob { config, reply }) = rx.blocking_recv() {
            let res = self.run_func(config);
            // Ignore if requester dropped the receiver.
            let _ = reply.send(res);
        }
    }
}
