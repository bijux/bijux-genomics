//! Owner: bijux-dna-bench-workflows
//! Core germline stages fastq filter.
use super::super::{
    path_relative_to_repo, required_output, write_filter_reads_report, BTreeMap, Context,
    CoreGermlineMicroPipelineRow, FilterEffectiveParams, PairedMode, Path, Result, Value,
    FASTQ_FILTER_TOOL_ID, FILTER_READS_REPORT_SCHEMA_VERSION,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_fastq_filter_stage(
    repo_root: &Path,
    output_root: &Path,
    trim_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/fastq.filter_reads");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let input_r1 = repo_root.join(required_output(trim_row, "trimmed_reads_r1_path")?);
    let input_r2 = repo_root.join(required_output(trim_row, "trimmed_reads_r2_path")?);
    let output_r1 = stage_root.join("filtered_R1.fastq");
    let output_r2 = stage_root.join("filtered_R2.fastq");
    let report_path = stage_root.join("filter.report.json");
    let raw_backend_report = stage_root.join("filter.backend.json");
    let report = write_filter_reads_report(
        repo_root,
        &super::super::observations::FilterReadPaths {
            input_r1: &input_r1,
            input_r2: Some(&input_r2),
            output_r1: &output_r1,
            output_r2: Some(&output_r2),
            report_path: &report_path,
            raw_backend_report: &raw_backend_report,
        },
        &FilterEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 1,
            max_n: None,
            max_n_fraction: None,
            max_n_count: None,
            low_complexity_threshold: None,
            entropy_threshold: None,
            contaminant_db: None,
            n_policy: Some("retain".to_string()),
            polyx_policy: None,
            damage_mode: None,
        },
    )?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::FASTQ_FILTER.to_string(),
        domain: "fastq".to_string(),
        tool_id: FASTQ_FILTER_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        consumed_inputs: BTreeMap::from([
            (
                "trimmed_reads_r1_path".to_string(),
                required_output(trim_row, "trimmed_reads_r1_path")?,
            ),
            (
                "trimmed_reads_r2_path".to_string(),
                required_output(trim_row, "trimmed_reads_r2_path")?,
            ),
        ]),
        outputs: BTreeMap::from([
            ("filtered_reads_r1_path".to_string(), path_relative_to_repo(repo_root, &output_r1)),
            ("filtered_reads_r2_path".to_string(), path_relative_to_repo(repo_root, &output_r2)),
            ("filter_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "filter_backend_report".to_string(),
                path_relative_to_repo(repo_root, &raw_backend_report),
            ),
        ]),
        metrics: BTreeMap::from([
            ("reads_in".to_string(), Value::from(report.reads_in)),
            ("reads_out".to_string(), Value::from(report.reads_out)),
            ("reads_dropped".to_string(), Value::from(report.reads_dropped)),
            ("mean_q_before".to_string(), Value::from(report.mean_q_before)),
            ("mean_q_after".to_string(), Value::from(report.mean_q_after)),
        ]),
    })
}
