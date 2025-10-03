#[derive(sonic_rs::Deserialize, Clone)]
pub struct FuncInitValue {
    #[serde(default)]
    pub init_clock: u64,

    #[serde(default)]
    // TODO: in execution state apparenly we need u8 in some seeds
    pub random_seed: u128
}
