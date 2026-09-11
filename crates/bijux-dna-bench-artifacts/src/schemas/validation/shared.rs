use super::super::location::{path_relative_to_repo, repo_relative_path};
use super::contracts::{
    BAM_SCHEMA_VALIDATION_REPORT_VERSION, FASTQ_SCHEMA_VALIDATION_REPORT_VERSION,
};
use super::io::read_json_value;
use super::model::{
    BamSchemaValidationReport, FastqSchemaValidationReport, SharedSchemaValidationRow,
};
use anyhow::{anyhow, Result};
use std::path::Path;
pub(super) fn build_fastq_schema_validation_report(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
) -> Result<FastqSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let shared_schema_path = repo_relative_path(repo_root, shared_schema_path);
    let canonical_shared = super::super::publication::render_fastq_normalized_metrics_schema();
    let disk_shared = read_json_value(&shared_schema_path)?;
    let rows = collect_shared_schema_rows(&canonical_shared, "FASTQ")?;

    Ok(FastqSchemaValidationReport {
        schema_version: FASTQ_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "fastq",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        passes_gate: disk_shared == canonical_shared,
        shared_schema_matches: disk_shared == canonical_shared,
        stage_count: rows.len(),
        rows,
    })
}
pub(super) fn build_bam_schema_validation_report(
    repo_root: &Path,
    output_path: &Path,
    shared_schema_path: &Path,
) -> Result<BamSchemaValidationReport> {
    let output_path = repo_relative_path(repo_root, output_path);
    let shared_schema_path = repo_relative_path(repo_root, shared_schema_path);
    let canonical_shared = super::super::publication::render_bam_normalized_metrics_schema();
    let disk_shared = read_json_value(&shared_schema_path)?;
    let rows = collect_shared_schema_rows(&canonical_shared, "BAM")?;

    Ok(BamSchemaValidationReport {
        schema_version: BAM_SCHEMA_VALIDATION_REPORT_VERSION,
        domain: "bam",
        output_path: path_relative_to_repo(repo_root, &output_path),
        shared_schema_path: path_relative_to_repo(repo_root, &shared_schema_path),
        passes_gate: disk_shared == canonical_shared,
        shared_schema_matches: disk_shared == canonical_shared,
        stage_count: rows.len(),
        rows,
    })
}
pub(super) fn collect_shared_schema_rows(
    schema: &serde_json::Value,
    domain_name: &str,
) -> Result<Vec<SharedSchemaValidationRow>> {
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            anyhow!("{domain_name} normalized metrics schema is missing object `$defs.stages`")
        })?;

    let mut rows = stage_defs
        .iter()
        .map(|(stage_id, value)| {
            let stage_contract = value
                .get("allOf")
                .and_then(serde_json::Value::as_array)
                .and_then(|items| items.get(1))
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing stage extension"))?;
            let extension_id = stage_contract
                .get("x-bijux-extension-id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing string `x-bijux-extension-id`"))?;
            let required_key_count = stage_contract
                .get("required")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| anyhow!("{domain_name} normalized metrics stage `{stage_id}` is missing `required` keys"))?
                .len();
            Ok(SharedSchemaValidationRow {
                stage_id: stage_id.clone(),
                extension_id: extension_id.to_string(),
                required_key_count,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(rows)
}
