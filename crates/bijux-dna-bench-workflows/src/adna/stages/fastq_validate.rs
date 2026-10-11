//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages fastq validate.
use super::super::{
    path_relative_to_repo, succeeded_row, validation_artifact_paths, AdnaMicroPipelineRow,
    BTreeMap, Context, PairSyncPolicy, Path, Result, ValidationMode, Value, FASTQ_VALIDATE_TOOL_ID,
    VALIDATION_REPORT_SCHEMA_VERSION,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_validate_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.validate_reads");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let artifact_paths = validation_artifact_paths(&stage_root, true);
    let (mut report, manifest) = bijux_dna_domain_fastq::stages::validate_reads(
        input_r1,
        Some(input_r2),
        ValidationMode::Strict,
        PairSyncPolicy::RequireHeaderSync,
        &artifact_paths.validation_log_r1,
        artifact_paths.validation_log_r2.as_deref(),
        &artifact_paths.report_json,
    )?;
    report.input_r1 = path_relative_to_repo(repo_root, input_r1);
    report.input_r2 = Some(path_relative_to_repo(repo_root, input_r2));
    report.validation_log_r1 = path_relative_to_repo(repo_root, &artifact_paths.validation_log_r1);
    report.validation_log_r2 = artifact_paths
        .validation_log_r2
        .as_ref()
        .map(|path| path_relative_to_repo(repo_root, path));
    bijux_dna_infra::atomic_write_json(&artifact_paths.report_json, &report)?;

    let mut manifest = manifest;
    manifest.input_r1 = path_relative_to_repo(repo_root, input_r1);
    manifest.input_r2 = Some(path_relative_to_repo(repo_root, input_r2));
    manifest.validation_report = path_relative_to_repo(repo_root, &artifact_paths.report_json);
    bijux_dna_infra::atomic_write_json(&artifact_paths.validated_reads_manifest, &manifest)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::FASTQ_VALIDATE_READS,
            domain: "fastq",
            tool_id: FASTQ_VALIDATE_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &artifact_paths.report_json),
        VALIDATION_REPORT_SCHEMA_VERSION,
        BTreeMap::from([
            ("raw_reads_r1_path".to_string(), path_relative_to_repo(repo_root, input_r1)),
            ("raw_reads_r2_path".to_string(), path_relative_to_repo(repo_root, input_r2)),
        ]),
        BTreeMap::from([
            ("validated_reads_r1_path".to_string(), path_relative_to_repo(repo_root, input_r1)),
            ("validated_reads_r2_path".to_string(), path_relative_to_repo(repo_root, input_r2)),
            (
                "validation_report".to_string(),
                path_relative_to_repo(repo_root, &artifact_paths.report_json),
            ),
            (
                "validated_reads_manifest".to_string(),
                path_relative_to_repo(repo_root, &artifact_paths.validated_reads_manifest),
            ),
        ]),
        BTreeMap::from([
            ("validated_pairs".to_string(), Value::from(report.validated_pairs.unwrap_or(0))),
            ("strict_pass".to_string(), Value::from(report.strict_pass)),
        ]),
        "validated synthetic aDNA-like read pairs".to_string(),
    ))
}
