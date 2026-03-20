use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};

use crate::metrics;

#[allow(dead_code)]
pub struct FastqStagePlugin;

impl StagePlugin for FastqStagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with(id_catalog::FASTQ_PREFIX)
    }

    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1> {
        Ok(StageInvocationV1 {
            command: plan.command.template.clone(),
            env: std::collections::BTreeMap::new(),
            expected_outputs: plan.io.outputs.clone(),
        })
    }

    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1> {
        let invariant = |id: &str, status: InvariantStatusV1, message: String| InvariantResultV1 {
            id: id.to_string(),
            status,
            message,
            remediation: None,
        };
        let input_paths: Vec<std::path::PathBuf> = plan
            .io
            .inputs
            .iter()
            .map(|input| input.path.clone())
            .collect();
        let output_paths: Vec<std::path::PathBuf> = plan
            .io
            .outputs
            .iter()
            .map(|output| output.path.clone())
            .collect();
        let envelope = metrics::build_metrics_envelope(plan, &input_paths, &output_paths)?;
        let interpretation_level =
            crate::runtime_interpretation_for_stage_tool(&plan.stage_id, &plan.tool_id)
                .unwrap_or(crate::RuntimeInterpretationLevel::GenericEnvelope);
        let observer_covered =
            interpretation_level == crate::RuntimeInterpretationLevel::ObserverSpecialized;
        let benchmark_scenarios =
            bijux_dna_domain_fastq::benchmark_scenarios_for_stage(&plan.stage_id);
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
        let expected_artifact_names = plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let actual_artifact_names = artifacts
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let missing_expected = expected_artifact_names
            .difference(&actual_artifact_names)
            .cloned()
            .collect::<Vec<_>>();
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
                format!(
                    "missing declared output artifacts: {}",
                    missing_expected.join(", ")
                ),
            )
        }];
        invariants.push(if observer_covered {
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
        let declared_metric_invariants =
            bijux_dna_domain_fastq::stage_metric_invariants(&plan.stage_id).unwrap_or(&[]);
        invariants.push(if declared_metric_invariants.is_empty() {
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
                    declared_metric_invariants.join(", ")
                ),
            )
        });
        let verdict = invariants.iter().fold(
            StageVerdictV1 {
                stage_id: plan.stage_id.as_str().to_string(),
                verdict: InvariantStatusV1::Pass,
                reasons: Vec::new(),
                key_metrics: serde_json::json!({
                    "artifact_count": artifacts.len(),
                    "observer_coverage": observer_covered,
                    "runtime_interpretation": format!("{interpretation_level:?}"),
                    "benchmark_scenarios": benchmark_scenarios
                        .iter()
                        .map(|scenario| scenario.scenario_id.clone())
                        .collect::<Vec<_>>(),
                    "semantic_loss": semantic_loss,
                    "used_observed_outputs": !outputs.is_empty(),
                    "declared_metric_invariants": declared_metric_invariants,
                }),
            },
            |mut verdict, item| {
                verdict.verdict = std::cmp::max(verdict.verdict, item.status.clone());
                verdict.reasons.push(item.message.clone());
                verdict
            },
        );
        let mut report_parts = vec![StageReportPartV1 {
            name: "stage_outputs".to_string(),
            file_name: "stage_outputs.json".to_string(),
            payload: serde_json::json!({
                "stage_id": plan.stage_id,
                "observer_coverage": observer_covered,
                "runtime_interpretation": format!("{interpretation_level:?}"),
                "benchmark_scenarios": benchmark_scenarios
                    .iter()
                    .map(|scenario| scenario.scenario_id.clone())
                    .collect::<Vec<_>>(),
                "comparison_artifact_ids": comparison_artifact_ids,
                "semantic_loss": semantic_loss,
                "artifact_count": artifacts.len(),
                "declared_metric_invariants": declared_metric_invariants,
                "artifact_ids": artifacts
                    .iter()
                    .map(|artifact| artifact.name.as_str().to_string())
                    .collect::<Vec<_>>(),
                "used_observed_outputs": !outputs.is_empty(),
            }),
        }];
        if !benchmark_scenarios.is_empty() {
            report_parts.push(StageReportPartV1 {
                name: "stage_tool_comparison".to_string(),
                file_name: "stage_tool_comparison.json".to_string(),
                payload: serde_json::json!({
                    "stage_id": plan.stage_id,
                    "tool_id": plan.tool_id,
                    "runtime_interpretation": format!("{interpretation_level:?}"),
                    "comparison_artifact_ids": comparison_artifact_ids,
                    "semantic_loss": semantic_loss,
                    "benchmark_scenarios": benchmark_scenarios
                        .iter()
                        .map(|scenario| serde_json::json!({
                            "scenario_id": scenario.scenario_id,
                            "description": scenario.description,
                            "fairness_rules": scenario.fairness_rules,
                        }))
                        .collect::<Vec<_>>(),
                    "normalized_artifact_ids": artifacts
                        .iter()
                        .map(|artifact| artifact.name.as_str().to_string())
                        .collect::<Vec<_>>(),
                    "observer_specialized_parser": observer_covered,
                }),
            });
        }
        let mut warnings = if observer_covered {
            Vec::new()
        } else {
            vec![format!(
                "{} has no observer-specialized parser; emitting metrics envelope from the stage plan only",
                plan.stage_id.as_str()
            )]
        };
        if !benchmark_scenarios.is_empty() && !semantic_loss.is_empty() {
            warnings.push(format!(
                "{} comparison record carries semantic loss tags: {}",
                plan.stage_id.as_str(),
                semantic_loss.join(", ")
            ));
        }
        let event_hints = vec![StageEventHintV1 {
            event_name: "stage_outputs_parsed".to_string(),
            status: if outputs.is_empty() {
                "expected_only".to_string()
            } else {
                "observed".to_string()
            },
            attrs: serde_json::json!({
                "stage_id": plan.stage_id,
                "observer_coverage": observer_covered,
                "runtime_interpretation": format!("{interpretation_level:?}"),
                "benchmark_scenarios": benchmark_scenarios
                    .iter()
                    .map(|scenario| scenario.scenario_id.clone())
                    .collect::<Vec<_>>(),
                "semantic_loss": semantic_loss,
                "artifact_count": artifacts.len(),
            }),
        }];
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts,
            report_parts,
            warnings,
            invariants,
            verdict: Some(verdict),
            event_hints,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
    use bijux_dna_core::ids::*;
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlugin};

    use super::FastqStagePlugin;

    fn plan(stage_id: &'static str) -> bijux_dna_stage_contract::StagePlanV1 {
        bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static(stage_id),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("fastqc"),
            tool_version: "test".to_string(),
            image: serde_json::from_value(serde_json::json!({
                "image": "bijuxdna/test",
                "digest": null,
            }))
            .expect("image"),
            command: serde_json::from_value(serde_json::json!({
                "template": ["echo", "ok"],
            }))
            .expect("command"),
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    PathBuf::from("reads.fastq.gz"),
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    PathBuf::from("report.json"),
                    ArtifactRole::ReportJson,
                )],
            },
            out_dir: PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            aux_images: std::collections::BTreeMap::new(),
            reason: PlanDecisionReason::default(),
        }
    }

    #[test]
    fn parse_outputs_emits_artifacts_report_parts_and_event_hints() {
        let plugin = FastqStagePlugin;
        let plan = plan("fastq.detect_adapters");
        let output = plugin
            .parse_outputs(&plan, &plan.io.outputs)
            .expect("parse outputs");
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output.report_parts.len(), 2);
        assert_eq!(output.event_hints.len(), 1);
        assert!(output.warnings.is_empty());
        assert_eq!(output.invariants.len(), 3);
        assert_eq!(
            output.report_parts[0].payload["runtime_interpretation"],
            serde_json::json!("ObserverSpecialized")
        );
        assert_eq!(
            output.report_parts[1].payload["benchmark_scenarios"][0]["scenario_id"],
            serde_json::json!("detect_adapters_fairness")
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .map(|verdict| verdict.verdict.clone()),
            Some(bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass)
        );
    }

    #[test]
    fn parse_outputs_warns_when_no_observer_parser_exists() {
        let plugin = FastqStagePlugin;
        let plan = plan("fastq.trim_reads");
        let output = plugin
            .parse_outputs(&plan, &plan.io.outputs)
            .expect("parse outputs");
        assert_eq!(output.artifacts.len(), 1);
        assert_eq!(output.report_parts.len(), 2);
        assert_eq!(output.warnings.len(), 2);
        assert_eq!(
            output.report_parts[0].payload["runtime_interpretation"],
            serde_json::json!("GenericEnvelope")
        );
        assert_eq!(
            output.report_parts[1].payload["comparison_artifact_ids"],
            serde_json::json!([
                "trim_tool_benchmark_cohort_json",
                "trim_tool_comparison_json",
                "trim_tool_normalization_json"
            ])
        );
        assert_eq!(
            output.report_parts[1].payload["benchmark_scenarios"][0]["scenario_id"],
            serde_json::json!("trim_fairness")
        );
        assert_eq!(
            output.report_parts[1].payload["semantic_loss"],
            serde_json::json!(["observer_specialized_parser_missing"])
        );
        assert!(output.warnings[0].contains("fastq.trim_reads"));
        assert!(output.warnings[1].contains("semantic loss tags"));
        assert_eq!(output.invariants.len(), 3);
        assert_eq!(
            output
                .verdict
                .as_ref()
                .map(|verdict| verdict.verdict.clone()),
            Some(bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn)
        );
    }
}
