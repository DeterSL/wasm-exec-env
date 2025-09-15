use std::{path::{PathBuf, Path}, fs};

use anyhow::Context;

use crate::config::FuncBinarySource;

use super::component_fetcher::ComponentFetcher;

pub struct FsFetcher;

impl FsFetcher {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }
}

impl ComponentFetcher for FsFetcher {
    fn fetch(&self, source: &FuncBinarySource) -> anyhow::Result<PathBuf> {
        match source {
            FuncBinarySource::Fs { path } => {
                let p = Path::new(path);
                if !p.exists() {
                    Err(anyhow::anyhow!("local binary path does not exist: {}", path))
                } else {
                    let canonical = fs::canonicalize(p)
                        .with_context(|| format!("failed to canonicalize path '{}'", path))?;
                    Ok(canonical)
                }
            }
            other => Err(anyhow::anyhow!("FsFetcher cannot handle source: {:?}", other)),
        }
    }
}
