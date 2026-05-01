use bijux_dna_core::contract::{ReportSeverity, StageReportContract, StageReportKind};
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{StageEventHintV1, StagePlanV1, StageReportPartV1};

use super::observation_context::ObservationContext;

fn invariant(id: &str, status: InvariantStatusV1, message: String) -> InvariantResultV1 {
    InvariantResultV1 { id: id.to_string(), status, message, remediation: None }
}

pub(super) fn output_invariants(
    missing_expected: &[String],
    context: &ObservationContext,
) -> Vec<InvariantResultV1> {
    let mut invariants = vec![if missing_expected.is_empty() {
        invariant(
            "stage_output_contract_complete",
            InvariantStatusV1::Pass,
            "all declared stage outputs are represented".to_string(),
        )
    } else {
        invariant(
            "stage_output_contract_complete",
            InvariantStatusV1::Warn,
            format!("missing declared output artifacts: {}", missing_expected.join(", ")),
        )
    }];
    invariants.push(if context.observer_covered {
        invariant(
            "observer_parser_coverage",
            InvariantStatusV1::Pass,
            "stage has observer-specialized runtime interpretation".to_string(),
        )
    } else {
        invariant(
            "observer_parser_coverage",
            InvariantStatusV1::Warn,
            "stage uses generic runtime interpretation only".to_string(),
        )
    });
    invariants.push(if context.declared_metric_invariants.is_empty() {
        invariant(
            "declared_metric_invariants_visible",
            InvariantStatusV1::Warn,
            "stage has no declared metric invariants in the FASTQ domain contract".to_string(),
        )
    } else {
        invariant(
            "declared_metric_invariants_visible",
            InvariantStatusV1::Pass,
            format!(
                "stage declares metric invariants: {}",
                context.declared_metric_invariants.join(", ")
            ),
        )
    });
    invariants
}

pub(super) fn output_verdict(
    plan: &StagePlanV1,
    outputs_used: bool,
    invariants: &[InvariantResultV1],
    context: &ObservationContext,
) -> StageVerdictV1 {
    invariants.iter().fold(
        StageVerdictV1 {
            stage_id: plan.stage_id.as_str().to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({
                "artifact_count": context.artifacts.len(),
                "observer_coverage": context.observer_covered,
                "runtime_interpretation": format!("{:?}", context.interpretation_level),
                "benchmark_scenarios": context.benchmark_scenarios
                    .iter()
                    .map(|scenario| scenario.scenario_id.clone())
                    .collect::<Vec<_>>(),
                "semantic_loss": context.semantic_loss,
                "used_observed_outputs": outputs_used,
                "declared_metric_invariants": context.declared_metric_invariants,
                "semantic_metrics": context.semantic_metrics.clone(),
            }),
        },
        |mut verdict, item| {
            verdict.verdict = std::cmp::max(verdict.verdict, item.status.clone());
            verdict.reasons.push(item.message.clone());
            verdict
        },
    )
}

pub(super) fn output_report_parts(
    plan: &StagePlanV1,
    outputs_used: bool,
    context: &ObservationContext,
) -> Vec<StageReportPartV1> {
    let mut report_parts = vec![StageReportPartV1 {
        name: "stage_outputs".to_string(),
        file_name: "stage_outputs.json".to_string(),
        contract: report_contract(
            "fastq.stage_outputs",
            StageReportKind::Qc,
            &["stage_id", "artifact_ids", "used_observed_outputs"],
        ),
        payload: serde_json::json!({
            "stage_id": plan.stage_id,
            "observer_coverage": context.observer_covered,
            "runtime_interpretation": format!("{:?}", context.interpretation_level),
            "benchmark_scenarios": context.benchmark_scenarios
                .iter()
                .map(|scenario| scenario.scenario_id.clone())
                .collect::<Vec<_>>(),
            "comparison_artifact_ids": context.comparison_artifact_ids,
            "semantic_loss": context.semantic_loss,
            "artifact_count": context.artifacts.len(),
            "declared_metric_invariants": context.declared_metric_invariants,
            "semantic_metrics": context.semantic_metrics.clone(),
            "artifact_ids": context.artifacts
                .iter()
                .map(|artifact| artifact.name.as_str().to_string())
                .collect::<Vec<_>>(),
            "used_observed_outputs": outputs_used,
        }),
    }];
    if !context.benchmark_scenarios.is_empty() {
        report_parts.push(StageReportPartV1 {
            name: "stage_tool_comparison".to_string(),
            file_name: "stage_tool_comparison.json".to_string(),
            contract: report_contract(
                "fastq.stage_tool_comparison",
                StageReportKind::PopulationSummary,
                &["stage_id", "tool_id", "benchmark_scenarios"],
            ),
            payload: serde_json::json!({
                "stage_id": plan.stage_id,
                "tool_id": plan.tool_id,
                "runtime_interpretation": format!("{:?}", context.interpretation_level),
                "comparison_artifact_ids": context.comparison_artifact_ids,
                "semantic_loss": context.semantic_loss,
                "benchmark_scenarios": context.benchmark_scenarios
                    .iter()
                    .map(|scenario| serde_json::json!({
                        "scenario_id": scenario.scenario_id,
                        "description": scenario.description,
                        "fairness_rules": scenario.fairness_rules,
                    }))
                    .collect::<Vec<_>>(),
                "normalized_artifact_ids": context.artifacts
                    .iter()
                    .map(|artifact| artifact.name.as_str().to_string())
                    .collect::<Vec<_>>(),
                "observer_specialized_parser": context.observer_covered,
                "semantic_metrics": context.semantic_metrics.clone(),
            }),
        });
    }
    if !context.semantic_metrics.is_null() {
        report_parts.push(StageReportPartV1 {
            name: "observed_semantic_metrics".to_string(),
            file_name: "observed_semantic_metrics.json".to_string(),
            contract: report_contract(
                "fastq.observed_semantic_metrics",
                StageReportKind::Qc,
                &["stage_id", "tool_id", "semantic_metrics"],
            ),
            payload: serde_json::json!({
                "stage_id": plan.stage_id,
                "tool_id": plan.tool_id,
                "semantic_metrics": context.semantic_metrics.clone(),
            }),
        });
    }
    report_parts
}

fn report_contract(
    report_id: &str,
    kind: StageReportKind,
    required_fields: &[&str],
) -> StageReportContract {
    StageReportContract {
        report_id: report_id.to_string(),
        kind,
        schema_version: "bijux.stage_report.v1".to_string(),
        required_fields: required_fields.iter().map(|field| (*field).to_string()).collect(),
        advisory_fields: vec!["semantic_loss".to_string()],
        severity: ReportSeverity::Warning,
    }
}

pub(super) fn output_warnings(plan: &StagePlanV1, context: &ObservationContext) -> Vec<String> {
    let mut warnings = if context.observer_covered {
        Vec::new()
    } else {
        vec![format!(
            "{} has no observer-specialized parser; emitting metrics envelope from the stage plan only",
            plan.stage_id.as_str()
        )]
    };
    if !context.benchmark_scenarios.is_empty() && !context.semantic_loss.is_empty() {
        warnings.push(format!(
            "{} comparison record carries semantic loss tags: {}",
            plan.stage_id.as_str(),
            context.semantic_loss.join(", ")
        ));
    }
    warnings
}

pub(super) fn output_event_hints(
    plan: &StagePlanV1,
    outputs_used: bool,
    context: &ObservationContext,
) -> Vec<StageEventHintV1> {
    vec![StageEventHintV1 {
        event_name: "stage_outputs_parsed".to_string(),
        status: if outputs_used { "observed".to_string() } else { "expected_only".to_string() },
        attrs: serde_json::json!({
            "stage_id": plan.stage_id,
            "observer_coverage": context.observer_covered,
            "runtime_interpretation": format!("{:?}", context.interpretation_level),
            "benchmark_scenarios": context.benchmark_scenarios
                .iter()
                .map(|scenario| scenario.scenario_id.clone())
                .collect::<Vec<_>>(),
            "semantic_loss": context.semantic_loss,
            "artifact_count": context.artifacts.len(),
            "semantic_metrics": context.semantic_metrics.clone(),
        }),
    }]
}
