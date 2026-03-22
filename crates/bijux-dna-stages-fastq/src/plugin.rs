use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};
use bijux_dna_stage_contract::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};

use crate::metrics;
use crate::observer::{
    parse_bbduk_reads_removed, parse_deduplicate_report, parse_fastp_metrics,
    parse_multiqc_general_stats_metrics, parse_terminal_damage_report, parse_trim_polyg_report,
    parse_trim_reads_report, parse_validated_reads_manifest, parse_validation_report,
};

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
        let multiqc_metrics = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "multiqc_data")
            .map(|artifact| artifact.path.join("multiqc_general_stats.json"))
            .filter(|path| path.exists())
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|raw| parse_multiqc_general_stats_metrics(&raw).ok());
        if let Some(manifest_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_manifest) = std::fs::read_to_string(manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw_manifest) {
                    let contributor_entries = manifest
                        .get("contributors")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut contributor_stage_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(stage_id, _tool_id)| stage_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("stage_id")
                                    .and_then(serde_json::Value::as_str)
                                    .map(ToString::to_string)
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_stage_ids.sort();
                    contributor_stage_ids.dedup();
                    let mut contributor_tool_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(_stage_id, tool_id)| tool_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("contributor_id")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(|contributor_id| {
                                        contributor_id
                                            .rsplit_once('.')
                                            .map(|(_, tool_id)| tool_id.to_string())
                                    })
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_tool_ids.sort();
                    contributor_tool_ids.dedup();
                    let contributor_count = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .map_or(0, std::vec::Vec::len)
                    } else {
                        contributor_entries.len()
                    };
                    return serde_json::json!({
                        "lineage_hash": manifest.get("lineage_hash").cloned().unwrap_or(serde_json::Value::Null),
                        "contributor_artifact_count": contributor_count,
                        "contributor_stage_ids": contributor_stage_ids,
                        "contributor_tool_ids": contributor_tool_ids,
                        "raw_fastqc_dir": manifest.get("raw_fastqc_dir").cloned().unwrap_or(serde_json::Value::Null),
                        "multiqc_sample_count": multiqc_metrics.as_ref().map(|metrics| metrics.sample_count),
                        "multiqc_module_count": multiqc_metrics.as_ref().map(|metrics| metrics.module_count),
                    });
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.validate_reads" {
        if let Some(semantics) = validate_semantic_metrics(artifacts) {
            return semantics;
        }
    }
    if plan.stage_id.as_str() == "fastq.trim_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                if let Ok(report) = parse_trim_reads_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        (
                            "min_length".to_string(),
                            serde_json::json!(report.min_length),
                        ),
                        (
                            "quality_cutoff".to_string(),
                            serde_json::json!(report.quality_cutoff),
                        ),
                        (
                            "adapter_policy".to_string(),
                            serde_json::json!(report.adapter_policy),
                        ),
                        (
                            "polyx_policy".to_string(),
                            serde_json::json!(report.polyx_policy),
                        ),
                        ("n_policy".to_string(), serde_json::json!(report.n_policy)),
                        (
                            "contaminant_policy".to_string(),
                            serde_json::json!(report.contaminant_policy),
                        ),
                        (
                            "adapter_bank_id".to_string(),
                            serde_json::json!(report.adapter_bank_id),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "contaminant_bank_id".to_string(),
                            serde_json::json!(report.contaminant_bank_id),
                        ),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        (
                            "mean_q_before".to_string(),
                            serde_json::json!(report.mean_q_before),
                        ),
                        (
                            "mean_q_after".to_string(),
                            serde_json::json!(report.mean_q_after),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_payload) = std::fs::read_to_string(raw_backend_report)
                        {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_payload) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_payload)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return serde_json::Value::Object(semantics);
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
                if let Ok(report) = parse_terminal_damage_report(&raw_report) {
                    return serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "damage_mode": report.damage_mode,
                        "execution_policy": report.execution_policy,
                        "trim_5p_bases": report.trim_5p_bases,
                        "trim_3p_bases": report.trim_3p_bases,
                        "requested_trim_5p_bases": report.requested_trim_5p_bases,
                        "requested_trim_3p_bases": report.requested_trim_3p_bases,
                        "udg_classification": report.udg_classification,
                        "ct_ga_asymmetry_pre": report.ct_ga_asymmetry_pre,
                        "ct_ga_asymmetry_post": report.ct_ga_asymmetry_post,
                        "raw_backend_report_format": report.raw_backend_report_format,
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
                if let Ok(report) = parse_trim_polyg_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        (
                            "paired_mode".to_string(),
                            serde_json::json!(report.paired_mode),
                        ),
                        (
                            "trim_polyg".to_string(),
                            serde_json::json!(report.trim_polyg),
                        ),
                        (
                            "min_polyg_run".to_string(),
                            serde_json::json!(report.min_polyg_run),
                        ),
                        (
                            "bases_trimmed_polyg".to_string(),
                            serde_json::json!(report.bases_trimmed_polyg),
                        ),
                        (
                            "polyx_bank_id".to_string(),
                            serde_json::json!(report.polyx_bank_id),
                        ),
                        (
                            "polyx_bank_hash".to_string(),
                            serde_json::json!(report.polyx_bank_hash),
                        ),
                        (
                            "polyx_preset".to_string(),
                            serde_json::json!(report.polyx_preset),
                        ),
                        (
                            "raw_backend_report".to_string(),
                            serde_json::json!(report.raw_backend_report),
                        ),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) = report
                        .backend_metrics
                        .as_ref()
                        .and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            if metric_name == "schema_version" {
                                continue;
                            }
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                        return serde_json::Value::Object(semantics);
                    }
                    if let (Some(raw_backend_report), Some(raw_backend_report_format)) = (
                        report.raw_backend_report.as_deref(),
                        report.raw_backend_report_format.as_deref(),
                    ) {
                        if let Ok(raw_backend_report) = std::fs::read_to_string(raw_backend_report)
                        {
                            match raw_backend_report_format {
                                "fastp_json" => {
                                    if let Ok(metrics) = parse_fastp_metrics(&raw_backend_report) {
                                        semantics.insert(
                                            "passed_filter_reads".to_string(),
                                            serde_json::json!(metrics.passed_filter_reads),
                                        );
                                        semantics.insert(
                                            "low_quality_reads".to_string(),
                                            serde_json::json!(metrics.low_quality_reads),
                                        );
                                        semantics.insert(
                                            "too_many_n_reads".to_string(),
                                            serde_json::json!(metrics.too_many_n_reads),
                                        );
                                        semantics.insert(
                                            "too_short_reads".to_string(),
                                            serde_json::json!(metrics.too_short_reads),
                                        );
                                    }
                                }
                                "bbduk_stats" => {
                                    if let Ok(reads_removed) =
                                        parse_bbduk_reads_removed(&raw_backend_report)
                                    {
                                        semantics.insert(
                                            "reads_removed".to_string(),
                                            serde_json::json!(reads_removed),
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    return serde_json::Value::Object(semantics);
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.correct_errors" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = std::fs::read_to_string(report_path) {
                if let Ok(report) = serde_json::from_str::<serde_json::Value>(&raw_report) {
                    return serde_json::json!({
                        "correction_engine": report.get("correction_engine").cloned().unwrap_or(serde_json::Value::Null),
                        "quality_encoding": report.get("quality_encoding").cloned().unwrap_or(serde_json::Value::Null),
                        "kmer_size": report.get("kmer_size").cloned().unwrap_or(serde_json::Value::Null),
                        "genome_size": report.get("genome_size").cloned().unwrap_or(serde_json::Value::Null),
                        "max_memory_gb": report.get("max_memory_gb").cloned().unwrap_or(serde_json::Value::Null),
                        "trusted_kmer_artifact": report.get("trusted_kmer_artifact").cloned().unwrap_or(serde_json::Value::Null),
                        "conservative_mode": report.get("conservative_mode").cloned().unwrap_or(serde_json::Value::Null),
                        "kmer_fix_rate": report.get("kmer_fix_rate").cloned().unwrap_or(serde_json::Value::Null),
                        "correction_effect": report.get("correction_effect").cloned().unwrap_or(serde_json::Value::Null),
                        "input_r1": report.get("input_r1").cloned().unwrap_or(serde_json::Value::Null),
                        "input_r2": report.get("input_r2").cloned().unwrap_or(serde_json::Value::Null),
                        "output_r1": report.get("output_r1").cloned().unwrap_or(serde_json::Value::Null),
                        "output_r2": report.get("output_r2").cloned().unwrap_or(serde_json::Value::Null),
                    });
                }
            }
        }
    }
    serde_json::Value::Null
}

fn validate_semantic_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let report = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validation_report")
        .map(|artifact| artifact.path.as_path())
        .and_then(|report_path| {
            std::fs::read_to_string(report_path)
                .ok()
                .and_then(|raw_report| parse_validation_report(&raw_report).ok())
        });
    let manifest = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validated_reads_manifest")
        .map(|artifact| artifact.path.as_path())
        .and_then(|manifest_path| {
            std::fs::read_to_string(manifest_path)
                .ok()
                .and_then(|raw_manifest| parse_validated_reads_manifest(&raw_manifest).ok())
        });
    if report.is_none() && manifest.is_none() {
        return None;
    }
    Some(serde_json::json!({
        "tool_id": report.as_ref().map(|value| value.tool_id.clone()).or_else(|| manifest.as_ref().map(|value| value.tool_id.clone())),
        "validation_mode": report.as_ref().map(|value| serde_json::to_value(&value.validation_mode).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "pair_sync_policy": report.as_ref().map(|value| serde_json::to_value(&value.pair_sync_policy).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "failure_class": report.as_ref().map(|value| serde_json::to_value(&value.failure_class).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "strict_pass": report.as_ref().map(|value| serde_json::json!(value.strict_pass)).unwrap_or(serde_json::Value::Null),
        "exit_code": report.as_ref().map(|value| serde_json::json!(value.exit_code)).unwrap_or(serde_json::Value::Null),
        "validated_inputs": report.as_ref().map(|value| serde_json::json!(value.validated_inputs)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r1": report.as_ref().map(|value| serde_json::json!(value.validated_reads_r1)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r2": report.as_ref().map(|value| serde_json::to_value(value.validated_reads_r2).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "validated_pairs": report.as_ref().map(|value| serde_json::to_value(value.validated_pairs).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "status_r1": report.as_ref().map(|value| serde_json::json!(value.status_r1)).unwrap_or(serde_json::Value::Null),
        "status_r2": report.as_ref().map(|value| serde_json::json!(value.status_r2)).unwrap_or(serde_json::Value::Null),
        "pair_sync_checked": report.as_ref().map(|value| serde_json::json!(value.pair_sync_checked)).or_else(|| manifest.as_ref().map(|value| serde_json::json!(value.pair_sync_checked))).unwrap_or(serde_json::Value::Null),
        "pair_sync_pass": report.as_ref().map(|value| serde_json::to_value(value.pair_sync_pass).ok()).flatten().or_else(|| manifest.as_ref().map(|value| serde_json::to_value(value.pair_sync_pass).ok()).flatten()).unwrap_or(serde_json::Value::Null),
        "pair_count_match": report.as_ref().map(|value| serde_json::to_value(value.pair_count_match).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "paired_mode": manifest.as_ref().map(|value| serde_json::to_value(&value.paired_mode).ok()).flatten().unwrap_or(serde_json::Value::Null),
        "validated_stream_ids": manifest.as_ref().map(|value| serde_json::json!(value.validated_stream_ids)).unwrap_or(serde_json::Value::Null),
        "validation_report": manifest.as_ref().map(|value| serde_json::json!(value.validation_report)).unwrap_or(serde_json::Value::Null),
    }))
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
    use bijux_dna_domain_fastq::params::{
        validate::{PairSyncPolicy, ValidationMode},
        PairedMode,
    };
    use bijux_dna_domain_fastq::{
        ValidateFailureClass, ValidatedReadsManifestV1, ValidationReportV1,
        VALIDATED_READS_MANIFEST_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
    };
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlugin};

    use super::{validate_semantic_metrics, FastqStagePlugin};

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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["dedup_rate"],
            serde_json::json!(0.25)
        );
    }

    #[test]
    fn validate_semantic_metrics_surface_pair_lineage_contract() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("validation_report.json");
        let manifest_path = temp.path().join("validated_reads_manifest.json");
        std::fs::write(
            &report_path,
            serde_json::to_string(&ValidationReportV1 {
                schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
                stage: "fastq.validate_reads".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "seqtk".to_string(),
                validation_mode: ValidationMode::Strict,
                pair_sync_policy: PairSyncPolicy::RequireHeaderSync,
                input_r1: "reads_R1.fastq.gz".to_string(),
                input_r2: Some("reads_R2.fastq.gz".to_string()),
                validation_log_r1: "validation_r1.log".to_string(),
                validation_log_r2: Some("validation_r2.log".to_string()),
                validated_inputs: 2,
                validated_reads_r1: 1,
                validated_reads_r2: Some(1),
                validated_pairs: Some(1),
                status_r1: 0,
                status_r2: 0,
                pair_sync_checked: true,
                pair_sync_pass: Some(false),
                pair_count_match: Some(false),
                failure_class: ValidateFailureClass::HeaderSyncMismatch,
                strict_pass: false,
                exit_code: 97,
            })
            .expect("serialize report"),
        )
        .expect("write report");
        std::fs::write(
            &manifest_path,
            serde_json::to_string(&ValidatedReadsManifestV1 {
                schema_version: VALIDATED_READS_MANIFEST_SCHEMA_VERSION.to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "seqtk".to_string(),
                validation_mode: ValidationMode::Strict,
                pair_sync_policy: PairSyncPolicy::RequireHeaderSync,
                input_r1: "reads_R1.fastq.gz".to_string(),
                input_r2: Some("reads_R2.fastq.gz".to_string()),
                validation_report: "validation_report.json".to_string(),
                paired_mode: PairedMode::PairedEnd,
                validated_stream_ids: vec!["reads_r1".to_string(), "reads_r2".to_string()],
                pair_sync_checked: true,
                pair_sync_pass: Some(false),
                validated_pairs: Some(1),
            })
            .expect("serialize manifest"),
        )
        .expect("write manifest");
        let semantics = validate_semantic_metrics(&[
            ArtifactRef::required(
                ArtifactId::new("validation_report"),
                report_path,
                ArtifactRole::SummaryJson,
            ),
            ArtifactRef::required(
                ArtifactId::new("validated_reads_manifest"),
                manifest_path,
                ArtifactRole::StageReport,
            ),
        ])
        .expect("validate semantics");

        assert_eq!(semantics["validated_pairs"], serde_json::json!(1_u64));
        assert_eq!(semantics["validation_mode"], serde_json::json!("strict"));
        assert_eq!(
            semantics["failure_class"],
            serde_json::json!("header_sync_mismatch")
        );
        assert_eq!(semantics["pair_sync_pass"], serde_json::json!(false));
        assert_eq!(
            semantics["pair_sync_policy"],
            serde_json::json!("require_header_sync")
        );
        assert_eq!(semantics["paired_mode"], serde_json::json!("paired_end"));
        assert_eq!(
            semantics["validated_stream_ids"],
            serde_json::json!(["reads_r1", "reads_r2"])
        );
        assert_eq!(semantics["validated_reads_r1"], serde_json::json!(1_u64));
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
                "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": "cutadapt",
                "paired_mode": "single_end",
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2_u64,
                "trim_3p_bases": 1_u64,
                "requested_trim_5p_bases": 2_u64,
                "requested_trim_3p_bases": 1_u64,
                "udg_classification": "non_udg",
                "input_r1": "reads.fastq",
                "input_r2": null,
                "output_r1": "trimmed.fastq",
                "output_r2": null,
                "reads_in": null,
                "reads_out": null,
                "bases_in": null,
                "bases_out": null,
                "mean_q_before": null,
                "mean_q_after": null,
                "ct_ga_asymmetry_pre": null,
                "ct_ga_asymmetry_post": null,
                "ct_ga_asymmetry_pre_r1": null,
                "ct_ga_asymmetry_post_r1": null,
                "ct_ga_asymmetry_pre_r2": null,
                "ct_ga_asymmetry_post_r2": null,
                "terminal_base_composition_pre_r1": null,
                "terminal_base_composition_post_r1": null,
                "terminal_base_composition_pre_r2": null,
                "terminal_base_composition_post_r2": null,
                "raw_backend_report": "cutadapt.damage.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": null,
                "memory_mb": null
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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["execution_policy"],
            serde_json::json!("explicit_terminal_trim")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["trim_5p_bases"],
            serde_json::json!(2_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["udg_classification"],
            serde_json::json!("non_udg")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["raw_backend_report_format"],
            serde_json::json!("cutadapt_json")
        );
    }

    #[test]
    fn parse_outputs_surfaces_trim_read_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_path = temp.path().join("reads.fastq");
        let trimmed_reads_path = temp.path().join("trimmed.fastq");
        let report_path = temp.path().join("trim_report.json");
        let raw_backend_report_path = temp.path().join("trim_report.fastp.json");
        std::fs::write(&reads_path, b"@r1\nACGTGGGG\n+\n########\n").expect("write reads");
        std::fs::write(&trimmed_reads_path, b"@r1\nACGT\n+\n####\n").expect("write trimmed reads");
        std::fs::write(
            &raw_backend_report_path,
            serde_json::json!({
                "filtering_result": {
                    "passed_filter_reads": 96_u64,
                    "low_quality_reads": 3_u64,
                    "too_many_N_reads": 1_u64,
                    "too_short_reads": 4_u64
                }
            })
            .to_string(),
        )
        .expect("write raw backend report");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "input_r1": "reads.fastq",
                "input_r2": null,
                "output_r1": "trimmed.fastq",
                "output_r2": null,
                "min_length": 30_u64,
                "quality_cutoff": 20_u64,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "drop",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "default",
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "contaminant_bank_id": null,
                "contaminant_bank_hash": null,
                "contaminant_preset": null,
                "reads_in": 100_u64,
                "reads_out": 96_u64,
                "bases_in": 1000_u64,
                "bases_out": 840_u64,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "runtime_s": 4.2,
                "memory_mb": 128.0,
                "raw_backend_report": raw_backend_report_path,
                "raw_backend_report_format": "fastp_json"
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_reads"),
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
            ..plan("fastq.trim_reads")
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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["adapter_policy"],
            serde_json::json!("bank")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_out"],
            serde_json::json!(96_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["passed_filter_reads"],
            serde_json::json!(96_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["raw_backend_report_format"],
            serde_json::json!("fastp_json")
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
                "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
                "stage": "fastq.trim_polyg_tails",
                "stage_id": "fastq.trim_polyg_tails",
                "tool_id": "fastp",
                "paired_mode": "single_end",
                "trim_polyg": true,
                "min_polyg_run": 10_u64,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": 1_u64,
                "reads_out": 1_u64,
                "bases_in": 8_u64,
                "bases_out": 4_u64,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 30.0,
                "mean_q_after": 31.0,
                "bases_trimmed_polyg": 4_u64,
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "runtime_s": 1.0,
                "memory_mb": 16.0,
                "raw_backend_report": null,
                "raw_backend_report_format": "fastp_json",
                "backend_metrics": {
                    "schema_version": "bijux.fastp.metrics.v1",
                    "passed_filter_reads": 960_u64,
                    "low_quality_reads": 18_u64,
                    "too_many_n_reads": 4_u64,
                    "too_short_reads": 12_u64
                }
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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["min_polyg_run"],
            serde_json::json!(10_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["raw_backend_report_format"],
            serde_json::json!("fastp_json")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["polyx_preset"],
            serde_json::json!("illumina_twocolor")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["passed_filter_reads"],
            serde_json::json!(960_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["too_short_reads"],
            serde_json::json!(12_u64)
        );
    }

    #[test]
    fn parse_outputs_surfaces_bbduk_polyg_trim_semantics() {
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
                "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
                "stage": "fastq.trim_polyg_tails",
                "stage_id": "fastq.trim_polyg_tails",
                "tool_id": "bbduk",
                "paired_mode": "single_end",
                "trim_polyg": true,
                "min_polyg_run": 10_u64,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "trimmed.fastq.gz",
                "output_r2": null,
                "reads_in": 1_u64,
                "reads_out": 1_u64,
                "bases_in": 8_u64,
                "bases_out": 4_u64,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 30.0,
                "mean_q_after": 30.5,
                "bases_trimmed_polyg": 4_u64,
                "polyx_bank_id": null,
                "polyx_bank_hash": null,
                "polyx_preset": null,
                "runtime_s": 1.0,
                "memory_mb": 16.0,
                "raw_backend_report": null,
                "raw_backend_report_format": "bbduk_stats",
                "backend_metrics": {
                    "schema_version": "bijux.bbduk.trim_polyg.metrics.v1",
                    "reads_removed": 137_u64
                }
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.trim_polyg_tails"),
            tool_id: ToolId::from_static("bbduk"),
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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["raw_backend_report_format"],
            serde_json::json!("bbduk_stats")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["reads_removed"],
            serde_json::json!(137_u64)
        );
    }

    #[test]
    fn parse_outputs_surfaces_correction_semantics() {
        let plugin = FastqStagePlugin;
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_r1_path = temp.path().join("reads_R1.fastq");
        let reads_r2_path = temp.path().join("reads_R2.fastq");
        let corrected_r1_path = temp.path().join("corrected_R1.fastq");
        let corrected_r2_path = temp.path().join("corrected_R2.fastq");
        let report_path = temp.path().join("correct_report.json");
        std::fs::write(&reads_r1_path, b"@r1\nACGT\n+\n####\n").expect("write reads r1");
        std::fs::write(&reads_r2_path, b"@r1\nTGCA\n+\n####\n").expect("write reads r2");
        std::fs::write(&corrected_r1_path, b"@r1\nACGT\n+\n####\n").expect("write corrected r1");
        std::fs::write(&corrected_r2_path, b"@r1\nTGCA\n+\n####\n").expect("write corrected r2");
        std::fs::write(
            &report_path,
            serde_json::json!({
                "correction_engine": "rcorrector",
                "quality_encoding": "phred33",
                "kmer_size": 31_u64,
                "trusted_kmer_artifact": "trusted.kmers",
                "conservative_mode": false,
                "kmer_fix_rate": 0.125_f64,
                "correction_effect": {
                    "outputs_changed": true,
                    "bases_delta": -300_i64,
                    "mean_q_delta": 2.5_f64
                },
                "input_r1": reads_r1_path,
                "input_r2": reads_r2_path,
                "output_r1": corrected_r1_path,
                "output_r2": corrected_r2_path
            })
            .to_string(),
        )
        .expect("write report");
        let plan = bijux_dna_stage_contract::StagePlanV1 {
            stage_id: StageId::from_static("fastq.correct_errors"),
            tool_id: ToolId::from_static("rcorrector"),
            io: StageIO {
                inputs: vec![
                    ArtifactRef::required(
                        ArtifactId::new("reads_r1"),
                        reads_r1_path,
                        ArtifactRole::Reads,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("reads_r2"),
                        reads_r2_path,
                        ArtifactRole::Reads,
                    ),
                ],
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::new("corrected_reads_r1"),
                        corrected_r1_path,
                        ArtifactRole::Reads,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("corrected_reads_r2"),
                        corrected_r2_path,
                        ArtifactRole::Reads,
                    ),
                ],
            },
            ..plan("fastq.correct_errors")
        };

        let output = plugin
            .parse_outputs(
                &plan,
                &[
                    plan.io.outputs[0].clone(),
                    plan.io.outputs[1].clone(),
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
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["correction_engine"],
            serde_json::json!("rcorrector")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["quality_encoding"],
            serde_json::json!("phred33")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["kmer_size"],
            serde_json::json!(31_u64)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["trusted_kmer_artifact"],
            serde_json::json!("trusted.kmers")
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["correction_effect"]["outputs_changed"],
            serde_json::json!(true)
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
        std::fs::create_dir_all(&data_dir).expect("multiqc data dir");
        std::fs::write(
            data_dir.join("multiqc_general_stats.json"),
            include_str!("../tests/fixtures/tool_metrics/default/multiqc_general_stats.json"),
        )
        .expect("write multiqc general stats");
        std::fs::write(
            &manifest_path,
            serde_json::json!({
                "schema_version": "bijux.fastq.report_qc.inputs.v1",
                "lineage_hash": "fastq.trim_reads.fastp=report_json",
                "raw_fastqc_dir": "/tmp/raw_fastqc",
                "contributors": [
                    {
                        "contributor_id": "fastq.trim_reads.fastp",
                        "stage_id": "fastq.trim_reads",
                        "artifact_id": "report_json",
                        "artifact_role": "report_json",
                        "path": "/tmp/fastp/report.json"
                    },
                    {
                        "contributor_id": "fastq.validate_reads.fastqvalidator",
                        "stage_id": "fastq.validate_reads",
                        "artifact_id": "validation_report",
                        "artifact_role": "validation_report",
                        "path": "/tmp/validate/report.json"
                    }
                ],
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
            .parse_outputs(&plan, &plan.io.outputs)
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
            output.report_parts[0].payload["semantic_metrics"]["multiqc_sample_count"],
            serde_json::json!(2)
        );
        assert_eq!(
            output.report_parts[0].payload["semantic_metrics"]["multiqc_module_count"],
            serde_json::json!(2)
        );
        assert_eq!(
            output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
                ["lineage_hash"],
            serde_json::json!("fastq.trim_reads.fastp=report_json")
        );
    }
}
