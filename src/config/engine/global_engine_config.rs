use std::sync::OnceLock;

use anyhow::{anyhow, Result};

use crate::config::engine::engine_config::EngineConfig;

static GLOBAL_ENGINE_CONFIG: OnceLock<EngineConfig> = OnceLock::new();

pub fn init_global_engine_config(cfg: EngineConfig) -> Result<&'static EngineConfig> {
    GLOBAL_ENGINE_CONFIG
        .set(cfg)
        .map_err(|_| anyhow!("global engine config is already initialized"))?;

    Ok(GLOBAL_ENGINE_CONFIG.get().unwrap())
}

pub fn init_global_engine_config_from_file(path: &str) -> Result<&'static EngineConfig> {
    let cfg = EngineConfig::from_json_file(path)?;
    init_global_engine_config(cfg)
}

pub fn init_global_engine_config_from_json(raw: &str) -> Result<&'static EngineConfig> {
    let cfg = EngineConfig::from_json_str(raw)?;
    init_global_engine_config(cfg)
}

pub fn get_global_engine_config() -> Option<&'static EngineConfig> {
    GLOBAL_ENGINE_CONFIG.get()
}

pub fn require_global_engine_config() -> &'static EngineConfig {
    GLOBAL_ENGINE_CONFIG
        .get()
        .expect("global engine config is not initialized")
}
