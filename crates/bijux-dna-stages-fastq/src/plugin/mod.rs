use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};
use std::fs;

use crate::metrics;
use crate::observer::{
    parse_bbduk_reads_removed, parse_cluster_otus_report, parse_correct_errors_report,
    parse_deduplicate_report, parse_deplete_host_report,
    parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
    parse_detect_adapters_report, parse_extract_umis_report, parse_fastp_metrics,
    parse_filter_low_complexity_report, parse_filter_reads_report, parse_index_reference_report,
    parse_infer_asvs_report, parse_merge_pairs_report, parse_multiqc_general_stats_metrics,
    parse_normalize_abundance_report, parse_normalize_primers_report,
    parse_profile_overrepresented_report, parse_profile_read_lengths_report,
    parse_profile_reads_report, parse_remove_chimeras_report, parse_remove_duplicates_provenance,
    parse_remove_duplicates_report, parse_report_qc_report, parse_screen_taxonomy_report,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validated_reads_manifest, parse_validation_report,
};

mod observation_context;
mod semantic;

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
        let context = observation_context::observation_context(plan, outputs);
        let expected_artifact_names = plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let actual_artifact_names = context
            .artifacts
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
        let verdict = invariants.iter().fold(
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
                    "used_observed_outputs": !outputs.is_empty(),
                    "declared_metric_invariants": context.declared_metric_invariants,
                    "semantic_metrics": context.semantic_metrics.clone(),
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
                "used_observed_outputs": !outputs.is_empty(),
            }),
        }];
        if !context.benchmark_scenarios.is_empty() {
            report_parts.push(StageReportPartV1 {
                name: "stage_tool_comparison".to_string(),
                file_name: "stage_tool_comparison.json".to_string(),
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
                payload: serde_json::json!({
                    "stage_id": plan.stage_id,
                    "tool_id": plan.tool_id,
                    "semantic_metrics": context.semantic_metrics.clone(),
                }),
            });
        }
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
        let event_hints = vec![StageEventHintV1 {
            event_name: "stage_outputs_parsed".to_string(),
            status: if outputs.is_empty() {
                "expected_only".to_string()
            } else {
                "observed".to_string()
            },
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
                "semantic_metrics": context.semantic_metrics,
            }),
        }];
        Ok(StagePluginOutputV1 {
            metrics: envelope,
            artifacts: context.artifacts,
            report_parts,
            warnings,
            invariants,
            verdict: Some(verdict),
            event_hints,
        })
    }
}

#[cfg(test)]
mod plugin_contracts;
