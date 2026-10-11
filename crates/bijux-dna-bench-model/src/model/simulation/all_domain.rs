//! Owner: all-domain simulation input/output contracts and deterministic output conventions.
use crate::AllDomainOutputDeclarationRow;
use anyhow::{anyhow, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub const DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/all-domains";
pub const ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION: &str = "bijux.bench.local_all_domain_fake_runs.v1";
pub const ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_run_metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeRunOutputEntry {
    pub artifact_id: String,
    pub declared_output: String,
    pub fake_run_path: String,
    pub role: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeRunMetrics {
    #[serde(deserialize_with = "deserialize_metrics_version")]
    #[schemars(schema_with = "metrics_version")]
    pub schema_version: String,
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_step_count: usize,
    pub declared_output_count: usize,
    pub expected_metric_count: usize,
    pub expected_metrics: Vec<String>,
    pub materialized_byte_count: u64,
    pub simulated_elapsed_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeRunResultReport {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_source: String,
    pub command_step_count: usize,
    pub declared_output_count: usize,
    pub created_output_count: usize,
    pub expected_metric_count: usize,
    pub command_script_path: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub metrics_path: String,
    pub stage_result_path: String,
    pub outputs: Vec<AllDomainFakeRunOutputEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeRunsReport {
    #[serde(deserialize_with = "deserialize_report_version")]
    #[schemars(schema_with = "report_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub root_manifest_path: String,
    pub result_count: usize,
    pub created_output_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub results: Vec<AllDomainFakeRunResultReport>,
}

/// Return each declared illustrative output once, preserving role order.
#[must_use]
pub fn all_domain_simulation_output_ids(outputs: &AllDomainOutputDeclarationRow) -> Vec<String> {
    let mut seen = BTreeSet::<&str>::new();
    let mut ordered = Vec::<String>::new();
    for artifact_id in outputs
        .raw_outputs
        .iter()
        .chain(outputs.normalized_metrics.iter())
        .chain(outputs.index_outputs.iter())
    {
        if seen.insert(artifact_id.as_str()) {
            ordered.push(artifact_id.clone());
        }
    }
    ordered
}
/// Project a declared artifact identifier to its illustrative local filename.
#[must_use]
pub fn all_domain_simulation_output_path(artifact_id: &str) -> PathBuf {
    let file_name = if artifact_id.ends_with("_bundle") || artifact_id.ends_with("_dir") {
        return PathBuf::from(artifact_id);
    } else if let Some(stem) = artifact_id.strip_suffix("_vcf_tbi") {
        format!("{stem}.vcf.tbi")
    } else if let Some(stem) = artifact_id.strip_suffix("_vcf") {
        format!("{stem}.vcf")
    } else if let Some(stem) = artifact_id.strip_suffix("_bam") {
        format!("{stem}.bam")
    } else if let Some(stem) = artifact_id.strip_suffix("_bai") {
        format!("{stem}.bai")
    } else if let Some(stem) = artifact_id.strip_suffix("_json") {
        format!("{stem}.json")
    } else if let Some(stem) = artifact_id.strip_suffix("_tsv") {
        format!("{stem}.tsv")
    } else if artifact_id.contains("reads") {
        format!("{artifact_id}.fastq")
    } else if artifact_id.contains("representatives") {
        format!("{artifact_id}.fasta")
    } else if artifact_id.contains("classification") || artifact_id.ends_with("_table") {
        format!("{artifact_id}.tsv")
    } else if artifact_id.contains("report")
        || artifact_id.contains("metrics")
        || artifact_id.contains("summary")
        || artifact_id.contains("decision")
        || artifact_id.contains("manifest")
    {
        format!("{artifact_id}.json")
    } else {
        format!("{artifact_id}.txt")
    };
    PathBuf::from(file_name)
}
/// Classify an illustrative artifact role without inspecting scientific content.
#[must_use]
pub fn all_domain_simulation_output_role(artifact_id: &str) -> &'static str {
    if artifact_id.contains("manifest") {
        "manifest"
    } else if artifact_id.contains("metrics") {
        "metrics"
    } else if artifact_id.contains("report") || artifact_id.contains("summary") {
        "report"
    } else if artifact_id.ends_with("_vcf") {
        "vcf"
    } else if artifact_id.ends_with("_vcf_tbi") || artifact_id.ends_with("_bai") {
        "index"
    } else if artifact_id.ends_with("_bam") {
        "bam"
    } else if artifact_id.contains("reads") {
        "reads"
    } else if artifact_id.contains("table") || artifact_id.contains("classification") {
        "table"
    } else if artifact_id.contains("representatives") {
        "sequences"
    } else if artifact_id.ends_with("_bundle") || artifact_id.ends_with("_dir") {
        "bundle"
    } else {
        "artifact"
    }
}
/// Return the historical command-count simulation heuristic; this is not observed runtime.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "The simulated heuristic preserves historical numeric output, not measured runtime precision"
)]
pub fn all_domain_simulation_elapsed_seconds(command_step_count: usize) -> f64 {
    1.0 + (command_step_count as f64 * 0.25)
}
/// Reject missing materialized outputs and required artifact paths.
///
/// # Errors
/// Returns an error when the persisted report disagrees with its declared observations.
pub fn validate_all_domain_simulation_result(report: &AllDomainFakeRunResultReport) -> Result<()> {
    if report.declared_output_count == 0
        || report.declared_output_count != report.outputs.len()
        || report.created_output_count != report.declared_output_count
    {
        return Err(anyhow!(
            "all-domain fake-runner result `{}` did not materialize every declared output",
            report.result_id
        ));
    }
    for relative_path in [
        &report.command_script_path,
        &report.stdout_path,
        &report.stderr_path,
        &report.metrics_path,
        &report.stage_result_path,
    ] {
        if relative_path.trim().is_empty() {
            return Err(anyhow!(
                "all-domain fake-runner result `{}` has an empty artifact path",
                report.result_id
            ));
        }
    }
    if report.outputs.iter().any(|output| !output.exists || output.fake_run_path.trim().is_empty())
    {
        return Err(anyhow!(
            "all-domain fake-runner result `{}` is missing a materialized output path",
            report.result_id
        ));
    }
    Ok(())
}
/// Reject unsupported versions and inconsistent simulation result summaries.
///
/// # Errors
/// Returns an error when the persisted report disagrees with its declared observations.
pub fn validate_all_domain_simulation_report(report: &AllDomainFakeRunsReport) -> Result<()> {
    anyhow::ensure!(
        report.schema_version == ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION,
        "unsupported all-domain simulation schema"
    );
    let mut domains = BTreeMap::new();
    for result in &report.results {
        validate_all_domain_simulation_result(result)?;
        *domains.entry(result.domain.clone()).or_insert(0usize) += 1;
    }
    anyhow::ensure!(
        domains == report.domain_counts,
        "simulation domain counts disagree with results"
    );
    if report.result_count == 0 || report.result_count != report.results.len() {
        return Err(anyhow!(
            "all-domain fake-runner must report every governed benchmark-ready result exactly once"
        ));
    }
    let unique_result_ids =
        report.results.iter().map(|result| result.result_id.as_str()).collect::<BTreeSet<_>>();
    if unique_result_ids.len() != report.result_count {
        return Err(anyhow!("all-domain fake-runner cannot repeat benchmark result identifiers"));
    }
    if report.results.iter().map(|result| result.created_output_count).sum::<usize>()
        != report.created_output_count
    {
        return Err(anyhow!(
            "all-domain fake-runner root counts must match per-result created output counts"
        ));
    }
    Ok(())
}

fn deserialize_report_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserialize_version(deserializer, ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION)
}
fn deserialize_metrics_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserialize_version(deserializer, ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION)
}
fn deserialize_version<'de, D: Deserializer<'de>>(
    deserializer: D,
    expected: &str,
) -> Result<String, D::Error> {
    let value = String::deserialize(deserializer)?;
    if value != expected {
        return Err(serde::de::Error::custom("unsupported all-domain simulation schema version"));
    }
    Ok(value)
}

fn report_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION})
}
fn metrics_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION})
}

#[cfg(test)]
#[path = "../../../tests/contracts/simulation.rs"]
mod tests;
