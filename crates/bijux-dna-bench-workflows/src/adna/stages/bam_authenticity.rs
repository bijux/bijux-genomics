//! Owner: bijux-dna-bench-workflows
//! Ancient DNA stages bam authenticity.
use super::super::{
    fs, path_relative_to_repo, required_output, succeeded_row, AdnaMicroPipelineRow, BTreeMap,
    Context, DamageMetricsV1, Path, Result, Value, BAM_AUTHENTICITY_TOOL_ID,
};
use crate::execution::stage::StageIdentity;
use bijux_dna_core::id_catalog as stage;

pub(crate) fn run_bam_authenticity_stage(
    repo_root: &Path,
    output_root: &Path,
    align_row: &AdnaMicroPipelineRow,
    damage_row: &AdnaMicroPipelineRow,
) -> Result<AdnaMicroPipelineRow> {
    let stage_root = output_root.join("artifacts/bam.authenticity");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let aligned_bam = repo_root.join(required_output(align_row, "aligned_bam")?);
    let damage_metrics_path = repo_root.join(required_output(damage_row, "damage_metrics_json")?);
    let damage_metrics: DamageMetricsV1 = serde_json::from_slice(
        &fs::read(&damage_metrics_path)
            .with_context(|| format!("read {}", damage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", damage_metrics_path.display()))?;
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_authenticity_advisory(
        &aligned_bam,
        &damage_metrics,
    )?;
    let report_path = stage_root.join("authenticity.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &summary)?;

    Ok(succeeded_row(
        StageIdentity {
            stage_id: stage::BAM_AUTHENTICITY,
            domain: "bam",
            tool_id: BAM_AUTHENTICITY_TOOL_ID,
            execution_mode: "domain_contract",
        },
        path_relative_to_repo(repo_root, &report_path),
        &summary.schema_version,
        BTreeMap::from([
            ("aligned_bam".to_string(), required_output(align_row, "aligned_bam")?),
            ("damage_report_json".to_string(), required_output(damage_row, "damage_report_json")?),
        ]),
        BTreeMap::from([(
            "authenticity_report_json".to_string(),
            path_relative_to_repo(repo_root, &report_path),
        )]),
        BTreeMap::from([
            ("score".to_string(), Value::from(summary.score)),
            ("confidence".to_string(), Value::from(summary.confidence)),
            ("pmd_like_signal_present".to_string(), Value::from(summary.pmd_like_signal_present)),
        ]),
        "composed authenticity advisory evidence from damage-bearing aligned molecules".to_string(),
    ))
}
