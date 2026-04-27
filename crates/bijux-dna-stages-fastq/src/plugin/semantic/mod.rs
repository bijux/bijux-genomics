use super::{ArtifactRef, StagePlanV1};

mod feature_tables;
mod processing;
mod processing_cleanup;
mod processing_read_preparation;
mod processing_trimming;
mod profiling;
mod quality;
mod quality_qc;
mod quality_read_flow;
mod taxonomy;
mod validation_semantics;

#[cfg(test)]
pub(crate) use self::validation_semantics::validate_semantic_metrics;

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
    if let Some(semantics) = taxonomy::observed_taxonomy_metrics(plan, artifacts) {
        return semantics;
    }
    serde_json::Value::Null
}
