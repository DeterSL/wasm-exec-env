#[derive(serde::Deserialize)]
pub struct FuncInitValue {
    #[serde(default)]
    pub init_clock: u64,

    #[serde(default)]
    pub random_seed: u128
}
