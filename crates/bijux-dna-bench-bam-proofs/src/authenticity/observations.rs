use super::models::AuthenticityCompositeReport;
use super::models::LocalAuthenticitySmokeMetrics;
use super::models::LocalAuthenticitySmokeReport;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub(super) struct AuthenticityStageProof {
    pub(super) local_smoke_report_path: PathBuf,
    pub(super) local_smoke_report: LocalAuthenticitySmokeReport,
    pub(super) authenticity_report: serde_json::Value,
    pub(super) summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1,
    pub(super) composition: AuthenticityCompositeReport,
    pub(super) normalized_metrics: LocalAuthenticitySmokeMetrics,
}
