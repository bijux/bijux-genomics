use super::{quality_qc, quality_read_flow, ArtifactRef, StagePlanV1};

pub(super) fn observed_quality_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if let Some(semantics) = quality_read_flow::observed_quality_read_flow_metrics(plan, artifacts)
    {
        return Some(semantics);
    }
    if let Some(semantics) = quality_qc::observed_quality_qc_metrics(plan, artifacts) {
        return Some(semantics);
    }
    None
}
