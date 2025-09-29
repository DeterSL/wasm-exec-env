use std::path::PathBuf;

use crate::config::FuncBinarySource;

use super::{fs_fetcher::FsFetcher, http_fetcher::HttpFetcher};

pub trait ComponentFetcher {
    fn fetch(&self, source: &FuncBinarySource) -> anyhow::Result<PathBuf>;
}

pub fn get_component_fetcher_for_source(source: &FuncBinarySource) -> anyhow::Result<Box<dyn ComponentFetcher>> {
    match source {
       FuncBinarySource::Fs { path: _path } => Ok(FsFetcher::new()), 
       FuncBinarySource::Http { url: _url, headers: _headers } => Ok(HttpFetcher::new())
    }
}

pub type OneTimeFetcherFn = Box<dyn Fn() -> anyhow::Result<PathBuf>>;
pub fn get_one_time_component_fetcher_for_source(source: &FuncBinarySource) -> anyhow::Result<OneTimeFetcherFn> {
    let fetcher = get_component_fetcher_for_source(source)?;
    let owned_source = source.clone();
    Ok(Box::new(Box::new(move || fetcher.fetch(&owned_source))))
}
