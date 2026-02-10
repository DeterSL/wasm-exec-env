use std::fs::File;
use std::io::Read;

use super::config::FuncBinaryConfig;
use super::config_parser::FuncBinaryConfigParser;

pub struct FuncBinaryConfigJsonParser;

impl FuncBinaryConfigJsonParser {
    pub fn new() -> Self {
        FuncBinaryConfigJsonParser
    }
}

impl FuncBinaryConfigParser for FuncBinaryConfigJsonParser {
    fn parse_from_str(&self, config: String) -> anyhow::Result<FuncBinaryConfig> {
        let parsed: FuncBinaryConfig = serde_json::from_str(&config)?;
        Ok(parsed)
    }

    #[allow(dead_code)]
    fn parse_from_file_path(&self, path: String) -> anyhow::Result<FuncBinaryConfig> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        self.parse_from_str(contents)
    }
}

impl Default for FuncBinaryConfigJsonParser {
    fn default() -> Self {
        Self::new()
    }
}
