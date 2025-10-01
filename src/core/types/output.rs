use crate::core::bindings::exports::detersl::api::func_handler;

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

