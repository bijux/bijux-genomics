//! Owner: runtime and memory evidence acquisition with declared-resource fallback.
use super::read_json_value;
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    MicroBenchmarkExecutionStatus, MicroBenchmarkResultRow, StageToolResourceRow,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;
const GOVERNED_RUNTIME_SOURCE: &str = "governed_runtime";
#[derive(Debug, Clone)]
pub(super) struct RuntimeEvidence {
    pub(super) elapsed_seconds: Option<f64>,
    pub(super) runtime_source: &'static str,
}

#[derive(Debug, Clone)]
pub(super) struct MemoryEvidence {
    pub(super) declared_memory_mb: Option<f64>,
    pub(super) declared_cpu_threads: Option<u32>,
    pub(super) observed_memory_mb: Option<f64>,
    pub(super) observed_cpu_threads: Option<u32>,
    pub(super) memory_source: &'static str,
    pub(super) resource_origin: Option<String>,
}

pub(super) fn load_runtime_evidence(
    repo_root: &Path,
    row: &MicroBenchmarkResultRow,
    governed_runtime_projection: bool,
) -> Result<RuntimeEvidence> {
    if row.status != MicroBenchmarkExecutionStatus::Succeeded {
        return Ok(RuntimeEvidence { elapsed_seconds: None, runtime_source: "not_applicable" });
    }
    if let Some(manifest_path) = &row.stage_result_manifest_path {
        if governed_runtime_projection {
            return Ok(RuntimeEvidence {
                elapsed_seconds: None,
                runtime_source: GOVERNED_RUNTIME_SOURCE,
            });
        }
        let manifest = crate::repo::stage_result::load_benchmark_stage_result_manifest(
            &repo_root.join(manifest_path),
        )
        .with_context(|| format!("load {}", repo_root.join(manifest_path).display()))?;
        return Ok(RuntimeEvidence {
            elapsed_seconds: Some(manifest.runtime.elapsed_seconds),
            runtime_source: "stage_result_manifest",
        });
    }
    if let Some(evidence_path) = &row.evidence_path {
        if evidence_path.ends_with(".json") {
            let payload = read_json_value(repo_root.join(evidence_path))?;
            if let Some(elapsed_seconds) = json_optional_f64(&payload, "elapsed_seconds")
                .or_else(|| json_optional_f64(&payload, "runtime_s"))
            {
                return Ok(RuntimeEvidence {
                    elapsed_seconds: Some(elapsed_seconds),
                    runtime_source: "evidence_report",
                });
            }
        }
    }
    Ok(RuntimeEvidence { elapsed_seconds: None, runtime_source: "not_available" })
}

pub(super) fn load_memory_evidence(
    repo_root: &Path,
    row: &MicroBenchmarkResultRow,
    stage_tool_resources: &BTreeMap<(String, String, String), StageToolResourceRow>,
) -> Result<MemoryEvidence> {
    if row.status != MicroBenchmarkExecutionStatus::Succeeded {
        return Ok(MemoryEvidence {
            declared_memory_mb: None,
            declared_cpu_threads: None,
            observed_memory_mb: None,
            observed_cpu_threads: None,
            memory_source: "not_applicable",
            resource_origin: None,
        });
    }
    if let Some(manifest_path) = &row.stage_result_manifest_path {
        let manifest = crate::repo::stage_result::load_benchmark_stage_result_manifest(
            &repo_root.join(manifest_path),
        )
        .with_context(|| format!("load {}", repo_root.join(manifest_path).display()))?;
        if manifest.resource_metrics.memory_mb.is_some()
            || manifest.resource_metrics.cpu_threads.is_some()
        {
            return Ok(MemoryEvidence {
                declared_memory_mb: None,
                declared_cpu_threads: None,
                observed_memory_mb: manifest.resource_metrics.memory_mb,
                observed_cpu_threads: manifest.resource_metrics.cpu_threads,
                memory_source: "stage_result_manifest",
                resource_origin: None,
            });
        }
    }
    if let Some(evidence_path) = &row.evidence_path {
        if evidence_path.ends_with(".json") {
            let payload = read_json_value(repo_root.join(evidence_path))?;
            let observed_memory_mb = json_optional_f64(&payload, "memory_mb")
                .or_else(|| json_optional_f64(&payload, "peak_memory_mb"));
            let observed_cpu_threads = json_optional_u32(&payload, "cpu_threads")
                .or_else(|| json_optional_u32(&payload, "threads"));
            if observed_memory_mb.is_some() || observed_cpu_threads.is_some() {
                return Ok(MemoryEvidence {
                    declared_memory_mb: None,
                    declared_cpu_threads: None,
                    observed_memory_mb,
                    observed_cpu_threads,
                    memory_source: "evidence_report",
                    resource_origin: None,
                });
            }
        }
    }
    if let Some(resource) =
        stage_tool_resources.get(&(row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
    {
        return Ok(MemoryEvidence {
            declared_memory_mb: Some(f64::from(resource.memory_gb) * 1024.0),
            declared_cpu_threads: Some(resource.threads),
            observed_memory_mb: None,
            observed_cpu_threads: None,
            memory_source: "declared_stage_tool_resource",
            resource_origin: Some(resource.resource_origin.clone()),
        });
    }
    Ok(MemoryEvidence {
        declared_memory_mb: None,
        declared_cpu_threads: None,
        observed_memory_mb: None,
        observed_cpu_threads: None,
        memory_source: "not_available",
        resource_origin: None,
    })
}

fn json_optional_f64(value: &Value, field: &str) -> Option<f64> {
    value.get(field).and_then(Value::as_f64)
}

fn json_optional_u32(value: &Value, field: &str) -> Option<u32> {
    value.get(field).and_then(Value::as_u64).and_then(|value| u32::try_from(value).ok())
}
