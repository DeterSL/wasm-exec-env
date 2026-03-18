use std::collections::HashMap;

use anyhow::{anyhow, Context};

use crate::{
    config::FuncBinaryConfig,
    core::{
        bindings,
        detersl_wasi::kv::KVType,
        engine::{
            detersl_engine::{DeterSLEngine, DeterSLFuncInfo},
            invocable::DeterSLFuncInvocable,
        },
        execution::ExecutionState,
        types,
    },
};

pub trait DeterSLFuncInvocableBuilder {
    fn encode_func_config(&mut self, func_config: &FuncBinaryConfig);
    fn compile_func(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()>;
    fn make_instance(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()>;

    // keep old API if you still use it elsewhere
    fn build(&mut self) -> anyhow::Result<DeterSLFuncInvocable>;

    // NEW: direct invoke from cached instance
    fn invoke_cached(&mut self, input: types::Event) -> anyhow::Result<types::Output>;

    #[allow(dead_code)]
    fn reset(&mut self) -> anyhow::Result<()>;
}

pub struct DefaultFuncInvocableBuilder {
    cfg: Option<FuncBinaryConfig>,
    info: Option<DeterSLFuncInfo>,
    pre: Option<bindings::DeterslApiPre<ExecutionState>>,
    invocable: Option<DeterSLFuncInvocable>,
    make_state:
        Option<Box<dyn Fn(&FuncBinaryConfig, Box<dyn KVType>) -> anyhow::Result<ExecutionState> + 'static>>,
    kv: Option<Box<dyn KVType>>,

    // cache FULL invocable (instance+store), keyed by function config
    invocable_cache: HashMap<String, DeterSLFuncInvocable>,
}

impl DefaultFuncInvocableBuilder {
    pub fn new() -> Self {
        Self {
            cfg: None,
            info: None,
            pre: None,
            invocable: None,
            make_state: None,
            kv: None,
            invocable_cache: HashMap::new(),
        }
    }

    pub fn with_kv(mut self, kv: Box<dyn KVType>) -> Self {
        self.kv = Some(kv);
        self
    }

    pub fn with_state_builder<F>(mut self, f: F) -> Self
    where
        F: Fn(&FuncBinaryConfig, Box<dyn KVType>) -> anyhow::Result<ExecutionState> + 'static,
    {
        self.make_state = Some(Box::new(f));
        self
    }

    fn cache_key(cfg: &FuncBinaryConfig, info: &DeterSLFuncInfo) -> String {
        cfg.func_binary_hash.clone()
    }

    pub fn clear(&mut self) {
        self.cfg = None;
        self.info = None;
        self.pre = None;
        self.invocable = None;
    }

    #[allow(dead_code)]
    pub fn clear_cache(&mut self) {
        self.invocable_cache.clear();
    }
}

pub fn build_state(func_config: &FuncBinaryConfig, kv: Box<dyn KVType>) -> anyhow::Result<ExecutionState> {
    Ok(ExecutionState::new(
        kv,
        &func_config.func_execution_policy,
        &func_config.func_initial_values,
    ))
}

impl DeterSLFuncInvocableBuilder for DefaultFuncInvocableBuilder {
    fn encode_func_config(&mut self, func_config: &FuncBinaryConfig) {
        self.cfg = Some(func_config.clone());
        self.info = DeterSLFuncInfo::from_config(func_config.clone()).ok();
    }

    fn compile_func(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()> {
        let info = self
            .info
            .as_ref()
            .context("call encode_func_config() first (no func info present)")?;

        let pre = engine
            .get_instance_from(info)
            .context("failed to get pre-instantiated binding from engine")?;

        self.pre = Some(pre);
        Ok(())
    }

    fn make_instance(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()> {
        let cfg = self
            .cfg
            .as_ref()
            .context("call encode_func_config() first (no config present)")?;
        let info = self
            .info
            .as_ref()
            .context("call encode_func_config() first (no func info present)")?;
        let key = Self::cache_key(cfg, info);

        // cache hit -> reuse full instance/store
        if let Some(cached) = self.invocable_cache.remove(&key) {
            self.invocable = Some(cached);
            return Ok(());
        }

        // cache miss -> build new full invocable
        let pre = self
            .pre
            .as_ref()
            .cloned()
            .context("call compile_func() first (no pre-instance present)")?;

        let make_state = self
            .make_state
            .as_ref()
            .context("no state builder configured; call with_state_builder() first")?;

        let kv = self.kv.as_mut().cloned().context("failed to get kv")?;
        let state = make_state(cfg, kv).context("failed to create ExecutionState from config")?;

        let mut invocable = DeterSLFuncInvocable::new();
        invocable
            .make_store(engine, state)
            .context("failed to create wasmtime Store")?;
        invocable
            .fill_instance(pre)
            .context("failed to instantiate typed world into the Store")?;

        self.invocable = Some(invocable);
        Ok(())
    }

    fn build(&mut self) -> anyhow::Result<DeterSLFuncInvocable> {
        self.invocable
            .take()
            .ok_or_else(|| anyhow!("no invocable built; call make_instance() first"))
    }

    fn invoke_cached(&mut self, input: types::Event) -> anyhow::Result<types::Output> {
        let cfg = self
            .cfg
            .as_ref()
            .context("call encode_func_config() first (no config present)")?;
        let info = self
            .info
            .as_ref()
            .context("call encode_func_config() first (no func info present)")?;
        let key = Self::cache_key(cfg, info);

        let invocable = self
            .invocable
            .as_mut()
            .ok_or_else(|| anyhow!("no invocable ready; call make_instance() first"))?;

        // requires invoke(&mut self, input) in DeterSLFuncInvocable
        let out = invocable.invoke(input)?;

        // keep it cached for next call
        let inv = self.invocable.take().unwrap();
        self.invocable_cache.insert(key, inv);

        Ok(out)
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.clear();
        Ok(())
    }
}
