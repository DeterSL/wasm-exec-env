#[derive(sonic_rs::Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FuncOutputEvent {

    // Default means the function just returns the output object.
    Default,
}
