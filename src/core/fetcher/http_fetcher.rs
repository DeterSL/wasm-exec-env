use std::path::PathBuf;

use crate::config::FuncBinarySource;

use anyhow::Context;

use super::component_fetcher::ComponentFetcher;

pub struct HttpFetcher {
    pub path_to_save: String,
}

impl HttpFetcher {
    pub fn new(path: String) -> Box<Self> {
        Box::new(Self {
            path_to_save: path,
        })
    }

    fn filename_from_content_disposition(
        headers: &reqwest::header::HeaderMap,
    ) -> Option<String> {
        let value = headers.get(reqwest::header::CONTENT_DISPOSITION)?;
        let value = value.to_str().ok()?;

        for part in value.split(';') {
            let part = part.trim();

            if let Some(filename) = part.strip_prefix("filename=") {
                let filename = filename.trim_matches('"').trim();
                if !filename.is_empty() {
                    return Some(filename.to_string());
                }
            }
        }

        None
    }

    fn filename_from_url(url: &str) -> Option<String> {
        let parsed = reqwest::Url::parse(url).ok()?;
        let last = parsed
            .path_segments()?
            .filter(|s| !s.is_empty())
            .next_back()?;

        if last.is_empty() {
            None
        } else {
            Some(last.to_string())
        }
    }

    fn infer_filename(
        &self,
        url: &str,
        headers: &reqwest::header::HeaderMap,
    ) -> String {
        Self::filename_from_content_disposition(headers)
            .or_else(|| Self::filename_from_url(url))
            .unwrap_or_else(|| {
                format!(
                    "downloaded_{}.wasm",
                    chrono::Utc::now().timestamp_millis()
                )
            })
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
                        if let Ok(name) = reqwest::header::HeaderName::from_lowercase(
                            k.trim().to_lowercase().as_bytes(),
                        ) {
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
                    return Err(anyhow::anyhow!(
                        "download failed: {} -> HTTP {}",
                        url,
                        resp.status()
                    ));
                }

                let file_name = self.infer_filename(url, resp.headers());

                let bytes = resp
                    .bytes()
                    .with_context(|| format!("failed to read response body from '{}'", url))?;

                let dir = PathBuf::from(&self.path_to_save);

                if !dir.exists() {
                    std::fs::create_dir_all(&dir)
                        .with_context(|| format!("failed to create directory {:?}", dir))?;
                }

                let dest = dir.join(file_name);

                std::fs::write(&dest, &bytes)
                    .with_context(|| format!("failed to write downloaded binary to {:?}", dest))?;

                Ok(dest)
            }
            other => Err(anyhow::anyhow!(
                "HttpFetcher cannot handle source: {:?}",
                other
            )),
        }
    }
}
