use std::sync::Arc;

use tokio::sync::{mpsc, oneshot};

use wasmtime::{Engine, Config, component::{Linker, Component}, Store};
use wasmtime_wasi::p2::add_to_linker_sync;

use crate::{core::{execution::ExecutionState, types::{self, Output, Event}, bindings}, config::func_config::FuncBinaryConfig};

type WorkerError = Box<dyn std::error::Error + Send + Sync>;

pub struct FuncJob {
    pub config: FuncBinaryConfig,
    pub reply: oneshot::Sender<Result<types::Output, WorkerError>>,
}

pub struct Worker {
    engine: Engine,
    linker: Linker<ExecutionState>
}

impl Worker {
    pub fn new(engine_config: Config) -> Self {
        let engine = Engine::new(&engine_config).unwrap(); 
        let mut linker = Linker::new(&engine);
        add_to_linker_sync(&mut linker);
        Self { engine , linker }
    }

    pub fn run_func(&self, func_config: FuncBinaryConfig) -> Result<types::Output, WorkerError> {
        let mut store = Store::new(&self.engine, ExecutionState::new());
        let component = Component::from_file(&self.engine, func_config.func_binary_path)?;
        let world = bindings::DeterslApi::instantiate(&mut store, &component, &self.linker)?;
        let event: Event = func_config.func_input_event.into();
        let output = world.detersl_api_func_handler().call_handle(store, &event.into_binding())?;
        Ok(Output::from(output))
    }

    pub fn run_forever(&self, mut rx: mpsc::Receiver<FuncJob>) {
        while let Some(FuncJob { config, reply }) = rx.blocking_recv() {
            let res = self.run_func(config);
            // Ignore if requester dropped the receiver.
            let _ = reply.send(res);
        }
    }
}
