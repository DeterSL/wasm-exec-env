mod detersl_engine;

use std::{env, process};
use detersl_engine::core::core::*;
use detersl_engine::parser::parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <wasm_file> <config_json>", args[0]);
        process::exit(1);
    }

    let wasm_path = &args[1];
    let config_path = &args[2];

    // Parse the JSON config to create a WasmModule
    let parser = WasmModuleJasonParser::new(config_path.to_string());
    let mut module = match parser.parse() {
        Ok(mut m) => {
            // Overwrite module_path with CLI wasm_path argument
            m.module_path = wasm_path.to_string();
            m
        },
        Err(e) => {
            eprintln!("Error parsing config '{}': {}", config_path, e);
            process::exit(1);
        }
    };

    let engine = DeterSLEngine::new();
    match engine.run_module(&module) {
        Ok(results) => println!("Wasm call results: {:?}", results),
        Err(e) => {
            eprintln!("Error running Wasm module '{}': {}", wasm_path, e);
            process::exit(1);
        }
    }
}
