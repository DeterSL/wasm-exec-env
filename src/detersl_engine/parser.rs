    use wasmtime::Val;
    use serde::Deserialize;
    use std::fs::File;
    use std::io::Read;

    use super::core::WasmModule;

    #[derive(Deserialize)]
    struct WasmModuleJson {
        module_name: String,
        module_path: String,
        module_entry_point: String,
        args: Vec<serde_json::Value>,
    }

    pub trait WasmModuleParser {
        fn parse(&self) -> anyhow::Result<WasmModule>;
    }

    pub struct WasmModuleJasonParser {
        pub json_file_path: String,
    }

    impl WasmModuleJasonParser {
        pub fn new(json_file_path: String) -> Self {
            WasmModuleJasonParser { json_file_path }
        }
    }

    impl WasmModuleParser for WasmModuleJasonParser {
        fn parse(&self) -> anyhow::Result<WasmModule> {
            let mut file = File::open(&self.json_file_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            let parsed: WasmModuleJson = serde_json::from_str(&contents)?;

            let mut vals = Vec::new();
            for v in parsed.args {
                let val = if let Some(i) = v.as_i64() {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        Val::I32(i as i32)
                    } else {
                        Val::I64(i as i64)
                    }
                } else if let Some(f) = v.as_f64() {
                    Val::F64(f as u64)
                } else if let Some(s) = v.as_str() {
                    Val::I32(0)
                } else {
                    return Err(anyhow::anyhow!("Unsupported arg type in JSON"));
                };
                vals.push(val);
            }

            Ok(WasmModule::new(
                parsed.module_name,
                parsed.module_path,
                parsed.module_entry_point,
                vals,
            ))
        }
    }
