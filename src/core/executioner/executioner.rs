use crate::{
    config::FuncBinaryConfig,
    core::{
        detersl_wasi::kv::KVType,
        engine::{build_state, DefaultFuncInvocableBuilder, DeterSLEngine, DeterSLFuncInvocableBuilder},
        types,
    },
};

pub struct DeterSLExecutioner {
    engine: DeterSLEngine,
    invocable_builder: Option<Box<dyn DeterSLFuncInvocableBuilder>>,
    kv: Option<Box<dyn KVType>>,
}

impl DeterSLExecutioner {
    pub fn new(engine: DeterSLEngine) -> Self {
        Self {
            engine,
            invocable_builder: None,
            kv: None,
        }
    }

    pub fn with_kv(mut self, kv: Box<dyn KVType>) -> Self {
        self.kv = Some(kv);
        let kv = self.kv.clone();
        self.invocable_builder = Some(Box::new(
            DefaultFuncInvocableBuilder::new()
                .with_state_builder(build_state)
                .with_kv(kv.unwrap()),
        ));

        self
    }

    pub fn run_func_with_config(&mut self, config: FuncBinaryConfig) -> anyhow::Result<types::Output> {
        let invocable_builder = self
            .invocable_builder
            .as_mut()
            .expect("invocable builder must be set (call with_kv first)");

        invocable_builder.encode_func_config(&config);
        invocable_builder.compile_func(&mut self.engine)?;
        invocable_builder.make_instance(&mut self.engine)?;

        if config.fast_execution {
            match invocable_builder.invoke_cached(config.func_input_event.into()) {
                Ok(out) => Ok(out),
                Err(err) => {
                    eprintln!("invocable_builder.invoke_cached failed:\n{:#}", err);
                    Err(err)
                }
            }
        } else {
            let mut invocable = invocable_builder.build()?;
            match invocable.invoke(config.func_input_event.into()) {
                Ok(out) => Ok(out),
                Err(err) => {
                    eprintln!("invocable.invoke failed:\n{:#}", err);
                    Err(err)
                }
            }
        }
    }

    #[allow(dead_code)] // This funciton is used in ffi
    pub fn compile_func_with_config(&mut self, config: FuncBinaryConfig) -> anyhow::Result<()> {
        let invocable_builder = self.invocable_builder
            .as_mut()
            .expect("invocable builder must be set (call with_kv first)");
        invocable_builder.encode_func_config(&config);
        match invocable_builder.compile_func(&mut self.engine) {
            Ok(_) => Ok(()),
            Err(err) => {
                eprintln!("compile failed:\n{:#}", err);
                Err(err)
            }
        }
    }
}
