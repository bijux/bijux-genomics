//! Owner: bijux-dna-bench-workflows
//! Core germline stages fastq profile.
use super::super::{
    build_profile_report, path_relative_to_repo, required_output, BTreeMap, Context,
    CoreGermlineMicroPipelineRow, Path, Result, Value, FASTQ_PROFILE_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_profile_stage(
    repo_root: &Path,
    output_root: &Path,
    input_r1: &Path,
    input_r2: &Path,
    validate_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.profile_reads");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let report_path = stage_root.join("profile.json");
    let report = build_profile_report(input_r1, Some(input_r2))?;
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::FASTQ_STATS_NEUTRAL.to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_PROFILE_TOOL_ID.to_string(),
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
        outputs: BTreeMap::from([(
            "validated_profile".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        metrics: BTreeMap::from([
            ("reads_total".to_string(), Value::from(report.reads_total)),
            ("bases_total".to_string(), Value::from(report.bases_total)),
            ("mean_q".to_string(), Value::from(report.mean_q)),
            ("gc_percent".to_string(), Value::from(report.gc_percent)),
        ]),
    })
}
