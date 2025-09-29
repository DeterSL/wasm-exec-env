
use std::{num::NonZeroUsize, path::PathBuf};

use lru::LruCache;
use wasmtime::{component::{Component, Linker}, Engine};
use wasmtime_wasi::p2::add_to_linker_sync;

use crate::{config::FuncBinaryConfig, core::{bindings, detersl_wasi::http, engine::config::DeterSLEngineConfig, execution::ExecutionState, fetcher::{get_one_time_component_fetcher_for_source, ComponentFetcher, OneTimeFetcherFn}}};

pub struct DeterSLFuncInfo {
    pub func_hash: String,
    pub func_name: String, 
    pub func_fetcher: OneTimeFetcherFn 
}

impl DeterSLFuncInfo {
    pub fn from_config(config: FuncBinaryConfig) -> anyhow::Result<Self> {
        let component_fetcher = get_one_time_component_fetcher_for_source(&config.func_binary_source)?;
        Ok(Self {
            func_name: config.func_name,
            func_hash: config.func_binary_hash,
            func_fetcher: component_fetcher
        })
    }

    pub fn fetch(&self) -> anyhow::Result<PathBuf> {
        (self.func_fetcher)()
    }
}

#[derive(Clone)]
pub struct DeterSLEngine {
    pub engine: Engine,
    pub instance_cache: LruCache<String, bindings::DeterslApiPre<ExecutionState>>,
    default_linker: Linker<ExecutionState>,
    config: DeterSLEngineConfig
}

impl DeterSLEngine {
    pub fn new(engine: Engine, config: DeterSLEngineConfig) -> anyhow::Result<Self> {
        let mut linker = Linker::new(&engine);
        add_to_linker_sync(&mut linker)?;
        http::add_only_http_to_linker_async(&mut linker)?;
        Ok(Self {
            engine,
            instance_cache: LruCache::<String, bindings::DeterslApiPre<ExecutionState>>::new(NonZeroUsize::new(config.LRUCacheCapacity).unwrap()),
            config,
            default_linker: linker
        })
    }

    pub fn compile_component(&self, detersl_func: &DeterSLFuncInfo) -> anyhow::Result<Component> {
        let component_path_buf = detersl_func.fetch()?;
        let component = Component::from_file_with_hash(&self.engine, component_path_buf.as_path(), detersl_func.func_hash.clone())?;
        Ok(component)
    }

    pub fn compile_component_and_cache_pre_instance(&mut self, detersl_func: &DeterSLFuncInfo) -> anyhow::Result<bindings::DeterslApiPre<ExecutionState>> {
        let component = self.compile_component(detersl_func)?;
        let instance_pre = self.default_linker.instantiate_pre(&component)?;
        let detersl_pre = bindings::DeterslApiPre::new(instance_pre)?;
        let clone = detersl_pre.clone();
        self.instance_cache.put(detersl_func.func_hash.clone(), detersl_pre);
        Ok(clone)
    }

    pub fn get_instance_from(&mut self, detersl_func: &DeterSLFuncInfo) -> anyhow::Result<bindings::DeterslApiPre<ExecutionState>> {
        match self.instance_cache.get(&detersl_func.func_hash) {
            Some(instance) => {
                Ok(instance.clone())
            },
            None => {
                let instance = self.compile_component_and_cache_pre_instance(detersl_func)?;
                Ok(instance)
            }
        }
    }
}
