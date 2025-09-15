use std::path::PathBuf;

use crate::config::FuncBinarySource;

use anyhow::Context;

use super::component_fetcher::ComponentFetcher;

pub struct HttpFetcher;

impl HttpFetcher {
    pub fn new() -> Box<Self> {
        Box::new(Self{})
    }
}

impl ComponentFetcher for HttpFetcher {
    fn fetch(&self, source: &FuncBinarySource) -> anyhow::Result<PathBuf> {
        match source {
            FuncBinarySource::Http { url, headers } => {
                let client = reqwest::blocking::Client::new();
                let mut req = client.get(url);

                if !headers.is_empty() {
                    let mut header_map = reqwest::header::HeaderMap::new();
                    for (k, v) in headers {
                        if let Ok(name) =
                            reqwest::header::HeaderName::from_lowercase(k.trim().to_lowercase().as_bytes())
                        {
                            if let Ok(val) = reqwest::header::HeaderValue::from_str(v) {
                                header_map.insert(name, val);
                            }
                        }
                    }
                    req = req.headers(header_map);
                }

                let resp = req
                    .send()
                    .with_context(|| format!("failed to GET '{}'", url))?;

                if !resp.status().is_success() {
                    return Err(anyhow::anyhow!("download failed: {} -> HTTP {}", url, resp.status()));
                }

                let bytes = resp
                    .bytes()
                    .with_context(|| format!("failed to read response body from '{}'", url))?;

                let tmp = std::env::temp_dir();
                // A simple filename; callers can manage lifecycle of the temp file.
                let filename = format!("funcbin_{}.bin", chrono::Utc::now().timestamp_nanos());
                let dest = tmp.join(filename);
                std::fs::write(&dest, &bytes)
                    .with_context(|| format!("failed to write downloaded binary to {:?}", dest))?;
                Ok(dest)
            }
            other => Err(anyhow::anyhow!("HttpFetcher cannot handle source: {:?}", other)),
        }
    }
}
