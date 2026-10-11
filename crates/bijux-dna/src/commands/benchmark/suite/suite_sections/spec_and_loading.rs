use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    ColdWarmContract, DecisionTraceRow, EnvironmentSnapshot, FairnessContract,
    MetricsArtifactRow, PostconditionRow, RunRecordRow, ScientificDefaultsStatus,
    SuiteRunManifest, ToolInvocationRow, BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use sha2::Digest as _;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteSpec {
    pub schema_version: String,
    pub suite_id: String,
    pub corpus: String,
    pub stages: Vec<SuiteStage>,
    #[serde(default = "default_repetitions")]
    pub repetitions: u32,
    #[serde(default)]
    pub resource_hints: Option<ResourceHints>,
    #[serde(default)]
    pub fairness: Option<FairnessSpec>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuiteStage {
    pub stage: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceHints {
    pub threads: u32,
    pub mem_gb: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FairnessSpec {
    pub threads: u32,
    pub mem_gb: u32,
    pub tmp_policy: String,
    pub cold_runs: u32,
    pub warm_runs: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchReportFormat {
    Json,
    Html,
}

#[derive(Debug, Serialize)]
pub struct SuiteAnalysisReport {
    pub schema_version: String,
    pub suite_id: String,
    pub run_dir: String,
    pub performance_ranking: Vec<RankingRow>,
    pub scientific_deltas: Vec<DeltaRow>,
    pub claims_registry: ClaimsRegistry,
    pub scientific_sufficiency: ScientificSufficiency,
    pub comparability_hash: String,
    pub environment: EnvironmentSnapshot,
    pub outliers: Vec<String>,
    pub invalid_runs_excluded: usize,
}

#[derive(Debug, Serialize)]
pub struct RankingRow {
    pub stage: String,
    pub tool: String,
    pub score: f64,
}

#[derive(Debug, Serialize)]
pub struct DeltaRow {
    pub stage: String,
    pub metric: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub struct ClaimsRegistry {
    pub can_conclude: Vec<String>,
    pub cannot_conclude: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ScientificSufficiency {
    pub sufficient: bool,
    pub thresholds: serde_json::Value,
    pub reasons: Vec<String>,
}

fn default_repetitions() -> u32 {
    1
}

pub(super) fn supported_suite_schema_version(schema_version: &str) -> bool {
    matches!(
        schema_version,
        "bijux.bench.suite.v1" | "bijux.bench-suite.fastq.v1"
    )
}

#[derive(Debug, Clone, Default)]
struct BenchKnobs {
    repetitions: Option<u32>,
    threads: Option<u32>,
    mem_gb: Option<u32>,
    cold_runs: Option<u32>,
    warm_runs: Option<u32>,
    tmp_policy: Option<String>,
}

fn load_bench_knobs(cwd: &Path) -> BenchKnobs {
    let path = bijux_dna_infra::configs_file(cwd, "bench/knobs.toml");
    let Ok(raw) = fs::read_to_string(path) else {
        return BenchKnobs::default();
    };
    let Ok(parsed): Result<toml::Value, _> = toml::from_str(&raw) else {
        return BenchKnobs::default();
    };
    let defaults = parsed.get("defaults").and_then(toml::Value::as_table);
    BenchKnobs {
        repetitions: defaults
            .and_then(|defaults| defaults.get("repetitions"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok()),
        threads: defaults
            .and_then(|defaults| defaults.get("threads"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok()),
        mem_gb: defaults
            .and_then(|defaults| defaults.get("mem_gb"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok()),
        cold_runs: defaults
            .and_then(|defaults| defaults.get("cold_runs"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok()),
        warm_runs: defaults
            .and_then(|defaults| defaults.get("warm_runs"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u32::try_from(v).ok()),
        tmp_policy: defaults
            .and_then(|defaults| defaults.get("tmp_policy"))
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    }
}

impl SuiteSpec {
    fn effective_fairness(&self) -> FairnessSpec {
        if let Some(fairness) = self.fairness.clone() {
            return fairness;
        }
        let (threads, mem_gb) = self
            .resource_hints
            .as_ref()
            .map_or((8_u32, 32_u32), |hints| (hints.threads, hints.mem_gb));
        FairnessSpec {
            threads,
            mem_gb,
            tmp_policy: "unique-per-run".to_string(),
            cold_runs: 1,
            warm_runs: self.repetitions.max(1),
        }
    }
}

pub fn load_suite(cwd: &Path, suite: &str) -> Result<SuiteSpec> {
    let path = suite_path(cwd, suite)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let parsed: SuiteSpec =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if !supported_suite_schema_version(&parsed.schema_version) {
        return Err(anyhow!(
            "unsupported suite schema `{}` in {}",
            parsed.schema_version,
            path.display()
        ));
    }
    if parsed.suite_id != suite {
        return Err(anyhow!(
            "suite_id mismatch: file is `{}` but suite argument is `{}`",
            parsed.suite_id,
            suite
        ));
    }
    let knobs = load_bench_knobs(cwd);
    let mut effective = parsed;
    if effective.repetitions == default_repetitions() {
        if let Some(repetitions) = knobs.repetitions {
            effective.repetitions = repetitions.max(1);
        }
    }
    if effective.fairness.is_none()
        && (knobs.threads.is_some()
            || knobs.mem_gb.is_some()
            || knobs.cold_runs.is_some()
            || knobs.warm_runs.is_some()
            || knobs.tmp_policy.is_some())
    {
        effective.fairness = Some(FairnessSpec {
            threads: knobs.threads.unwrap_or(8),
            mem_gb: knobs.mem_gb.unwrap_or(32),
            tmp_policy: knobs.tmp_policy.unwrap_or_else(|| "unique-per-run".to_string()),
            cold_runs: knobs.cold_runs.unwrap_or(1),
            warm_runs: knobs.warm_runs.unwrap_or(effective.repetitions.max(1)),
        });
    }
    validate_suite_contracts(&effective)?;
    Ok(effective)
}

#[cfg(test)]
mod schema_tests {
    use super::supported_suite_schema_version;

    #[test]
    fn governed_suite_schema_ids_are_accepted() {
        assert!(supported_suite_schema_version("bijux.bench.suite.v1"));
        assert!(supported_suite_schema_version("bijux.bench-suite.fastq.v1"));
        assert!(!supported_suite_schema_version("bijux.bench.suite.v0"));
    }
}
