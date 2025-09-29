#[derive(serde::Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FuncOutputEvent {
    Default,
    KV {
        key: String
    }
}
