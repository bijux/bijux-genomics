use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};

use crate::metrics;
use crate::observer::parse_deduplicate_report;

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
        let semantic_metrics = observed_semantic_metrics(plan, &artifacts);
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
                    "semantic_metrics": semantic_metrics.clone(),
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
                "semantic_metrics": semantic_metrics.clone(),
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
                    "semantic_metrics": semantic_metrics.clone(),
                }),
            });
        }
        if !semantic_metrics.is_null() {
            report_parts.push(StageReportPartV1 {
                name: "observed_semantic_metrics".to_string(),
                file_name: "observed_semantic_metrics.json".to_string(),
                payload: serde_json::json!({
                    "stage_id": plan.stage_id,
                    "tool_id": plan.tool_id,
                    "semantic_metrics": semantic_metrics.clone(),
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
                    "semantic_metrics": semantic_metrics,
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

#[allow(dead_code)]
fn observed_semantic_metrics(plan: &StagePlanV1, artifacts: &[ArtifactRef]) -> serde_json::Value {
    if plan.stage_id.as_str() == "fastq.report_qc" {
        if let Some(manifest_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_manifest) = std::fs::read_to_string(manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw_manifest) {
                    let mut contributor_stage_ids = manifest
                        .get("qc_inputs")
                        .and_then(serde_json::Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|entry| entry.get("name").and_then(serde_json::Value::as_str))
                        .filter_map(parse_qc_contributor_identity)
                        .map(|(stage_id, _tool_id)| stage_id)
                        .collect::<Vec<_>>();
                    contributor_stage_ids.sort();
                    contributor_stage_ids.dedup();
                    let mut contributor_tool_ids = manifest
                        .get("qc_inputs")
                        .and_then(serde_json::Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(|entry| entry.get("name").and_then(serde_json::Value::as_str))
                        .filter_map(parse_qc_contributor_identity)
                        .map(|(_stage_id, tool_id)| tool_id)
                        .collect::<Vec<_>>();
                    contributor_tool_ids.sort();
                    contributor_tool_ids.dedup();
                    let contributor_count = manifest
                        .get("qc_inputs")
                        .and_then(serde_json::Value::as_array)
                        .map_or(0, std::vec::Vec::len);
                    return serde_json::json!({
                        "lineage_hash": manifest.get("lineage_hash").cloned().unwrap_or(serde_json::Value::Null),
                        "contributor_artifact_count": contributor_count,
                        "contributor_stage_ids": contributor_stage_ids,
                        "contributor_tool_ids": contributor_tool_ids,
                        "raw_fastqc_dir": manifest.get("raw_fastqc_dir").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.validate_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "validation_report")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                if let Ok(report) = serde_json::from_str::<serde_json::Value>(&raw_report) {
                    return serde_json::json!({
                        "validation_mode": report.get("validation_mode").cloned().unwrap_or(serde_json::Value::Null),
                        "pair_sync_policy": report.get("pair_sync_policy").cloned().unwrap_or(serde_json::Value::Null),
                        "strict_pass": report.get("strict_pass").cloned().unwrap_or(serde_json::Value::Null),
                        "exit_code": report.get("exit_code").cloned().unwrap_or(serde_json::Value::Null),
                        "validated_inputs": report.get("validated_inputs").cloned().unwrap_or(serde_json::Value::Null),
                        "pair_sync_checked": report.get("pair_sync_checked").cloned().unwrap_or(serde_json::Value::Null),
                        "pair_sync_pass": report.get("pair_sync_pass").cloned().unwrap_or(serde_json::Value::Null),
                        "validated_pairs": report.get("validated_pairs").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.remove_duplicates" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                let parsed: Result<(u64, u64), _> = parse_deduplicate_report(&raw_report);
                if let Ok((reads_in, reads_out)) = parsed {
                    let duplicates_removed = reads_in.saturating_sub(reads_out);
                    let dedup_rate = if reads_in > 0 {
                        duplicates_removed as f64 / reads_in as f64
                    } else {
                        0.0
                    };
                    return serde_json::json!({
                        "reads_in": reads_in,
                        "reads_out": reads_out,
                        "duplicates_removed": duplicates_removed,
                        "dedup_rate": dedup_rate,
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_terminal_damage" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                if let Ok(report) = serde_json::from_str::<serde_json::Value>(&raw_report) {
                    return serde_json::json!({
                        "damage_mode": report.get("damage_mode").cloned().unwrap_or(serde_json::Value::Null),
                        "execution_policy": report.get("execution_policy").cloned().unwrap_or(serde_json::Value::Null),
                        "trim_5p_bases": report.get("trim_5p_bases").cloned().unwrap_or(serde_json::Value::Null),
                        "trim_3p_bases": report.get("trim_3p_bases").cloned().unwrap_or(serde_json::Value::Null),
                        "requested_trim_5p_bases": report.get("requested_trim_5p_bases").cloned().unwrap_or(serde_json::Value::Null),
                        "requested_trim_3p_bases": report.get("requested_trim_3p_bases").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_polyg_tails" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                if let Ok(report) = serde_json::from_str::<serde_json::Value>(&raw_report) {
                    return serde_json::json!({
                        "trim_polyg": report.get("trim_polyg").cloned().unwrap_or(serde_json::Value::Null),
                        "min_polyg_run": report.get("min_polyg_run").cloned().unwrap_or(serde_json::Value::Null),
                        "raw_report_path": report.get("raw_report_path").cloned().unwrap_or(serde_json::Value::Null),
                        "raw_report_format": report.get("raw_report_format").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }
    serde_json::Value::Null
}

fn parse_qc_contributor_identity(name: &str) -> Option<(String, String)> {
    let mut parts = name.split('.');
    let domain = parts.next()?;
    let stage = parts.next()?;
    let tool = parts.next()?;
    Some((format!("{domain}.{stage}"), tool.to_string()))
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

    #[test]
    fn parse_outputs_surfaces_observed_deduplicate_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_path = temp.path().join("reads.fastq");
        let dedup_reads_path = temp.path().join("dedup.fastq");
        std::fs::write(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
        std::fs::write(&dedup_reads_path, b"@r1\nACGT\n+\n####\n").expect("write dedup reads");
        let report_path = temp.path().join("deduplicate_report.json");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "reads_in": 12_u64,
                "reads_out": 9_u64
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.remove_duplicates"),
            tool_id: ToolId::from_static("clumpify"),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_path,
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("dedup_reads_r1"),
                    dedup_reads_path,
                    ArtifactRole::Reads,
                )],
            },
            ..plan("fastq.remove_duplicates")
        };

        let output = plugin
            .parse_outputs(
                &plan,
                &[
                    plan.io.outputs[0].clone(),
                    ArtifactRef::required(
                        ArtifactId::new("report_json"),
                        report_path.clone(),
                        ArtifactRole::ReportJson,
                    ),
                ],
            )
            .expect("parse outputs");

        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["reads_in"],
            serde_json::json!(12_u64)
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["duplicates_removed"],
            serde_json::json!(3_u64)
        );
        assert!(output
            .report_parts
            .iter()
            .any(|part| part.name == "observed_semantic_metrics"));
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["dedup_rate"],
            serde_json::json!(0.25)
        );
    }

    #[test]
    fn parse_outputs_surfaces_observed_validation_semantics_for_seqtk() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_path = temp.path().join("reads.fastq");
        let report_path = temp.path().join("validation_report.json");
        std::fs::write(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.validate.report.v1",
                "validation_mode": "strict",
                "pair_sync_policy": "require_header_sync",
                "strict_pass": true,
                "exit_code": 0,
                "validated_inputs": 2_u64,
                "pair_sync_checked": true,
                "pair_sync_pass": true,
                "validated_pairs": 1_u64
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.validate_reads"),
            tool_id: ToolId::from_static("seqtk"),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_path,
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("validation_report"),
                    report_path.clone(),
                    ArtifactRole::SummaryJson,
                )],
            },
            ..plan("fastq.validate_reads")
        };

        let output = plugin
            .parse_outputs(&plan, &plan.io.outputs)
            .expect("parse outputs");

        assert!(output.warnings.is_empty());
        assert_eq!(
            output.report_parts[0].payload["runtime_interpretation"],
            serde_json::json!("ObserverSpecialized")
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["validated_pairs"],
            serde_json::json!(1_u64)
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["validation_mode"],
            serde_json::json!("strict")
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["pair_sync_pass"],
            serde_json::json!(true)
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["pair_sync_policy"],
            serde_json::json!("require_header_sync")
        );
    }

    #[test]
    fn parse_outputs_surfaces_terminal_damage_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_path = temp.path().join("reads.fastq");
        let trimmed_reads_path = temp.path().join("trimmed.fastq");
        let report_path = temp.path().join("trim_terminal_damage_report.json");
        std::fs::write(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
        std::fs::write(&trimmed_reads_path, b"@r1\nCG\n+\n##\n").expect("write trimmed reads");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2_u64,
                "trim_3p_bases": 1_u64,
                "requested_trim_5p_bases": 2_u64,
                "requested_trim_3p_bases": 1_u64
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_terminal_damage"),
            tool_id: ToolId::from_static("cutadapt"),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_path,
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("trimmed_reads_r1"),
                    trimmed_reads_path,
                    ArtifactRole::Reads,
                )],
            },
            ..plan("fastq.trim_terminal_damage")
        };

        let output = plugin
            .parse_outputs(
                &plan,
                &[
                    plan.io.outputs[0].clone(),
                    ArtifactRef::required(
                        ArtifactId::new("report_json"),
                        report_path.clone(),
                        ArtifactRole::ReportJson,
                    ),
                ],
            )
            .expect("parse outputs");

        assert!(output.warnings.is_empty());
        assert_eq!(
            output.report_parts[0].payload["runtime_interpretation"],
            serde_json::json!("ObserverSpecialized")
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["execution_policy"],
            serde_json::json!("explicit_terminal_trim")
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["trim_5p_bases"],
            serde_json::json!(2_u64)
        );
    }

    #[test]
    fn parse_outputs_surfaces_polyg_trim_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_path = temp.path().join("reads.fastq");
        let trimmed_reads_path = temp.path().join("trimmed.fastq");
        let report_path = temp.path().join("trim_polyg_tails_report.json");
        std::fs::write(&reads_path, b"@r1\nACGTGGGG\n+\n########\n").expect("write reads");
        std::fs::write(&trimmed_reads_path, b"@r1\nACGT\n+\n####\n").expect("write trimmed reads");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "trim_polyg": true,
                "min_polyg_run": 10_u64,
                "raw_report_path": "/tmp/trim_polyg.fastp.json",
                "raw_report_format": "fastp_json"
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_polyg_tails"),
            tool_id: ToolId::from_static("fastp"),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_path,
                    ArtifactRole::Reads,
                )],
                outputs: vec![ArtifactRef::required(
                    ArtifactId::new("trimmed_reads_r1"),
                    trimmed_reads_path,
                    ArtifactRole::Reads,
                )],
            },
            ..plan("fastq.trim_polyg_tails")
        };

        let output = plugin
            .parse_outputs(
                &plan,
                &[
                    plan.io.outputs[0].clone(),
                    ArtifactRef::required(
                        ArtifactId::new("report_json"),
                        report_path.clone(),
                        ArtifactRole::ReportJson,
                    ),
                ],
            )
            .expect("parse outputs");

        assert!(output.warnings.is_empty());
        assert_eq!(
            output.report_parts[0].payload["runtime_interpretation"],
            serde_json::json!("ObserverSpecialized")
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["min_polyg_run"],
            serde_json::json!(10_u64)
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["raw_report_format"],
            serde_json::json!("fastp_json")
        );
    }

    #[test]
    fn parse_outputs_surfaces_qc_contributor_lineage_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let qc_input_path = temp.path().join("qc_input.fastq");
        let report_path = temp.path().join("multiqc_report.html");
        let data_dir = temp.path().join("multiqc_data");
        let manifest_path = temp.path().join("governed_qc_inputs_manifest.json");
        std::fs::write(&qc_input_path, b"@r1\nACGT\n+\n####\n").expect("write qc input");
        std::fs::write(
            &manifest_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.inputs.v1",
                "lineage_hash": "fastq.trim_reads.fastp=report_json",
                "raw_fastqc_dir": "/tmp/raw_fastqc",
                "qc_inputs": [
                    {
                        "name": "fastq.trim_reads.fastp.report_json",
                        "path": "/tmp/fastp/report.json",
                        "role": "report_json"
                    },
                    {
                        "name": "fastq.validate_reads.fastqvalidator.validation_report",
                        "path": "/tmp/validate/report.json",
                        "role": "validation_report"
                    }
                ]
            })
            .to_string(),
        )
        .expect("write manifest");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.report_qc"),
            tool_id: ToolId::from_static("multiqc"),
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("qc_artifacts"),
                    qc_input_path,
                    ArtifactRole::Reads,
                )],
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::new("multiqc_report"),
                        report_path,
                        ArtifactRole::ReportHtml,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("multiqc_data"),
                        data_dir,
                        ArtifactRole::Unknown,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("governed_qc_inputs_manifest"),
                        manifest_path.clone(),
                        ArtifactRole::SummaryJson,
                    ),
                ],
            },
            ..plan("fastq.report_qc")
        };

        let output = plugin
            .parse_outputs(
                &plan,
                &[plan.io.outputs[2].clone()],
            )
            .expect("parse outputs");

        assert!(output.warnings.is_empty());
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["contributor_artifact_count"],
            serde_json::json!(2)
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["contributor_stage_ids"],
            serde_json::json!(["fastq.trim_reads", "fastq.validate_reads"])
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["contributor_tool_ids"],
            serde_json::json!(["fastp", "fastqvalidator"])
        );
        assert_eq!(
            output
                .verdict
                .as_ref()
                .expect("verdict")
                .key_metrics["semantic_metrics"]["lineage_hash"],
            serde_json::json!("fastq.trim_reads.fastp=report_json")
        );
    }
}
