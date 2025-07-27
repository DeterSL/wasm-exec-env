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

    pub fn run_module(&self, module: &WasmModule) -> Result<Vec<Val>> {
        let wasm_module = Module::from_file(&self.engine, &module.module_path)?;

        let mut store = Store::new(&self.engine, ()); 
        let instance = Instance::new(&mut store, &wasm_module, &[])?;

        let func = instance.get_func(&mut store, &module.module_entry_point)
            .ok_or_else(|| anyhow::anyhow!(
                "function '{}' not found in module '{}'",
                &module.module_entry_point, &module.module_name
            ))?;

        let mut result: Vec<Val> = Vec::new();
        result.push(Val::I32(0));
        func.call(&mut store, &module.args[..], &mut result)?;

        Ok(result)
    }
}

pub struct WasmModule {
    pub module_name: String,
    pub module_path: String,
    pub module_entry_point: String,
    pub args: Vec<wasmtime::Val>
}

impl WasmModule {
    pub fn new(module_name: String, module_path: String, entry_point: String, args: Vec<wasmtime::Val>) -> Self {
        WasmModule { 
            module_name,
            module_path,
            module_entry_point: entry_point,
            args 
        }
    }
}
