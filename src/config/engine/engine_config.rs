use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use wasmtime::{
    Cache, CacheConfig, Config as WasmConfig, Engine as WasmEngine,
    InstanceAllocationStrategy, PoolingAllocationConfig, Strategy,
};

use crate::core::engine::{DeterSLEngine, DeterSLEngineConfig};

const fn minus_one_i64() -> i64 {
    -1
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct EngineFileConfig {
    // Wasmtime-side
    cache_enabled: Option<bool>,
    strategy: Option<WasmStrategyConfig>,
    memory_init_cow: Option<bool>,

    #[serde(default = "minus_one_i64")]
    memory_guard_size: i64,

    #[serde(default = "minus_one_i64")]
    memory_reservation: i64,

    allocation: Option<AllocationStrategyConfig>,

    // DeterSL-side
    #[serde(default = "minus_one_i64")]
    cache_size: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WasmStrategyConfig {
    Cranelift,
    Winch,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct AllocationStrategyConfig {
    kind: Option<AllocationKind>,
    pooling: PoolingConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AllocationKind {
    OnDemand,
    Pooling,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct PoolingConfig {
    #[serde(default = "minus_one_i64")]
    total_memories: i64,

    #[serde(default = "minus_one_i64")]
    max_memory_size: i64,

    #[serde(default = "minus_one_i64")]
    total_tables: i64,

    #[serde(default = "minus_one_i64")]
    table_elements: i64,

    #[serde(default = "minus_one_i64")]
    total_core_instances: i64,

    #[serde(default = "minus_one_i64")]
    linear_memory_keep_resident: i64,

    #[serde(default = "minus_one_i64")]
    table_keep_resident: i64,

    #[serde(default = "minus_one_i64")]
    decommit_batch_size: i64,

    #[serde(default = "minus_one_i64")]
    max_unused_warm_slots: i64,
}

fn read_engine_file_config(path: &str) -> Result<EngineFileConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read engine config file: {path}"))?;

    let cfg: EngineFileConfig = sonic_rs::from_str(&raw)
        .with_context(|| format!("failed to parse engine config JSON: {path}"))?;

    Ok(cfg)
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

fn apply_engine_file_config(engine_cfg: &mut WasmConfig, cfg: &EngineFileConfig) -> Result<()> {
    if matches!(cfg.cache_enabled, Some(true)) {
        let cache = Cache::new(CacheConfig::new())
            .context("failed to create Wasmtime cache")?;
        engine_cfg.cache(Some(cache));
    }

    if let Some(strategy) = &cfg.strategy {
        match strategy {
            WasmStrategyConfig::Cranelift => engine_cfg.strategy(Strategy::Cranelift),
            WasmStrategyConfig::Winch => engine_cfg.strategy(Strategy::Winch),
        };
    }

    if let Some(memory_init_cow) = cfg.memory_init_cow {
        engine_cfg.memory_init_cow(memory_init_cow);
    }

    if let Some(v) = parse_minus_one("memory_guard_size", cfg.memory_guard_size)? {
        engine_cfg.memory_guard_size(v);
    }

    if let Some(v) = parse_minus_one("memory_reservation", cfg.memory_reservation)? {
        engine_cfg.memory_reservation(v);
    }

    if let Some(allocation) = &cfg.allocation {
        match allocation.kind {
            Some(AllocationKind::OnDemand) | None => {
                // Preserve Wasmtime default allocation strategy.
            }
            Some(AllocationKind::Pooling) => {
                let p = &allocation.pooling;
                let mut pool = PoolingAllocationConfig::new();

                if let Some(v) = parse_minus_one("allocation.pooling.total_memories", p.total_memories)? {
                    pool.total_memories(v);
                }
                if let Some(v) = parse_minus_one("allocation.pooling.max_memory_size", p.max_memory_size)? {
                    pool.max_memory_size(v);
                }
                if let Some(v) = parse_minus_one("allocation.pooling.total_tables", p.total_tables)? {
                    pool.total_tables(v);
                }
                if let Some(v) = parse_minus_one("allocation.pooling.table_elements", p.table_elements)? {
                    pool.table_elements(v);
                }
                if let Some(v) = parse_minus_one("allocation.pooling.total_core_instances", p.total_core_instances)? {
                    pool.total_core_instances(v);
                }
                if let Some(v) = parse_minus_one(
                    "allocation.pooling.linear_memory_keep_resident",
                    p.linear_memory_keep_resident,
                )? {
                    pool.linear_memory_keep_resident(v);
                }
                if let Some(v) = parse_minus_one(
                    "allocation.pooling.table_keep_resident",
                    p.table_keep_resident,
                )? {
                    pool.table_keep_resident(v);
                }
                if let Some(v) = parse_minus_one(
                    "allocation.pooling.decommit_batch_size",
                    p.decommit_batch_size,
                )? {
                    pool.decommit_batch_size(v);
                }
                if let Some(v) = parse_minus_one(
                    "allocation.pooling.max_unused_warm_slots",
                    p.max_unused_warm_slots,
                )? {
                    pool.max_unused_warm_slots(v);
                }

                engine_cfg.allocation_strategy(InstanceAllocationStrategy::Pooling(pool));
            }
        }
    }

    Ok(())
}

pub fn new_detersl_engine_from_config_path(config_path: &str) -> Result<DeterSLEngine> {
    let cfg = if config_path.trim().is_empty() {
        EngineFileConfig::default()
    } else {
        read_engine_file_config(config_path)?
    };

    let mut engine_cfg = WasmConfig::new();
    apply_engine_file_config(&mut engine_cfg, &cfg)?;

    let wasmtime_engine =
        WasmEngine::new(&engine_cfg).context("failed to create Wasmtime Engine")?;

    let mut det_cfg = DeterSLEngineConfig::default();
    if let Some(cache_size) = parse_minus_one("cache_size", cfg.cache_size)? {
        det_cfg = det_cfg.with_cache_capacity(cache_size);
    }

    let det_engine = DeterSLEngine::new(wasmtime_engine, det_cfg)
        .context("failed to create DeterSLEngine")?;

    Ok(det_engine)
}
