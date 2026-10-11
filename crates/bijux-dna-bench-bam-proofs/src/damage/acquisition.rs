use super::models::LocalDamageSmokeMetrics;
use super::observations::DamageParserOutputReport;
use super::observations::DamageStageProof;
use super::observations::DamageUnifiedMetricsReport;
use super::observations::LocalDamageSmokeReport;
use crate::ports::NativeProofKind;
use crate::ProofContext;
use anyhow::Result;
pub(super) fn load_damage_stage_proof(context: &ProofContext<'_>) -> Result<DamageStageProof> {
    let repo_root = context.repo_root;
    let local_smoke_report_path = context.native.write(repo_root, NativeProofKind::Damage)?;
    let local_smoke_report: LocalDamageSmokeReport =
        crate::input::read_json(&local_smoke_report_path)?;

    let summary_path = repo_root.join(&local_smoke_report.damage_report);
    let summary: bijux_dna_domain_bam::BamDamageEvidenceV1 =
        crate::input::read_json(&summary_path)?;

    let unified_metrics_path = repo_root.join(&local_smoke_report.terminal_position_metrics);
    let unified_metrics: DamageUnifiedMetricsReport =
        crate::input::read_json(&unified_metrics_path)?;

    let parser_output_path = repo_root.join(&local_smoke_report.parser_output);
    let parser_output: DamageParserOutputReport = crate::input::read_json(&parser_output_path)?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalDamageSmokeMetrics = crate::input::read_json(&stage_metrics_path)?;

    Ok(DamageStageProof {
        local_smoke_report_path,
        local_smoke_report,
        summary,
        unified_metrics,
        parser_output,
        normalized_metrics,
    })
}
