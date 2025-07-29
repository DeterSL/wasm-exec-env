mod core;
mod config;

use std::{env, process, time::Instant};
use core::*;
use config::parser::*;
use wasmtime_wasi::{
    p2::{WasiCtxBuilder, WasiCtx, IoView, WasiView},
    ResourceTable,
};

wasmtime::component::bindgen!({
    path: "/home/sod/research/serverless/source-code/deterSL_runtime/wasm-examples/json-processor/json_processor.wit",
    world: "json-processor-world",
});

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl IoView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}
impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

impl MyState {
    fn new() -> MyState {
        let mut wasi = WasiCtxBuilder::new();

        MyState {
            ctx: wasi.build(),
            table: ResourceTable::new(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let overall_timer = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <config_json>", args[0]);
        process::exit(1);
    }
    let config_path = &args[1];

    // 1. Parse the JSON config to create a WasmModule
    let start_parse = Instant::now();
    let parser = WasmBinaryJsonParser::new(config_path.to_string());
    let mut module = match parser.parse() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing config '{}': {}", config_path, e);
            process::exit(1);
        }
    };
    println!("Config parsed in: {:.2?}", start_parse.elapsed());

    // 2. Initialize engine and linker
    let start_engine = Instant::now();
    let engine = engine::DeterSLEngine::new();
    let mut detersl_linker_builder = builder::DeterSLComponentLinkerBuilder::new(&engine); 
    detersl_linker_builder.add_wasi();
    println!("Engine and linker initialized in: {:.2?}", start_engine.elapsed());

    // 3. Create instance builder and store with state
    let start_store = Instant::now();
    let mut detersl_instance_builder = builder::DeterSLComponentInstanceBuilder::new(&engine, detersl_linker_builder);
    detersl_instance_builder.create_store_with(MyState::new());
    println!("Store created in: {:.2?}", start_store.elapsed());

    // 4. Load component from config
    let start_load = Instant::now();
    detersl_instance_builder.load_component_from_config(&module);
    println!("Component loaded from config in: {:.2?}", start_load.elapsed());

    // 5. Instantiate the component using bindgen
    let start_instantiate = Instant::now();
    let bindgen_world = detersl_instance_builder.instantiate_instance(|store, component, linker| {
        let world = JsonProcessorWorld::instantiate(store, component, &linker)?;
        Ok(world)
    })?;
    println!("Component instantiated in: {:.2?}", start_instantiate.elapsed());

    // 6. Call process function and measure the duration
    let start_call = Instant::now();
    let input = r#"{"number":42}"#;
    let output = bindgen_world.http_reader_json_jsonprocess().call_process(detersl_instance_builder.take_store(), input)?;
    println!("Processing call executed in: {:.2?}", start_call.elapsed());

    println!("Processed JSON: {}", output);
    println!("Total execution time: {:.2?}", overall_timer.elapsed());
    Ok(())
}
