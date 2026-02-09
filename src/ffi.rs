use anyhow::{Context, Result};
use cxx::CxxString;
use serde_json;
use std::pin::Pin;
use wasmtime::{Cache, CacheConfig, Config as WasmConfig, Engine as WasmEngine};

use crate::{
    config::FuncBinaryConfig,
    core::{
        bindings,
        detersl_wasi::kv::{DummyKV, KVType, KVTypeClone, KvBox},
        engine::{DeterSLEngine, DeterSLEngineConfig},
        executioner::DeterSLExecutioner,
    },
};

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        // Engine / executioner
        type FuncBinaryConfig;
        type DeterSLEngine;
        type KvBox;
        type FfiExecutioner;

        fn func_config_from_json(json: &CxxString) -> Result<Box<FuncBinaryConfig>>;
        fn new_detersl_engine(cache_capacity: usize) -> Result<Box<DeterSLEngine>>;
        fn new_executioner(engine: &DeterSLEngine, kv: Box<KvBox>) -> Result<Box<FfiExecutioner>>;
        fn executioner_run_cfg(self: &mut FfiExecutioner, cfg: &FuncBinaryConfig) -> Result<String>;
        fn executioner_run_json(self: &mut FfiExecutioner, json: &CxxString) -> Result<String>;
        fn executioner_compile_json(self: &mut FfiExecutioner, json: &CxxString) -> Result<()>;

        // KV-side constructors
        fn new_dummy_kv() -> Box<KvBox>;
        fn clone_kv(kv: &Box<KvBox>) -> Box<KvBox>;
        unsafe fn new_cpp_kv(kv_interface: *mut KVInterface) -> Box<KvBox>;
    }

    unsafe extern "C++" {
        include!("kv_api.h");
        type KVInterface;

        fn get(self: Pin<&mut KVInterface>, key: &str) -> &[u8];
        fn set(self: Pin<&mut KVInterface>, key: &str, value: Vec<u8>);
        fn delete_key(self: Pin<&mut KVInterface>, key: &str) -> bool;
    }
}

// ---------- Engine / executioner ----------

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

pub struct FfiExecutioner {
    pub executioner: DeterSLExecutioner,
}

fn new_executioner(engine: &DeterSLEngine, kv: Box<KvBox>) -> Result<Box<FfiExecutioner>> {
    let exec_engine = engine.clone();
    let executioner = DeterSLExecutioner::new(exec_engine).with_kv(kv.into_inner());
    Ok(Box::new(FfiExecutioner { executioner }))
}

impl FfiExecutioner {
    fn executioner_run_cfg(&mut self, cfg: &FuncBinaryConfig) -> Result<String> {
        let output = self
            .executioner
            .run_func_with_config(cfg.clone())
            .context("invoke failed")?;
        let json = serde_json::to_string(&output).context("serialize output failed")?;
        Ok(json)
    }

    fn executioner_run_json(&mut self, json: &CxxString) -> Result<String> {
        let cfg: FuncBinaryConfig = serde_json::from_str(json.to_str()?)
            .with_context(|| "failed to parse FuncBinaryConfig JSON")?;
        self.executioner_run_cfg(&cfg)
    }

    fn executioner_compile_json(&mut self, json: &CxxString) -> Result<()> {
        let cfg: FuncBinaryConfig = serde_json::from_str(json.to_str()?)
            .with_context(|| "failed to parse FuncBinaryConfig JSON")?;
        self.executioner.compile_func_with_config(cfg.clone())?;
        Ok(())
    }
}

// ---------- KV side & C++ bridge ----------

fn new_dummy_kv() -> Box<KvBox> {
    Box::new(KvBox::new(Box::new(DummyKV::new())))
}

fn clone_kv(kv: &Box<KvBox>) -> Box<KvBox> {
    Box::new(KvBox::new(kv.inner.clone()))
}

pub struct CppKV {
    kv_interface: *mut ffi::KVInterface,
}

impl CppKV {
    pub fn new(kv_interface: *mut ffi::KVInterface) -> Self {
        Self { kv_interface }
    }
}

impl bindings::detersl::kv_api::kv::Host for CppKV {
    fn get(&mut self, key: String) -> Option<Vec<u8>> {
        let slice: &[u8] = unsafe { Pin::new_unchecked(&mut *self.kv_interface).get(&key) };
        if slice.is_empty() {
            None
        } else {
            Some(slice.to_vec())
        }
    }

    fn set(&mut self, key: String, value: Vec<u8>) {
        // Ownership transferred to C++; no copy!
        unsafe { Pin::new_unchecked(&mut *self.kv_interface).set(&key, value) }
    }

    fn delete(&mut self, key: String) -> bool {
        unsafe { Pin::new_unchecked(&mut *self.kv_interface).delete_key(&key) }
    }
}

impl KVTypeClone for CppKV {
    fn clone_to_box(&self) -> Box<dyn KVType> {
        Box::new(CppKV::new(self.kv_interface))
    }
}

impl Drop for CppKV {
    fn drop(&mut self) {
        // C++ owns the pointer
    }
}

unsafe fn new_cpp_kv(kv_interface: *mut ffi::KVInterface) -> Box<KvBox> {
    Box::new(KvBox::new(Box::new(CppKV::new(kv_interface))))
}

impl KVType for CppKV {}
