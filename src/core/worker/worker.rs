use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use wasmtime::{Engine, Config, component::{Linker, Component}, Store};
use wasmtime_wasi::p2::add_to_linker_sync;

use crate::{core::{execution::ExecutionState, types::{self, Output, Event}, bindings, utils::Cache}, config::func_config::{FuncBinaryConfig, FuncExecutionPolicy}};

use super::{linker_builder::{encode_execution_policy, self, LinkerBuilder}, linker_opts::get_linker_opts_from_execution_policy};

type WorkerError = Box<dyn std::error::Error + Send + Sync>;

pub struct FuncJob {
    pub config: FuncBinaryConfig,
    pub reply: oneshot::Sender<Result<types::Output, WorkerError>>,
}

pub struct Worker {
    engine: Engine,
    linker_cache: Cache<Linker<ExecutionState>>
}

impl Worker {
    pub fn new(engine_config: Config) -> Self {
        let engine = Engine::new(&engine_config).unwrap(); 
        let linker_cache = Cache::new();
        Self { engine , linker_cache}
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
               let linker = linker_builder.build();
               self.linker_cache.insert(encoded_policy.clone(), linker);
               let retrived_linker = self.linker_cache.get(&encoded_policy).unwrap();
               retrived_linker
           }
        }
    }

    pub fn run_func(&mut self, func_config: FuncBinaryConfig) -> Result<types::Output, WorkerError> {
        let mut store = Store::new(&self.engine, ExecutionState::new());
        let component = Component::from_file(&self.engine, func_config.func_binary_path)?;
        let linker_base_on_policy = self.generate_or_get_linker_from(func_config.func_execution_policy);
        let world = bindings::DeterslApi::instantiate(&mut store, &component, &linker_base_on_policy)?;
        let event: Event = func_config.func_input_event.into();
        let output = world.detersl_api_func_handler().call_handle(store, &event.into_binding())?;
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
