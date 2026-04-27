//! Owner: bijux-dna-stages-fastq
//! Observer artifacts: stable, canonical JSON outputs for derived FASTQ metadata.
//! Formats are versioned and written atomically under run artifacts.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_core::id_catalog;

use crate::stage_specs::{
    AdapterBankV1, AdapterTrimmingReportV1, EffectiveAdapterSet, RetentionReportV1, ToolReferenceV1,
};
use crate::StagePlanJson;

fn run_artifacts_dir(run_dir: &Path) -> PathBuf {
    run_dir.join("run_artifacts")
}

/// # Errors
/// Returns an error if the effective adapter artifact directory or JSON payload cannot be written.
pub fn write_effective_adapters(
    run_dir: &Path,
    effective: &EffectiveAdapterSet,
    bank_checksum: &str,
    presets_checksum: &str,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dir);
    let adapters_dir = root.join("adapters");
    bijux_dna_infra::ensure_dir(&adapters_dir).context("create adapters artifact dir")?;
    let path = adapters_dir.join("effective_adapters.json");
    let adapters: Vec<serde_json::Value> = effective
        .adapters
        .iter()
        .map(|adapter| {
            serde_json::json!({
                "id": adapter.id,
                "sequence": adapter.sequence,
            })
        })
        .collect();
    let payload = serde_json::json!({
        "schema_version": "bijux.effective_adapters.v1",
        "preset": effective.preset,
        "enabled_adapter_ids": effective.enabled_ids,
        "adapters": adapters,
        "bank_checksum": bank_checksum,
        "presets_checksum": presets_checksum
    });
    bijux_dna_infra::atomic_write_json(&path, &payload).context("write effective_adapters.json")?;
    Ok(path)
}

/// # Errors
/// Returns an error if the adapter bank reference artifact directory or JSON payload cannot be
/// written.
pub fn write_adapter_bank_ref(
    run_dir: &Path,
    bank: &AdapterBankV1,
    bank_path: &Path,
    presets_path: &Path,
    bank_checksum: &str,
    presets_checksum: &str,
    effective: &EffectiveAdapterSet,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dir);
    let adapters_dir = root.join("adapters");
    bijux_dna_infra::ensure_dir(&adapters_dir).context("create adapters artifact dir")?;
    let path = adapters_dir.join("adapter_bank_ref.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.adapter_bank_ref.v1",
        "bank_schema": bank.schema_version,
        "bank_id": bank.bank_id,
        "bank_version": bank.version,
        "bank_checksum": bank_checksum,
        "presets_checksum": presets_checksum,
        "preset": effective.preset,
        "enabled_adapter_ids": effective.enabled_ids,
        "sources": {
            "bank_path": bank_path.display().to_string(),
            "presets_path": presets_path.display().to_string()
        }
    });
    bijux_dna_infra::atomic_write_json(&path, &payload).context("write adapter_bank_ref.json")?;
    Ok(path)
}

#[allow(clippy::too_many_arguments)]
/// # Errors
/// Returns an error if the adapter trimming report directory or JSON payload cannot be written.
pub fn write_adapter_trimming_report(
    run_dir: &Path,
    tool: &str,
    tool_version: &str,
    params: &serde_json::Value,
    total_reads: u64,
    reads_with_adapter: u64,
    bases_trimmed_total: u64,
    per_adapter_counts: std::collections::BTreeMap<String, u64>,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dir);
    let reports_dir = root.join("reports");
    bijux_dna_infra::ensure_dir(&reports_dir).context("create reports artifact dir")?;
    let path = reports_dir.join("adapter_trimming_report.json");
    let report = AdapterTrimmingReportV1 {
        schema_version: "bijux.adapter_trimming_report.v1".to_string(),
        reads_with_adapter,
        total_reads,
        bases_trimmed_total,
        per_adapter_counts,
        top_k_adapters: Vec::new(),
        tool: ToolReferenceV1 {
            id: tool.to_string(),
            stage: id_catalog::FASTQ_TRIM.to_string(),
            version: tool_version.to_string(),
            params: params.clone(),
        },
    };
    bijux_dna_infra::atomic_write_json(&path, &report)
        .context("write adapter_trimming_report.json")?;
    Ok(path)
}

/// # Errors
/// Returns an error if the retention report directory or JSON payload cannot be written.
pub fn write_retention_report_artifact(
    run_dir: &Path,
    report: &RetentionReportV1,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dir);
    let reports_dir = root.join("reports");
    bijux_dna_infra::ensure_dir(&reports_dir).context("create reports artifact dir")?;
    let path = reports_dir.join("retention_report.json");
    bijux_dna_infra::atomic_write_json(&path, report).context("write retention_report.json")?;
    Ok(path)
}

/// # Errors
/// Returns an error if the stage plan directory tree or JSON payload cannot be written.
pub fn write_stage_plan_json(
    run_dir: &Path,
    file_name: &str,
    plan: &StagePlanJson,
) -> Result<PathBuf> {
    let root = run_artifacts_dir(run_dir);
    let plans_dir = root.join("plans");
    bijux_dna_infra::ensure_dir(&plans_dir).context("create plans artifact dir")?;
    let path = plans_dir.join(file_name);
    bijux_dna_infra::ensure_dir(path.parent().unwrap_or(&plans_dir))
        .context("create plan parent dir")?;
    bijux_dna_infra::atomic_write_json(&path, plan).context("write stage plan json")?;
    Ok(path)
}
