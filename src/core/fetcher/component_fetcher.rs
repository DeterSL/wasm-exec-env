use std::path::PathBuf;

use crate::config::FuncBinarySource;

use super::{fs_fetcher::FsFetcher, http_fetcher::HttpFetcher};

pub trait ComponentFetcher {
    fn fetch(&self, source: &FuncBinarySource) -> anyhow::Result<PathBuf>;
}

pub fn get_component_fetcher_for_source(source: &FuncBinarySource) -> anyhow::Result<Box<dyn ComponentFetcher>> {
    match source {
       FuncBinarySource::Fs { _path } => Ok(FsFetcher::new()), 
       FuncBinarySource::Http { url, headers } => Ok(HttpFetcher::new()),
       _ => Err(anyhow::anyhow!("the provided source is invalid")) 
    }
}
