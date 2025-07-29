use anyhow::Result;
use wasmtime::{Caller, Func, Instance, Store, Val};

pub struct Execution<T>
where
    T: 'static,
{
    pub store: Store<T>,
    pub instance: Instance,
}

impl<T> Execution<T>
where
    T: 'static,
{
    pub fn new(store: Store<T>, instance: Instance) -> Self {
        Self { store, instance }
    }

    pub fn get_func(&mut self, name: &str) -> Option<Func> {
        self.instance.get_func(&mut self.store, name)
    }

    /// Calls an exported function by name with the provided arguments.
    /// The `result_count` parameter specifies the expected number of results.
    /// This method prepares a vector of default values and passes its mutable slice
    /// to the function call, which then populates it with the actual results.
    pub fn call_func(&mut self, name: &str, args: &[Val], result_count: usize) -> Result<Vec<Val>> {
        let func = self
            .get_func(name)
            .ok_or_else(|| anyhow::anyhow!("Function '{}' not found", name))?;
        
        // Prepare a vector of default values.
        let mut results: Vec<Val> = (0..result_count)
            .map(|_| Val::I32(0))
            .collect();
        
        // Call the function with the provided arguments and the mutable results slice.
        func.call(&mut self.store, args, results.as_mut_slice())?;
        Ok(results)
    }
}
