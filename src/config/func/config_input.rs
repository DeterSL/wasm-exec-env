
#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FuncInputEvent {
    Data {
        data: String
    },

    KV {
        key: String
    }
}
