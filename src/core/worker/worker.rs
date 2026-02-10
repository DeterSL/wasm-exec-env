use std::time::Instant;

use tokio::sync::{mpsc, oneshot};
use wasmtime::Config;

use crate::{config::FuncBinaryConfig, core::{detersl_wasi::kv::{DummyKV, KVType}, engine::{DeterSLEngine, DeterSLEngineConfig}, executioner::DeterSLExecutioner, types}};

// Keep your error alias
type WorkerError = anyhow::Error;

pub struct FuncJob {
    pub config: FuncBinaryConfig,
    pub reply: oneshot::Sender<Result<types::Output, WorkerError>>,
}

// Executioner-based worker
pub struct Worker {
    exec: DeterSLExecutioner,
}

impl Worker {
    // Build a worker with a fresh DeterSLEngine and a default DummyKV backend.
    // You can add overloads to inject a custom KV or custom engine if needed.
    #[allow(dead_code)]
    pub fn new(engine_config: Config, detersl_engine_config: DeterSLEngineConfig) -> Self {
        let engine = wasmtime::Engine::new(&engine_config).expect("engine");
        let detersl_engine = DeterSLEngine::new(engine, detersl_engine_config).expect("couldnt made the engine");

        // Build an executioner and inject a KV backend.
        let mut exec = DeterSLExecutioner::new(detersl_engine);
        let kv: Box<dyn KVType> = Box::new(DummyKV::new());
        exec = exec.with_kv(kv);

        Self { exec }
    }

    // Optional convenience constructor when you already have a DeterSLEngine and KV
    pub fn from_parts(engine: DeterSLEngine) -> Self {
        let mut exec = DeterSLExecutioner::new(engine);
        let kv: Box<dyn KVType> = Box::new(DummyKV::new());
        exec = exec.with_kv(kv);
        Self { exec }
    }

    pub fn run_func(&mut self, func_config: FuncBinaryConfig) -> Result<types::Output, WorkerError> {
        // The executioner handles compile/cache/instantiate/invoke internally
        self.exec
            .run_func_with_config(func_config)
            .map_err(|e| -> WorkerError { e.into() })
    }

    pub fn run_forever(&mut self, mut rx: mpsc::Receiver<FuncJob>) {
        // Synchronous loop receiving jobs from an async channel is fine here.
        while let Some(FuncJob { config, reply }) = rx.blocking_recv() {
            let start = Instant::now();
            let res = self.run_func(config);
            let end = Instant::now();
            log::info!("worker run_func finished in {} µs", (end - start).as_micros());
            // Ignore if requester dropped the receiver.
            let _ = reply.send(res);
        }
    }
}

