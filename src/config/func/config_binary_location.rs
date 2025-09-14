use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FuncBinarySource {
    Fs { path: String },

    Http {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
}
