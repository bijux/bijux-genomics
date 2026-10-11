//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam coverage.
use super::super::{
    path_relative_to_repo, required_output, succeeded_row, AdnaMicroPipelineRow,
    AlignmentFixtureFormat, AlignmentFixtureOperations, BTreeMap, Context, Path, Result, Value,
    BAM_COVERAGE_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_coverage_stage(
    alignment: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    mapping_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.coverage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let align_sam = repo_root.join(required_output(align_row, "align_sam")?);
    let regions_path = stage_root.join("coverage_regions.bed");
    alignment.write_reference_intervals(&align_sam, AlignmentFixtureFormat::Sam, &regions_path)?;
    let (summary, regions) = bijux_dna_domain_bam::summarize_tiny_bam_coverage_regions(
        &align_sam,
        Some(&regions_path),
        &[1, 5],
    )?;
    let covered_bases = regions.iter().map(|region| region.covered_bases).sum::<u64>();
    let summary_path = stage_root.join("coverage.summary.json");
    let regions_json_path = stage_root.join("coverage.regions.json");
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)?;
    bijux_dna_infra::atomic_write_json(&regions_json_path, &regions)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_COVERAGE,
            domain: "bam",
            tool_id: BAM_COVERAGE_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &summary_path),
        &summary.schema_version,
        BTreeMap::from([
            ("align_sam".to_string(), required_output(align_row, "align_sam")?),
            (
                "mapping_summary_report_json".to_string(),
                required_output(mapping_row, "mapping_summary_report_json")?,
            ),
        ]),
        BTreeMap::from([
            ("coverage_report_json".to_string(), path_relative_to_repo(repo_root, &summary_path)),
            (
                "coverage_regions_json".to_string(),
                path_relative_to_repo(repo_root, &regions_json_path),
            ),
        ]),
        BTreeMap::from([
            ("mean_depth".to_string(), summary.mean_depth.map_or(Value::Null, Value::from)),
            ("covered_bases".to_string(), Value::from(covered_bases)),
        ]),
        "measured coverage across the governed ancient-DNA reference contigs".to_string(),
    ))
}
