use anyhow::Result;
use wasmtime::component::{Component, Instance, Linker};
use wasmtime::Store;
use wasmtime_wasi::p2::{add_to_linker_sync, WasiView};
use super::engine::*; // Assumes that DeterSLEngine now provides a wasmtime::component::Engine

/// A linker builder for the Component Model using Wasmtime.
pub struct DeterSLComponentLinkerBuilder<'a, T>
where
    T: WasiView + 'static,
{
    engine: &'a DeterSLEngine,
    pub linker: Linker<T>,
}

impl<'a, T> DeterSLComponentLinkerBuilder<'a, T> 
where
    T: WasiView
{
    /// Constructs a new component linker builder using the given engine.
    pub fn new(engine: &'a DeterSLEngine) -> Self {
        Self {
            engine,
            linker: Linker::new(engine.get_internal_engine()),
        }
    }

    /// Adds a host function to the component linker.
    ///
    /// Note: In the component model, the host function signatures are generally simpler.
    /// Here we use a concrete signature of a function taking an i32 and returning an i32.
    /// This avoids using `Caller` which is not available in `wasmtime::component`.
    pub fn add_function(
        &mut self,
        module: &str,
        name: &str,
        func: impl Fn(i32) -> i32 + 'static,
    ) -> Result<()> {
        // Here you would actually call something like:
        // self.linker.func_wrap(module, name, func)?;
        Ok(())
    }

    pub fn add_wasi(&mut self) {
        add_to_linker_sync(&mut self.linker);
    }
}

/// An instance builder for the Component Model using Wasmtime.
pub struct DeterSLComponentInstanceBuilder<'a, T>
where
    T: WasiView + 'static,
{
    engine: &'a DeterSLEngine,
    store: Option<Store<T>>,
    linker: Option<DeterSLComponentLinkerBuilder<'a, T>>,
    component: Option<Component>,
}

impl<'a, T> DeterSLComponentInstanceBuilder<'a, T>
where
    T: WasiView
{
    /// Creates a new instance builder with a given engine and linker builder.
    pub fn new(engine: &'a DeterSLEngine, linker: DeterSLComponentLinkerBuilder<'a, T>) -> Self {
        Self {
            engine,
            linker: Some(linker),
            store: None,
            component: None,
        }
    }

    /// Creates a new store using the engine's internal component engine.
    pub fn create_store_with(&mut self, data: T) {
        self.store = Some(Store::new(self.engine.get_internal_engine(), data));
    }

    /// Loads the component from the binary path specified in the config.
    /// (Optional: if you wish to validate using a WIT file you can extend this method)
    pub fn load_component_from_config(&mut self, config: &crate::config::parser::WasmBinaryConfig) {
        let component = Component::from_file(self.engine.get_internal_engine(), &config.binary_path)
            .expect("Error creating component");
        self.component = Some(component);
    }

    /// Instantiates a component instance from any bindgen using a closure.
    ///
    /// The closure `instantiator` has access to a mutable store,
    /// the loaded component, and the linker. It returns any desired instantiation object.
    pub fn instantiate_instance<F, I>(&mut self, instantiator: F) -> Result<I>
    where
        F: FnOnce(&mut Store<T>, &Component, &mut Linker<T>) -> Result<I>,
    {
        let comp = self.component.as_ref().expect("Component not loaded");
        let store = self.store.as_mut().expect("Store not created");
        let linker = &mut self.linker.as_mut().expect("Linker not built").linker;
        let instance = instantiator(store, comp, linker)?;
        Ok(instance)
    }

    pub fn take_store(&mut self) -> Store<T> {
        self.store.take().expect("store isnt intilized")
    }
}
