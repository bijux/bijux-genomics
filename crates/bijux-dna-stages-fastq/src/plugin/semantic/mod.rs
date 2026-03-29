use super::*;

mod feature_tables;
mod processing;
mod profiling;
mod quality;
mod support;

pub(super) fn observed_semantic_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> serde_json::Value {
    if let Some(semantics) = quality::observed_quality_metrics(plan, artifacts) {
        return semantics;
    }
    if let Some(semantics) = feature_tables::observed_feature_table_metrics(plan, artifacts) {
        return semantics;
    }
    if let Some(semantics) = profiling::observed_profiling_metrics(plan, artifacts) {
        return semantics;
    }
    if let Some(semantics) = processing::observed_processing_metrics(plan, artifacts) {
        return semantics;
    }
    serde_json::Value::Null
}
