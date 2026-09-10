//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam damage.
use super::super::{
    derive_damage_metrics, path_relative_to_repo, required_output, succeeded_row,
    AdnaMicroPipelineRow, BTreeMap, Context, Path, PipelineInputFixtures, Result, Value,
    BAM_DAMAGE_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_damage_stage(
    repo_root: &Path,
    output_root: &Path,
    reference_fasta: &Path,
    input_fixtures: &PipelineInputFixtures,
    trim_row: &AdnaMicroPipelineRow,
    align_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.damage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let trimmed_r1 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r1_path")?);
    let trimmed_r2 =
        repo_root.join(required_output(trim_row, "terminal_damage_trimmed_reads_r2_path")?);
    let damage_metrics = derive_damage_metrics(
        reference_fasta,
        &input_fixtures.reference_name,
        &trimmed_r1,
        &trimmed_r2,
        &input_fixtures.placements,
    )?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_damage_evidence(
        &aligned_bam,
        &damage_metrics,
        true,
    )?;
    let damage_metrics_path = stage_root.join("damage.metrics.json");
    let report_path = stage_root.join("damage.summary.json");
    bijux_dna_infra::atomic_write_json(&damage_metrics_path, &damage_metrics)?;
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_DAMAGE,
            domain: "bam",
            tool_id: BAM_DAMAGE_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?)]),
        BTreeMap::from([
            ("damage_report_json".to_string(), path_relative_to_repo(repo_root, &report_path)),
            (
                "damage_metrics_json".to_string(),
                path_relative_to_repo(repo_root, &damage_metrics_path),
            ),
        ]),
        BTreeMap::from([
            ("terminal_c_to_t_5p".to_string(), Value::from(summary.terminal_c_to_t_5p)),
            ("terminal_g_to_a_3p".to_string(), Value::from(summary.terminal_g_to_a_3p)),
            ("short_fragment_fraction".to_string(), Value::from(summary.short_fragment_fraction)),
        ]),
        "derived ancient-DNA terminal damage evidence from the aligned and trimmed read support"
            .to_string(),
    ))
}
