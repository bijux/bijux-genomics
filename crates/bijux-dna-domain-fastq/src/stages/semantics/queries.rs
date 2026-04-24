use crate::metrics::spec::{metric_spec_for_stage, MetricClass};
use crate::pipeline_contract::{self, StageCriticality};
use bijux_dna_core::ids::StageId;

use super::catalog::{FastqStageKind, StageSemantics, STAGES};

#[must_use]
pub fn stage_semantics(stage_id: &StageId) -> Option<StageSemantics> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id.as_str() == stage_id.as_str())
        .map(|stage| stage.semantics)
}

#[must_use]
pub fn stage_kind(stage_id: &StageId) -> Option<FastqStageKind> {
    STAGES.iter().find(|stage| stage.stage_id.as_str() == stage_id.as_str()).map(|stage| stage.kind)
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id.as_str() == stage_id.as_str())
        .map(|stage| stage.criticality)
}

#[must_use]
pub fn fastq_stage_is_stable(stage_id: &StageId) -> bool {
    !matches!(stage_criticality(stage_id), Some(StageCriticality::Experimental))
}

#[must_use]
pub fn stage_metric_classes(stage_id: &StageId) -> Option<&'static [MetricClass]> {
    stage_semantics(stage_id).map(|semantics| semantics.affects_metrics)
}

#[must_use]
pub fn stage_metric_invariants(stage_id: &StageId) -> Option<&'static [&'static str]> {
    metric_spec_for_stage(stage_id.as_str()).map(|spec| spec.invariants)
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    pipeline_contract::canonical_stage_order()
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    pipeline_contract::optional_branches()
}
