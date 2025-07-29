use anyhow::{Result, bail};
use std::sync::Arc;
use wasmtime::*;

pub struct DeterSLEngine {
    engine: Arc<Engine>,
}

impl DeterSLEngine {
    pub fn new() -> Self {
        DeterSLEngine { 
            engine: Arc::new(Engine::default())
        }
    }

    pub fn get_internal_engine(&self) -> &Engine {
        return self.engine.as_ref(); 
    }
}


