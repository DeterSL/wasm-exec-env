use std::sync::Arc;
use anyhow::Result;

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

    pub fn run(&mut self, path: &str) -> Result<()> {
        let module = Module::from_file(&self.engine, path)?;

        let mut store = Store::new(&self.engine, ()); 

        let instance = Instance::new(&mut store, &module, &[])?;

        let run = instance.get_typed_func::<(), ()>(&mut store, "hello")?;
        run.call(&mut store, ())?;

        Ok(())
    }
}

