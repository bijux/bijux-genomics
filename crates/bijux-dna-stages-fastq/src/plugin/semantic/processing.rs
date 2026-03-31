use super::{
    processing_cleanup, processing_read_preparation, processing_trimming, ArtifactRef, StagePlanV1,
};

pub(super) fn observed_processing_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if let Some(semantics) = processing_cleanup::observed_cleanup_metrics(plan, artifacts) {
        return Some(semantics);
    }
    if let Some(semantics) =
        processing_read_preparation::observed_read_preparation_metrics(plan, artifacts)
    {
        return Some(semantics);
    }
    if let Some(semantics) = processing_trimming::observed_trimming_metrics(plan, artifacts) {
        return Some(semantics);
    }
    None
}
