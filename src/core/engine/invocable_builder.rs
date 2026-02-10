use anyhow::Context;

use crate::{config::FuncBinaryConfig, core::{bindings, detersl_wasi::kv::KVType, engine::{detersl_engine::{DeterSLEngine, DeterSLFuncInfo}, invocable::DeterSLFuncInvocable}, execution::ExecutionState}};


pub trait DeterSLFuncInvocableBuilder {
    fn encode_func_config(&mut self, func_config: &FuncBinaryConfig);

    fn compile_func(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()>;

    fn make_instance(&mut self, engine: &mut DeterSLEngine) -> anyhow::Result<()>;

    fn build(&mut self) -> anyhow::Result<DeterSLFuncInvocable>;

    #[allow(dead_code)]
    fn reset(&mut self) -> anyhow::Result<()>;
}

pub struct DefaultFuncInvocableBuilder {
    cfg: Option<FuncBinaryConfig>,
    info: Option<DeterSLFuncInfo>,
    pre: Option<bindings::DeterslApiPre<ExecutionState>>,
    invocable: Option<DeterSLFuncInvocable>,
    make_state: Option<Box<dyn Fn(&FuncBinaryConfig, Box<dyn KVType>) -> anyhow::Result<ExecutionState> + 'static>>,
    kv: Option<Box<dyn KVType>>
}

impl DefaultFuncInvocableBuilder {
    pub fn new() -> Self {
        Self {
            cfg: None,
            info: None,
            pre: None,
            invocable: None,
            make_state: None,
            kv: None
        }
    }

    pub fn with_kv(mut self, kv: Box<dyn KVType>) -> Self {
        self.kv = Some(kv);
        self
    }

    pub fn with_state_builder<F>(mut self, f: F) -> Self
    where
        F: Fn(&FuncBinaryConfig, Box<dyn KVType>) -> anyhow::Result<ExecutionState> + 'static
    {
        self.make_state = Some(Box::new(f));
        self
    }

    pub fn clear(&mut self) {
        self.cfg = None;
        self.info = None;
        self.pre = None;
        self.invocable = None;
    }
}

pub fn build_state(func_config: &FuncBinaryConfig, kv: Box<dyn KVType>) -> anyhow::Result<ExecutionState> {
    Ok(ExecutionState::new(kv, &func_config.func_execution_policy, &func_config.func_initial_values))
}

impl DeterSLFuncInvocableBuilder for DefaultFuncInvocableBuilder {
    fn encode_func_config(&mut self, func_config: &FuncBinaryConfig) {
        self.cfg = Some(func_config.clone());

        self.info = match DeterSLFuncInfo::from_config(func_config.clone()) {
            Ok(info) => Some(info),
            Err(_) => None,
        };
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
            .context("no invocable built; call make_instance() first")
    }

    fn reset(&mut self) -> anyhow::Result<()> {
        self.clear();
        Ok(())
    }
}
