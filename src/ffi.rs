#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type FuncBinaryConfig;
        type DeterSLEngine;
        type KvBox;
        type FfiExecutioner;

        fn func_config_from_json(json: &CxxString) -> Result<Box<FuncBinaryConfig>>;

        fn new_detersl_engine(cache_capacity: usize) -> Result<Box<DeterSLEngine>>;

        fn new_dummy_kv() -> Box<KvBox>;

        fn new_executioner(engine: &DeterSLEngine, kv: Box<KvBox>) -> Result<Box<FfiExecutioner>>;

        fn clone_kv(kv: &Box<KvBox>) -> Box<KvBox>; 

        fn executioner_run_cfg(self: &mut FfiExecutioner, cfg: &FuncBinaryConfig) -> Result<String>;

        fn executioner_run_json(self: &mut FfiExecutioner, json: &CxxString) -> Result<String>;
    }
}

use anyhow::{Context, Result};
use cxx::CxxString;
use serde_json;

use wasmtime::{Cache, CacheConfig, Config as WasmConfig, Engine as WasmEngine};

use crate::{config::FuncBinaryConfig, core::{detersl_wasi::kv::{DummyKV, KVType}, engine::{DeterSLEngine, DeterSLEngineConfig}, executioner::DeterSLExecutioner}};

fn func_config_from_json(json: &CxxString) -> Result<Box<FuncBinaryConfig>> {
    let cfg: FuncBinaryConfig = sonic_rs::from_str(json.to_str()?)
        .with_context(|| "failed to parse FuncBinaryConfig JSON")?;
    Ok(Box::new(cfg))
}

fn new_detersl_engine(cache_capacity: usize) -> Result<Box<DeterSLEngine>> {
    let mut engine_cfg = WasmConfig::new();
    let cache = Cache::new(CacheConfig::new()).context("failed to create Wasmtime cache")?;
    engine_cfg.cache(Some(cache));
    let wasmtime_engine =
        WasmEngine::new(&engine_cfg).context("failed to create Wasmtime Engine")?;

    let det_cfg = DeterSLEngineConfig::default().with_cache_capacity(cache_capacity);
    let det_engine = DeterSLEngine::new(wasmtime_engine, det_cfg)
        .context("failed to create DeterSLEngine")?;

    Ok(Box::new(det_engine))
}

pub struct KvBox {
    inner: Box<dyn KVType>,
}

impl KvBox {
    fn new(inner: Box<dyn KVType>) -> Self {
        Self { inner }
    }
    fn into_inner(self: Box<Self>) -> Box<dyn KVType> {
        self.inner
    }
}

fn new_dummy_kv() -> Box<KvBox> {
    Box::new(KvBox::new(Box::new(DummyKV::new())))
}

fn clone_kv(kv: &Box<KvBox>) -> Box<KvBox> {
    Box::new(KvBox::new(kv.inner.clone()))
}

pub struct FfiExecutioner {
    pub executioner: DeterSLExecutioner
}

fn new_executioner(engine: &DeterSLEngine, kv: Box<KvBox>) -> Result<Box<FfiExecutioner>> {
    let exec_engine = engine.clone();

    let executioner = DeterSLExecutioner::new(exec_engine).with_kv(kv.into_inner());

    Ok(Box::new(FfiExecutioner {
        executioner
    }))
}


impl FfiExecutioner {
    fn executioner_run_cfg(&mut self, cfg: &FuncBinaryConfig) -> Result<String> {
        let output = self.executioner.run_func_with_config(cfg.clone())
            .context("invoke failed")?;

        let json = serde_json::to_string(&output).context("serialize output failed")?;
        Ok(json)
    }

    fn executioner_run_json(&mut self, json: &CxxString) -> Result<String> {
        let cfg: FuncBinaryConfig = serde_json::from_str(json.to_str()?)
            .with_context(|| "failed to parse FuncBinaryConfig JSON")?;
        self.executioner_run_cfg(&cfg)
    }
}
