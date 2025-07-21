mod engine;

use std::{env, process};
use engine::DeterSLEngine;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <wasm_file>", args[0]);
        process::exit(1);
    }
    let wasm_path = &args[1];
    let mut engine = DeterSLEngine::new();
    if let Err(e) = engine.run(wasm_path) {
        eprintln!("Error running Wasm module '{}': {}", wasm_path, e);
        process::exit(1);
    }
}
