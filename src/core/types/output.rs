use crate::core::bindings::exports::detersl::api::func_handler;
use anyhow::Context;
use serde_json;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Output {
    pub data: String
}

impl From<func_handler::Output> for Output {
    fn from(value: func_handler::Output) -> Self {
        Self {
            data: value.data
        }
    }
}

#[allow(dead_code)]
impl Output {
    pub fn to_json(&self) -> anyhow::Result<String> {
        return serde_json::to_string(self).context("serialize output failed");
    }
}

