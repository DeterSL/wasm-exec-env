use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use wasmtime::{
    Cache, CacheConfig, Config as WasmConfig, Engine as WasmEngine,
    InstanceAllocationStrategy, PoolingAllocationConfig, Strategy,
};

use crate::{config::engine::global_engine_config::{init_global_engine_config_from_file, init_global_engine_config_from_json}, core::engine::DeterSLEngine};

const fn minus_one_i64() -> i64 {
    -1
}

const fn default_lrucache_capacity_i64() -> i64 {
    10
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct EngineConfig {
    // Wasmtime-side
    pub cache_enabled: Option<bool>,
    pub strategy: Option<WasmStrategyConfig>,
    pub memory_init_cow: Option<bool>,

    #[serde(default = "minus_one_i64")]
    pub memory_guard_size: i64,

    #[serde(default = "minus_one_i64")]
    pub memory_reservation: i64,

    pub allocation: Option<AllocationStrategyConfig>,

    // DeterSL-side
    #[serde(default = "default_lrucache_capacity_i64")]
    pub lrucache_capacity: i64,

    pub module_save_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WasmStrategyConfig {
    Cranelift,
    Winch,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AllocationStrategyConfig {
    pub kind: Option<AllocationKind>,
    pub pooling: PoolingConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationKind {
    OnDemand,
    Pooling,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct PoolingConfig {
    #[serde(default = "minus_one_i64")]
    pub total_memories: i64,

    #[serde(default = "minus_one_i64")]
    pub max_memory_size: i64,

    #[serde(default = "minus_one_i64")]
    pub total_tables: i64,

    #[serde(default = "minus_one_i64")]
    pub table_elements: i64,

    #[serde(default = "minus_one_i64")]
    pub total_core_instances: i64,

    #[serde(default = "minus_one_i64")]
    pub linear_memory_keep_resident: i64,

    #[serde(default = "minus_one_i64")]
    pub table_keep_resident: i64,

    #[serde(default = "minus_one_i64")]
    pub decommit_batch_size: i64,

    #[serde(default = "minus_one_i64")]
    pub max_unused_warm_slots: i64,
}

impl EngineConfig {
    pub fn default_module_save_path() -> String {
        String::from("./modules")
    }

    pub fn effective_module_save_path(&self) -> String {
        match &self.module_save_path {
            Some(v) if !v.trim().is_empty() => v.clone(),
            _ => Self::default_module_save_path(),
        }
    }

    pub fn effective_lrucache_capacity(&self) -> Result<usize> {
        if self.lrucache_capacity < 0 {
            bail!("lrucache_capacity must be >= 0");
        }

        usize::try_from(self.lrucache_capacity)
            .context("lrucache_capacity is out of range")
    }

    pub fn from_json_str(raw: &str) -> Result<Self> {
        if raw.trim().is_empty() {
            return Ok(Self::default());
        }

        let cfg: EngineConfig = sonic_rs::from_str(raw)
            .context("failed to parse engine config JSON string")?;

        cfg.validate()?;
        Ok(cfg)
    }

    pub fn from_json_file(path: &str) -> Result<Self> {
        if path.trim().is_empty() {
            let cfg = Self::default();
            cfg.validate()?;
            return Ok(cfg);
        }

        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read engine config file: {path}"))?;

        let cfg: EngineConfig = sonic_rs::from_str(&raw)
            .with_context(|| format!("failed to parse engine config JSON: {path}"))?;

        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<()> {
        let _ = self.effective_lrucache_capacity()?;

        validate_minus_one("memory_guard_size", self.memory_guard_size)?;
        validate_minus_one("memory_reservation", self.memory_reservation)?;

        if let Some(allocation) = &self.allocation {
            let p = &allocation.pooling;

            validate_minus_one("allocation.pooling.total_memories", p.total_memories)?;
            validate_minus_one("allocation.pooling.max_memory_size", p.max_memory_size)?;
            validate_minus_one("allocation.pooling.total_tables", p.total_tables)?;
            validate_minus_one("allocation.pooling.table_elements", p.table_elements)?;
            validate_minus_one(
                "allocation.pooling.total_core_instances",
                p.total_core_instances,
            )?;
            validate_minus_one(
                "allocation.pooling.linear_memory_keep_resident",
                p.linear_memory_keep_resident,
            )?;
            validate_minus_one(
                "allocation.pooling.table_keep_resident",
                p.table_keep_resident,
            )?;
            validate_minus_one(
                "allocation.pooling.decommit_batch_size",
                p.decommit_batch_size,
            )?;
            validate_minus_one(
                "allocation.pooling.max_unused_warm_slots",
                p.max_unused_warm_slots,
            )?;
        }

        Ok(())
    }

    pub fn apply_to_wasmtime_config(&self, engine_cfg: &mut WasmConfig) -> Result<()> {
        if matches!(self.cache_enabled, Some(true)) {
            let cache = Cache::new(CacheConfig::new())
                .context("failed to create Wasmtime cache")?;
            engine_cfg.cache(Some(cache));
        }

        if let Some(strategy) = &self.strategy {
            match strategy {
                WasmStrategyConfig::Cranelift => engine_cfg.strategy(Strategy::Cranelift),
                WasmStrategyConfig::Winch => engine_cfg.strategy(Strategy::Winch),
            };
        }

        if let Some(memory_init_cow) = self.memory_init_cow {
            engine_cfg.memory_init_cow(memory_init_cow);
        }

        if let Some(v) = parse_minus_one::<u64>("memory_guard_size", self.memory_guard_size)? {
            engine_cfg.memory_guard_size(v);
        }

        if let Some(v) = parse_minus_one::<u64>("memory_reservation", self.memory_reservation)? {
            engine_cfg.memory_reservation(v);
        }

        if let Some(allocation) = &self.allocation {
            match allocation.kind {
                Some(AllocationKind::OnDemand) | None => {}
                Some(AllocationKind::Pooling) => {
                    let p = &allocation.pooling;
                    let mut pool = PoolingAllocationConfig::new();

                    if let Some(v) = parse_minus_one::<u32>(
                        "allocation.pooling.total_memories",
                        p.total_memories,
                    )? {
                        pool.total_memories(v);
                    }

                    if let Some(v) = parse_minus_one::<usize>(
                        "allocation.pooling.max_memory_size",
                        p.max_memory_size,
                    )? {
                        pool.max_memory_size(v);
                    }

                    if let Some(v) = parse_minus_one::<u32>(
                        "allocation.pooling.total_tables",
                        p.total_tables,
                    )? {
                        pool.total_tables(v);
                    }

                    if let Some(v) = parse_minus_one::<u32>(
                        "allocation.pooling.table_elements",
                        p.table_elements,
                    )? {
                        pool.table_elements(v as usize);
                    }

                    if let Some(v) = parse_minus_one::<u32>(
                        "allocation.pooling.total_core_instances",
                        p.total_core_instances,
                    )? {
                        pool.total_core_instances(v);
                    }

                    if let Some(v) = parse_minus_one::<usize>(
                        "allocation.pooling.linear_memory_keep_resident",
                        p.linear_memory_keep_resident,
                    )? {
                        pool.linear_memory_keep_resident(v);
                    }

                    if let Some(v) = parse_minus_one::<usize>(
                        "allocation.pooling.table_keep_resident",
                        p.table_keep_resident,
                    )? {
                        pool.table_keep_resident(v);
                    }

                    if let Some(v) = parse_minus_one::<usize>(
                        "allocation.pooling.decommit_batch_size",
                        p.decommit_batch_size,
                    )? {
                        pool.decommit_batch_size(v);
                    }

                    if let Some(v) = parse_minus_one::<u32>(
                        "allocation.pooling.max_unused_warm_slots",
                        p.max_unused_warm_slots,
                    )? {
                        pool.max_unused_warm_slots(v);
                    }

                    engine_cfg.allocation_strategy(
                        InstanceAllocationStrategy::Pooling(pool),
                    );
                }
            }
        }

        Ok(())
    }

    pub fn build_detersl_engine(&self) -> Result<DeterSLEngine> {
        let mut engine_cfg = WasmConfig::new();
        self.apply_to_wasmtime_config(&mut engine_cfg)?;

        let wasmtime_engine =
            WasmEngine::new(&engine_cfg).context("failed to create Wasmtime Engine")?;

        let det_engine = DeterSLEngine::new(wasmtime_engine)
            .context("failed to create DeterSLEngine")?;

        Ok(det_engine)
    }
}

fn validate_minus_one(field: &str, raw: i64) -> Result<()> {
    if raw < -1 {
        bail!("{field} must be >= 0 or -1");
    }
    Ok(())
}

fn parse_minus_one<T>(field: &str, raw: i64) -> Result<Option<T>>
where
    T: TryFrom<i64>,
    <T as TryFrom<i64>>::Error: std::error::Error + Send + Sync + 'static,
{
    match raw {
        -1 => Ok(None),
        v if v < -1 => bail!("{field} must be >= 0 or -1"),
        v => Ok(Some(
            T::try_from(v).with_context(|| format!("{field} is out of range"))?,
        )),
    }
}

pub fn new_detersl_engine_from_config_path(config_path: &str) -> Result<DeterSLEngine> {
    let cfg = init_global_engine_config_from_file(config_path)?;
    cfg.build_detersl_engine()
}

pub fn new_detersl_engine_from_json(config_json: &str) -> Result<DeterSLEngine> {
    let cfg = init_global_engine_config_from_json(config_json)?;
    cfg.build_detersl_engine()
}
