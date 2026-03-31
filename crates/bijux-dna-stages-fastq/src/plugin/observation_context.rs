use bijux_dna_domain_fastq::BenchmarkScenario;
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

pub(super) struct ObservationContext {
    pub(super) interpretation_level: crate::RuntimeInterpretationLevel,
    pub(super) observer_covered: bool,
    pub(super) benchmark_scenarios: Vec<BenchmarkScenario>,
    pub(super) comparison_artifact_ids: Vec<String>,
    pub(super) semantic_loss: Vec<&'static str>,
    pub(super) artifacts: Vec<ArtifactRef>,
    pub(super) semantic_metrics: serde_json::Value,
    pub(super) declared_metric_invariants: &'static [&'static str],
}

pub(super) fn observation_context(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> ObservationContext {
    let interpretation_level =
        crate::runtime_interpretation_for_stage_tool(&plan.stage_id, &plan.tool_id)
            .unwrap_or(crate::RuntimeInterpretationLevel::GenericEnvelope);
    let observer_covered =
        interpretation_level == crate::RuntimeInterpretationLevel::ObserverSpecialized;
    let benchmark_scenarios = bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&plan.stage_id);
    let comparison_artifact_ids =
        bijux_dna_domain_fastq::comparison_artifact_ids_for_stage(&plan.stage_id);
    let semantic_loss = match interpretation_level {
        crate::RuntimeInterpretationLevel::ObserverSpecialized => Vec::new(),
        crate::RuntimeInterpretationLevel::GenericEnvelope => {
            vec!["observer_specialized_parser_missing"]
        }
    };
    let artifacts = if outputs.is_empty() {
        plan.io.outputs.clone()
    } else {
        outputs.to_vec()
    };
    let semantic_metrics = super::semantic::observed_semantic_metrics(plan, &artifacts);
    let declared_metric_invariants =
        bijux_dna_domain_fastq::stage_metric_invariants(&plan.stage_id).unwrap_or(&[]);

    ObservationContext {
        interpretation_level,
        observer_covered,
        benchmark_scenarios,
        comparison_artifact_ids,
        semantic_loss,
        artifacts,
        semantic_metrics,
        declared_metric_invariants,
    }
}
