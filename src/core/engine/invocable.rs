use anyhow::anyhow;
use wasmtime::Store;
use std::time::Instant;

use crate::{core::{bindings, engine::detersl_engine::{DeterSLEngine}, execution::ExecutionState, types}};

pub struct DeterSLFuncInvocable {
    pub instance: Option<bindings::DeterslApi>,
    pub store: Option<Store<ExecutionState>>,
}

impl DeterSLFuncInvocable {
    pub fn new() -> Self {
        Self {
            instance: None,
            store: None
        }
    }

    pub fn make_store(&mut self, engine: &mut DeterSLEngine, execution_state: ExecutionState) -> anyhow::Result<()> {
        let store = Store::new(&engine.engine, execution_state);
        self.store = Some(store);
        Ok(())
    }

    pub fn fill_instance(&mut self, pre_instance: bindings::DeterslApiPre<ExecutionState>) -> anyhow::Result<()> {
        match &mut self.store {
            Some(store) => {
                let instance = pre_instance.instantiate(store)?;
                self.instance = Some(instance);
                Ok(())
            },
            None => {
                Err(anyhow!("there isnt any store"))
            }
        }
    }

    pub fn invoke(&mut self, input: types::Event) -> anyhow::Result<types::Output> {
	    let instance = self.instance.as_ref().ok_or_else(|| anyhow!("there isnt any instance"))?;
	    let store = self.store.as_mut().ok_or_else(|| anyhow!("there isnt any store"))?;
	    let output = instance
		    .detersl_api_func_handler()
		    .call_handle(store, &input.into_binding())?;
	    Ok(types::Output::from(output))
    }
}
