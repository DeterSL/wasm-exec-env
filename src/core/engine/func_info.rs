use std::path::PathBuf;

use crate::{config::FuncBinaryConfig, core::fetcher::{get_one_time_component_fetcher_for_source, OneTimeFetcherFn}};

/// Function metadata for fetching and managing compilation
pub struct DeterSLFuncInfo {
    pub func_hash: String,
    pub func_fetcher: OneTimeFetcherFn,
}

impl DeterSLFuncInfo {
    /// Create function metadata from a configuration.
    pub fn from_config(config: FuncBinaryConfig) -> anyhow::Result<Self> {
        let component_fetcher =
            get_one_time_component_fetcher_for_source(&config.func_binary_source)?;
        Ok(Self {
            func_hash: config.func_binary_hash,
            func_fetcher: component_fetcher,
        })
    }

    /// Fetch a Wasm component for this function.
    pub fn fetch(&self) -> anyhow::Result<PathBuf> {
        (self.func_fetcher)()
    }

    // Encode the func info and generate a unique id base on the function info
    pub fn encode_func_info(&self) -> String {
        return self.func_hash.clone()
    }
}

