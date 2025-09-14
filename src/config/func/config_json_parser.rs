use std::fs::File;
use std::io::Read;

use super::config::FuncBinaryConfig;
use super::config_parser::FuncBinaryConfigParser;

pub struct FuncBinaryConfigJsonParser {
    pub json_file_path: String,
}

impl FuncBinaryConfigJsonParser {
    pub fn new(json_file_path: String) -> Self {
        FuncBinaryConfigJsonParser { json_file_path }
    }
}

impl FuncBinaryConfigParser for FuncBinaryConfigJsonParser {
    fn parse(&self) -> anyhow::Result<FuncBinaryConfig> {
        let mut file = File::open(&self.json_file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let parsed: FuncBinaryConfig = serde_json::from_str(&contents)?;
        return Ok(parsed);
    }
}
