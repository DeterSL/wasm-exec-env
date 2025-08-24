use crate::{bindings::exports::detersl::api::func_handler, config::func_config::{FuncBinaryConfig, FuncInputEvent}};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub data: String
}

impl Event {
    pub fn into_binding(self) -> func_handler::Event {
        func_handler::Event { data: self.data}
    }
}

impl From<FuncInputEvent> for Event {
    fn from(value: FuncInputEvent) -> Self {
        Self { data: value.data }
    }
}
