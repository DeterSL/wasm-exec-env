use serde::Deserialize;
use std::fs::File;
use std::io::Read;

pub struct WasmBinaryConfig {
    pub binary_name: String,
    pub binary_path: String,
    pub binary_entry_point: String,
    // Optional WIT file path that defines the component’s interface.
    pub wit_path: Option<String>,
}

impl WasmBinaryConfig {
    pub fn new(
        binary_name: String, 
        binary_path: String, 
        binary_entry_point: String,
        wit_path: Option<String>,
    ) -> Self {
        WasmBinaryConfig { 
            binary_name,
            binary_path,
            binary_entry_point,
            wit_path,
        }
    }
}

#[derive(Deserialize)]
struct WasmBinaryConfigJson {
    module_name: String,
    module_path: String,
    module_entry_point: String,
    // Optionally, a path to the .wit file for the component model.
    wit_file_path: Option<String>,
}

pub trait WasmBinaryConfigParser {
    fn parse(&self) -> anyhow::Result<WasmBinaryConfig>;
}

pub struct WasmBinaryJsonParser {
    pub json_file_path: String,
}

impl WasmBinaryJsonParser {
    pub fn new(json_file_path: String) -> Self {
        WasmBinaryJsonParser { json_file_path }
    }
}

impl WasmBinaryConfigParser for WasmBinaryJsonParser {
    fn parse(&self) -> anyhow::Result<WasmBinaryConfig> {
        let mut file = File::open(&self.json_file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let parsed: WasmBinaryConfigJson = serde_json::from_str(&contents)?;

        Ok(WasmBinaryConfig::new(
            parsed.module_name,
            parsed.module_path,
            parsed.module_entry_point,
            parsed.wit_file_path,
        ))
    }
}
