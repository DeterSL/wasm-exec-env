use std::time::Instant;
use crate::{
    config::FuncBinaryConfig,
    core::{
        detersl_wasi::kv::KVType,
        engine::{build_state, DefaultFuncInvocableBuilder, DeterSLEngine, DeterSLFuncInvocableBuilder},
        types,
    },
};

const REPORT_EVERY: u32 = 1000;

pub struct DeterSLExecutioner {
    engine: DeterSLEngine,
    invocable_builder: Option<Box<dyn DeterSLFuncInvocableBuilder>>,
    kv: Option<Box<dyn KVType>>,
    cnt: u32,

    // per-step totals (since start)
    total_encode_us: u128,
    total_compile_us: u128,
    total_make_instance_us: u128,
    total_build_us: u128,
    total_invoke_us: u128,

    // per-step rolling window totals (last REPORT_EVERY calls)
    window_encode_us: u128,
    window_compile_us: u128,
    window_make_instance_us: u128,
    window_build_us: u128,
    window_invoke_us: u128,
}

impl DeterSLExecutioner {
    pub fn new(engine: DeterSLEngine) -> Self {
        Self {
            engine,
            invocable_builder: None,
            kv: None,
            cnt: 0,

            total_encode_us: 0,
            total_compile_us: 0,
            total_make_instance_us: 0,
            total_build_us: 0,
            total_invoke_us: 0,

            window_encode_us: 0,
            window_compile_us: 0,
            window_make_instance_us: 0,
            window_build_us: 0,
            window_invoke_us: 0,
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

        // encode_func_config
        //let t0 = Instant::now();
        invocable_builder.encode_func_config(&config);
        //let encode_us = t0.elapsed().as_micros();

        // compile_func
        //let t1 = Instant::now();
        invocable_builder.compile_func(&mut self.engine)?;
        //let compile_us = t1.elapsed().as_micros();

        // make_instance
        //let t2 = Instant::now();
        invocable_builder.make_instance(&mut self.engine)?;
        //let make_instance_us = t2.elapsed().as_micros();

        // build
/*        let t3 = Instant::now();*/
        /*let mut invocable = invocable_builder.build()?;*/
        /*let build_us = t3.elapsed().as_micros();*/

        /*// invoke*/
        /*let t4 = Instant::now();*/
        /*let output = invocable.invoke(config.func_input_event.into())?;*/
        /*let invoke_us = t4.elapsed().as_micros();*/

        //let t4 = Instant::now();
        let output = invocable_builder.invoke_cached(config.func_input_event.into())?;
        //let invoke_us = t4.elapsed().as_micros();

        // counters
        /*self.cnt += 1;*/

        /*// accumulate totals*/
        /*self.total_encode_us += encode_us;*/
        /*self.total_compile_us += compile_us;*/
        /*self.total_make_instance_us += make_instance_us;*/
        /*self.total_invoke_us += invoke_us;*/

        /*// accumulate window*/
        /*self.window_encode_us += encode_us;*/
        /*self.window_compile_us += compile_us;*/
        /*self.window_make_instance_us += make_instance_us;*/
        /*self.window_invoke_us += invoke_us;*/

        /*if self.cnt % REPORT_EVERY == 0 {*/
            /*let n_total = self.cnt as u128;*/
            /*let n_window = REPORT_EVERY as u128;*/

            /*println!("==== Timing report ({} calls) ====", self.cnt);*/
            /*println!(*/
                /*"encode_func_config: last={} µs | avg(total)={} µs | avg(last {})={} µs",*/
                /*encode_us,*/
                /*self.total_encode_us / n_total,*/
                /*REPORT_EVERY,*/
                /*self.window_encode_us / n_window*/
            /*);*/
            /*println!(*/
                /*"compile_func:       last={} µs | avg(total)={} µs | avg(last {})={} µs",*/
                /*compile_us,*/
                /*self.total_compile_us / n_total,*/
                /*REPORT_EVERY,*/
                /*self.window_compile_us / n_window*/
            /*);*/
            /*println!(*/
                /*"make_instance:      last={} µs | avg(total)={} µs | avg(last {})={} µs",*/
                /*make_instance_us,*/
                /*self.total_make_instance_us / n_total,*/
                /*REPORT_EVERY,*/
                /*self.window_make_instance_us / n_window*/
            /*);*/
            /*println!(*/
                /*"invoke:             last={} µs | avg(total)={} µs | avg(last {})={} µs",*/
                /*invoke_us,*/
                /*self.total_invoke_us / n_total,*/
                /*REPORT_EVERY,*/
                /*self.window_invoke_us / n_window*/
            /*);*/

            /*// reset window*/
            /*self.window_encode_us = 0;*/
            /*self.window_compile_us = 0;*/
            /*self.window_make_instance_us = 0;*/
            /*self.window_build_us = 0;*/
            /*self.window_invoke_us = 0;*/
        /*}*/

        Ok(output)
    }

    #[allow(dead_code)] // This function is used in ffi
    pub fn compile_func_with_config(&mut self, config: FuncBinaryConfig) -> anyhow::Result<()> {
        let invocable_builder = self
            .invocable_builder
            .as_mut()
            .expect("invocable builder must be set (call with_kv first)");
        invocable_builder.encode_func_config(&config);
        invocable_builder.compile_func(&mut self.engine)?;
        Ok(())
    }
}
