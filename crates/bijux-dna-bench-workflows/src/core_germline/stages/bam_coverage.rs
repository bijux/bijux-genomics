//! Owner: bijux-dna-bench-workflows
//! Core germline stages bam coverage.
use super::super::{
    path_relative_to_repo, required_output, AlignmentFixtureFormat, AlignmentFixtureOperations,
    BTreeMap, Context, CoreGermlineMicroPipelineRow, Path, Result, Value, BAM_COVERAGE_TOOL_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_coverage_stage(
    alignment: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_root: &Path,
    align_row: &CoreGermlineMicroPipelineRow,
    qc_pre_row: &CoreGermlineMicroPipelineRow,
) -> Result<CoreGermlineMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.coverage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let regions_bed = stage_root.join("coverage_regions.bed");
    alignment.write_reference_intervals(&aligned_bam, AlignmentFixtureFormat::Bam, &regions_bed)?;
    let (summary, region_rows) = bijux_dna_domain_bam::summarize_tiny_bam_coverage_regions(
        &aligned_bam,
        Some(&regions_bed),
        &[1],
    )?;
    let report_path = stage_root.join("coverage.summary.json");
    let regions_path = stage_root.join("coverage.regions.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;
    bijux_dna_infra::atomic_write_json(&regions_path, &region_rows)?;

    Ok(CoreGermlineMicroPipelineRow {
        stage_id: stage::BAM_COVERAGE.to_string(),
        domain: "bam".to_string(),
        tool_id: BAM_COVERAGE_TOOL_ID.to_string(),
        execution_mode: "domain_contract".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &report_path),
        parsed_schema_version: summary.schema_version.clone(),
        consumed_inputs: BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("aligned_bai".to_string(), required_output(align_row, "aligned_bai")?),
            ("qc_pre_report".to_string(), required_output(qc_pre_row, "qc_pre_report")?),
            (
                "coverage_region_contract".to_string(),
                path_relative_to_repo(repo_root, &regions_bed),
            ),
        ]),
        outputs: BTreeMap::from([
            ("coverage_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            ("coverage_regions_json".to_string(), path_relative_to_repo(repo_root, &regions_path)),
        ]),
        metrics: BTreeMap::from([
            ("mean_depth".to_string(), Value::from(summary.mean_depth.unwrap_or_default())),
            (
                "region_count".to_string(),
                Value::from(u64::try_from(region_rows.len()).unwrap_or(u64::MAX)),
            ),
            (
                "coverage_regime".to_string(),
                Value::from(summary.coverage_regime.unwrap_or_else(|| "unknown".to_string())),
            ),
        ]),
    })
}
