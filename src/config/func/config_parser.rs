use super::config::FuncBinaryConfig;

pub trait FuncBinaryConfigParser {
    fn parse(&self) -> anyhow::Result<FuncBinaryConfig>;
}
