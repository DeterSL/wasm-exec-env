use std::{path::PathBuf, sync::Arc, sync::OnceLock};

use anyhow::Context;
use moka::sync::Cache; // Import synchronous Moka cache
use once_cell::sync::Lazy;
use wasmtime::{
    component::{Component, Linker},
    Engine,
};
use wasmtime_wasi::p2::add_to_linker_sync;

use crate::{
    config::FuncBinaryConfig,
    core::{
        bindings,
        detersl_linker::{AddKVToLinker, LinkerOption},
        detersl_wasi::http,
        engine::config::DeterSLEngineConfig,
        execution::ExecutionState,
        fetcher::{get_one_time_component_fetcher_for_source, OneTimeFetcherFn},
    },
};

/// Static for handling the global shared cache size initialization.
static CACHE_CAPACITY: OnceLock<u64> = OnceLock::new();

/// Singleton for the shared global cache.
static GLOBAL_SHARED_CACHE: Lazy<Arc<Cache<String, bindings::DeterslApiPre<ExecutionState>>>> =
    Lazy::new(|| {
        let capacity = CACHE_CAPACITY
            .get()
            .copied()
            .unwrap_or(1000); // Default to 1000 if no engine initialized the capacity.

        Arc::new(Cache::builder().max_capacity(capacity).build())
    });

/// Function metadata for fetching and managing compilation
pub struct DeterSLFuncInfo {
    pub func_hash: String,
    pub func_fetcher: OneTimeFetcherFn,
}

impl DeterSLFuncInfo {
    /// Create function metadata from a configuration.
    pub fn from_config(config: FuncBinaryConfig) -> anyhow::Result<Self> {
        let component_fetcher =
            get_one_time_component_fetcher_for_source(&config.func_binary_source)?;
        Ok(Self {
            func_hash: config.func_binary_hash,
            func_fetcher: component_fetcher,
        })
    }

    /// Fetch a Wasm component for this function.
    pub fn fetch(&self) -> anyhow::Result<PathBuf> {
        (self.func_fetcher)()
    }
}

/// Wrapper around the Wasmtime engine and cache for managing compiled components
pub struct DeterSLEngine {
    pub engine: Engine,
    pub instance_cache: Arc<Cache<String, bindings::DeterslApiPre<ExecutionState>>>, // Shared Cache
    default_linker: Linker<ExecutionState>,
    config: DeterSLEngineConfig,
}

impl DeterSLEngine {
    /// Initializes the global cache capacity if this is the first engine being created.
    fn initialize_cache_capacity(config: &DeterSLEngineConfig) {
        CACHE_CAPACITY
            .set(config.lrucache_capacity as u64)
            .ok(); // Ignore if already set
    }

    /// Create a new engine. Initializes the cache on the first instantiation.
    pub fn new(engine: Engine, config: DeterSLEngineConfig) -> anyhow::Result<Self> {
        // Initialize the cache capacity during the first engine's creation.
        Self::initialize_cache_capacity(&config);

        let mut linker = Linker::new(&engine);
        add_to_linker_sync(&mut linker)?; // Adds WASI linker
        http::add_only_http_to_linker_async(&mut linker)?;
        let mut kv_linker = AddKVToLinker::new();
        kv_linker.apply_to_linker(&mut linker)?;

        Ok(Self {
            engine,
            default_linker: linker,
            config,
            instance_cache: GLOBAL_SHARED_CACHE.clone(), // Reference the global cache
        })
    }

    /// Compile a component and return it.
    pub fn compile_component(
        &self,
        detersl_func: &DeterSLFuncInfo,
    ) -> anyhow::Result<Component> {
        let component_path_buf = detersl_func.fetch().context("failed to fetch the component")?;
        let component = Component::from_file_with_hash(
            &self.engine,
            component_path_buf.as_path(),
            detersl_func.func_hash.clone(),
        )
        .context("failed to load component from file")?;
        Ok(component)
    }

    /// Compile a component and cache the pre-instantiated binding.
    pub fn compile_component_and_cache_pre_instance(
        &mut self,
        detersl_func: &DeterSLFuncInfo,
    ) -> anyhow::Result<bindings::DeterslApiPre<ExecutionState>> {
        let component = self.compile_component(detersl_func).context("failed to compile component")?;
        let instance_pre = self
            .default_linker
            .instantiate_pre(&component)
            .context("failed to make a pre-instance from component")?;
        let detersl_pre = bindings::DeterslApiPre::new(instance_pre)
            .context("failed to make a pre-instance")?;
        let clone = detersl_pre.clone();

        self.instance_cache
            .insert(detersl_func.func_hash.clone(), detersl_pre); // Cache the pre-instance

        Ok(clone)
    }

    /// Retrieve a cached instance or compile and cache it.
    pub fn get_instance_from(
        &mut self,
        detersl_func: &DeterSLFuncInfo,
    ) -> anyhow::Result<bindings::DeterslApiPre<ExecutionState>> {
        match self.instance_cache.get(&detersl_func.func_hash) {
            Some(instance) => Ok(instance.clone()), // Cache hit
            None => {
                let instance = self
                    .compile_component_and_cache_pre_instance(detersl_func)
                    .context("failed to compile and cache pre-instance")?;
                Ok(instance)
            }
        }
    }
}

impl Clone for DeterSLEngine {
    /// Cloning only increments the reference to the shared cache.
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            config: self.config.clone(),
            default_linker: self.default_linker.clone(),
            instance_cache: Arc::clone(&self.instance_cache), // Cache is shared
        }
    }
}
