use super::config::FuncBinaryConfig;

pub trait FuncBinaryConfigJsonParser {
    fn parse(&self) -> anyhow::Result<FuncBinaryConfig>;
}
