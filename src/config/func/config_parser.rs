use super::config::FuncBinaryConfig;

pub trait FuncBinaryConfigParser {
    fn parse_from_str(&self, config: String) -> anyhow::Result<FuncBinaryConfig>;
    
    fn parse_from_file_path(&self, path: String) -> anyhow::Result<FuncBinaryConfig>;
}
