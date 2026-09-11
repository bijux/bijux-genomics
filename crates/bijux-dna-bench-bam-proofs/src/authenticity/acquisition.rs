use super::models::AuthenticityCompositeReport;
use super::models::LocalAuthenticitySmokeMetrics;
use super::models::LocalAuthenticitySmokeReport;
use super::observations::AuthenticityStageProof;
use crate::ports::NativeProofKind;
use crate::ProofContext;
use anyhow::Result;
pub(super) fn load_authenticity_stage_proof(
    context: &ProofContext<'_>,
) -> Result<AuthenticityStageProof> {
    let repo_root = context.repo_root;
    let local_smoke_report_path = context.native.write(repo_root, NativeProofKind::Authenticity)?;
    let local_smoke_report: LocalAuthenticitySmokeReport =
        crate::input::read_json(&local_smoke_report_path)?;

    let authenticity_report_path = repo_root.join(&local_smoke_report.authenticity_report);
    let authenticity_report: serde_json::Value =
        crate::input::read_json(&authenticity_report_path)?;

    let summary_path = repo_root.join(&local_smoke_report.authenticity_summary);
    let summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 =
        crate::input::read_json(&summary_path)?;

    let composition_path = repo_root.join(&local_smoke_report.authenticity_composite);
    let composition: AuthenticityCompositeReport = crate::input::read_json(&composition_path)?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalAuthenticitySmokeMetrics =
        crate::input::read_json(&stage_metrics_path)?;

    Ok(AuthenticityStageProof {
        local_smoke_report_path,
        local_smoke_report,
        authenticity_report,
        summary,
        composition,
        normalized_metrics,
    })
}
