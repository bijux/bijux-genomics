//! Owner: bijux-dna-bench-workflows
//! Core germline stages fastq trim.
use super::super::{
    json, path_relative_to_repo, required_output, BTreeMap, Context, CoreGermlineMicroPipelineRow,
    PairedMode, Path, Result, TrimEffectiveParams, Value, FASTQ_TRIM_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_trim_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.trim_reads");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let output_r1 = stage_root.join("trimmed_R1.fastq");
    let output_r2 = stage_root.join("trimmed_R2.fastq");
    let report_path = stage_root.join("trim.report.json");
    let raw_backend_report = stage_root.join("trim.backend.json");
    let report = bijux_dna_domain_fastq::stages::trim_reads(
        input_r1,
        Some(input_r2),
        &TrimEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            min_len: 4,
            q_cutoff: None,
            adapter_policy: "none".to_string(),
            damage_mode: None,
            polyx_policy: None,
            n_policy: Some("retain".to_string()),
            contaminant_policy: Some("none".to_string()),
        },
        FASTQ_TRIM_TOOL_ID,
        &output_r1,
        Some(&output_r2),
        Some(&raw_backend_report),
    )?;
    if !raw_backend_report.is_file() {
        bijux_dna_infra::write_bytes(
            &raw_backend_report,
            json!({
                "summary": {
                    "reads_in": report.reads_in,
                    "reads_out": report.reads_out,
                    "bases_in": report.bases_in,
                    "bases_out": report.bases_out,
                }
            })
            .to_string(),
        )?;
    }
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::FASTQ_TRIM.to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_TRIM_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: report.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            (
                "validated_reads_r1_path".to_string(),
                required_output(validate_row, "validated_reads_r1_path")?,
            ),
            (
                "validated_reads_r2_path".to_string(),
                required_output(validate_row, "validated_reads_r2_path")?,
            ),
        ]),
        outputs: BTreeMap::from([
            ("trimmed_reads_r1_path".to_string(), path_relative_to_repo(repo_root, &output_r1)),
            ("trimmed_reads_r2_path".to_string(), path_relative_to_repo(repo_root, &output_r2)),
            ("trim_report".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "trim_backend_report".to_string(),
                path_relative_to_repo(repo_root, &raw_backend_report),
            ),
        ]),
        metrics: BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in.unwrap_or(0))),
            ("reads_out".to_string(), Value::from(report.reads_out.unwrap_or(0))),
            ("bases_in".to_string(), Value::from(report.bases_in.unwrap_or(0))),
            ("bases_out".to_string(), Value::from(report.bases_out.unwrap_or(0))),
        ]),
    })
}
