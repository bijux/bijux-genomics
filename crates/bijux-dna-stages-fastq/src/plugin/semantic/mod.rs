use super::*;

mod quality;
mod support;

pub(super) fn observed_semantic_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> serde_json::Value {
    if let Some(semantics) = quality::observed_quality_metrics(plan, artifacts) {
        return semantics;
    }
    serde_json::Value::Null
}
