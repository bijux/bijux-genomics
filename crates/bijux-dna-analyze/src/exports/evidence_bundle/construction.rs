//! Owner: bijux-dna-analyze
//! Acquire completed-run inputs and construct evidence projections.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_runtime::{FactsRowV1, TelemetryEventName, TelemetryEventV1};

use crate::load::{load_facts, load_run_summary};

use super::model::{
    EvidenceArtifactV1, EvidenceBundleV1, EvidenceCheckV1, EvidenceCitationTypeV1,
    EvidenceCitationV1, EvidenceCompactSummaryV1, EvidenceEdgeV1, EvidenceGapV1, EvidenceHealthV1,
    EvidenceMethodsSummaryV1, EvidenceMethodsToolV1, EvidenceMetricsV1, EvidenceNodeV1,
    EvidenceProvenanceGraphV1, EvidenceSeverityV1, EvidenceSourcesV1, EvidenceTimelineCategoryV1,
    EvidenceTimelineEventV1,
};

#[derive(Debug, Clone)]
struct EvidenceInputs {
    manifest_path: Option<PathBuf>,
    plan_manifest_path: Option<PathBuf>,
    report_path: Option<PathBuf>,
    run_summary_path: Option<PathBuf>,
    facts_path: Option<PathBuf>,
    graph_path: Option<PathBuf>,
    environment_path: Option<PathBuf>,
    runtime_policy_path: Option<PathBuf>,
    run_state_path: Option<PathBuf>,
    executor_descriptor_path: Option<PathBuf>,
    checkpoint_path: Option<PathBuf>,
    failure_path: Option<PathBuf>,
    artifact_inventory_path: Option<PathBuf>,
    replay_manifest_path: Option<PathBuf>,
    hash_ledger_path: Option<PathBuf>,
    evidence_verification_path: Option<PathBuf>,
    telemetry_paths: Vec<PathBuf>,
}

/// Build an evidence bundle from an existing run directory.
///
/// # Errors
/// Returns an error if required evidence inputs cannot be parsed.
#[allow(clippy::too_many_lines)]
pub fn build_evidence_bundle(
    base_dir: &Path,
    facts_path: Option<&Path>,
) -> Result<EvidenceBundleV1> {
    let inputs = discover_inputs(base_dir, facts_path);
    let manifest =
        load_optional_json(inputs.manifest_path.as_deref()).context("load evidence manifest")?;
    let report =
        load_optional_json(inputs.report_path.as_deref()).context("load evidence report")?;
    let summary = if let Some(path) = inputs.run_summary_path.as_deref() {
        load_run_summary(path).ok()
    } else {
        None
    };
    let facts = if let Some(path) = inputs.facts_path.as_deref() {
        Some(load_facts(path).with_context(|| format!("load facts {}", path.display()))?)
    } else {
        None
    };
    let telemetry = load_telemetry_events(&inputs.telemetry_paths)?;

    let run_id = manifest
        .as_ref()
        .and_then(|value| value.get("run_id"))
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            summary
                .as_ref()
                .and_then(|value| value.stage_rows.first().map(|row| row.run_id.as_str()))
        })
        .or_else(|| facts.as_ref().and_then(|rows| rows.first().map(|row| row.run_id.as_str())))
        .unwrap_or("unknown-run")
        .to_string();
    let correlation_id = manifest
        .as_ref()
        .and_then(|value| value.get("correlation_id"))
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| run_id.clone(), str::to_string);

    let timeline = build_timeline(
        &correlation_id,
        manifest.as_ref(),
        &telemetry,
        summary.as_ref(),
        report.as_ref(),
    );
    let artifacts = collect_declared_artifacts(manifest.as_ref());
    let provenance_graph =
        build_provenance_graph(manifest.as_ref(), summary.as_ref(), facts.as_deref(), &artifacts);
    let health = build_health(
        base_dir,
        &inputs,
        manifest.as_ref(),
        report.as_ref(),
        summary.as_ref(),
        facts.as_deref(),
        &artifacts,
    );
    let metrics =
        build_metrics(&timeline, report.as_ref(), summary.as_ref(), facts.as_deref(), &health);
    let compact_summary = build_compact_summary(
        summary.as_ref(),
        manifest.as_ref(),
        facts.as_deref(),
        &artifacts,
        &health,
    );
    let citations = build_citations(report.as_ref(), summary.as_ref());
    let methods_summary = Some(build_methods_summary(
        &run_id,
        &correlation_id,
        summary.as_ref(),
        facts.as_deref(),
        report.as_ref(),
        &health,
        &citations,
    ));

    Ok(EvidenceBundleV1 {
        schema_version: "bijux.evidence_bundle.v1".to_string(),
        run_id,
        correlation_id,
        sources: EvidenceSourcesV1 {
            manifest_path: to_relative_string(base_dir, inputs.manifest_path.as_deref()),
            plan_manifest_path: to_relative_string(base_dir, inputs.plan_manifest_path.as_deref()),
            report_path: to_relative_string(base_dir, inputs.report_path.as_deref()),
            run_summary_path: to_relative_string(base_dir, inputs.run_summary_path.as_deref()),
            facts_path: to_relative_string(base_dir, inputs.facts_path.as_deref()),
            graph_path: to_relative_string(base_dir, inputs.graph_path.as_deref()),
            environment_path: to_relative_string(base_dir, inputs.environment_path.as_deref()),
            runtime_policy_path: to_relative_string(
                base_dir,
                inputs.runtime_policy_path.as_deref(),
            ),
            run_state_path: to_relative_string(base_dir, inputs.run_state_path.as_deref()),
            executor_descriptor_path: to_relative_string(
                base_dir,
                inputs.executor_descriptor_path.as_deref(),
            ),
            checkpoint_path: to_relative_string(base_dir, inputs.checkpoint_path.as_deref()),
            failure_path: to_relative_string(base_dir, inputs.failure_path.as_deref()),
            artifact_inventory_path: to_relative_string(
                base_dir,
                inputs.artifact_inventory_path.as_deref(),
            ),
            replay_manifest_path: to_relative_string(
                base_dir,
                inputs.replay_manifest_path.as_deref(),
            ),
            hash_ledger_path: to_relative_string(base_dir, inputs.hash_ledger_path.as_deref()),
            evidence_verification_path: to_relative_string(
                base_dir,
                inputs.evidence_verification_path.as_deref(),
            ),
            telemetry_paths: inputs
                .telemetry_paths
                .iter()
                .map(|path| relative_or_display(base_dir, path))
                .collect(),
        },
        compact_summary,
        health,
        metrics,
        timeline,
        artifacts,
        provenance_graph,
        citations,
        methods_summary,
    })
}

fn discover_inputs(base_dir: &Path, facts_path: Option<&Path>) -> EvidenceInputs {
    EvidenceInputs {
        manifest_path: first_existing(base_dir, &["run_manifest.json", "execution_manifest.json"]),
        plan_manifest_path: first_existing(
            base_dir,
            &["manifests/plan_manifest.json", "plan_manifest.json"],
        ),
        report_path: first_existing(base_dir, &["report.json"]),
        run_summary_path: first_existing(
            base_dir,
            &["run_summary.json", "summary/run_summary.json"],
        ),
        facts_path: facts_path
            .map(Path::to_path_buf)
            .or_else(|| first_existing(base_dir, &["facts.jsonl", "summary/facts.jsonl"])),
        graph_path: first_existing(base_dir, &["graph.json", "run_artifacts/graph.json"]),
        environment_path: first_existing(base_dir, &["environment.json"]),
        runtime_policy_path: first_existing(base_dir, &["runtime_policy.json"]),
        run_state_path: first_existing(base_dir, &["run_state.json"]),
        executor_descriptor_path: first_existing(base_dir, &["executor_descriptor.json"]),
        checkpoint_path: first_existing(
            base_dir,
            &["checkpoints/checkpoint.json", "checkpoint.json"],
        ),
        failure_path: first_existing(base_dir, &["run_failure.json"]),
        artifact_inventory_path: first_existing(base_dir, &["artifact_inventory.json"]),
        replay_manifest_path: first_existing(base_dir, &["replay_manifest.json"]),
        hash_ledger_path: first_existing(base_dir, &["hash_ledger.json"]),
        evidence_verification_path: first_existing(base_dir, &["evidence_verification.json"]),
        telemetry_paths: find_telemetry_paths(base_dir),
    }
}

#[allow(clippy::too_many_lines)]
fn build_timeline(
    correlation_id: &str,
    manifest: Option<&serde_json::Value>,
    telemetry: &[TelemetryEventV1],
    summary: Option<&crate::model::RunSummaryV1>,
    report: Option<&serde_json::Value>,
) -> Vec<EvidenceTimelineEventV1> {
    let mut timeline = Vec::new();
    if let Some(manifest) = manifest {
        timeline.push(EvidenceTimelineEventV1 {
            category: EvidenceTimelineCategoryV1::Planner,
            event: "plan_manifest_declared".to_string(),
            timestamp: None,
            stage_id: None,
            tool_id: None,
            correlation_id: correlation_id.to_string(),
            status: "observed".to_string(),
            attrs: serde_json::json!({
                "graph_hash": manifest.get("graph_hash").cloned().unwrap_or(serde_json::Value::Null),
                "stage_count": manifest
                    .get("stages")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len),
            }),
        });
        timeline.push(EvidenceTimelineEventV1 {
            category: EvidenceTimelineCategoryV1::Scheduler,
            event: "stage_schedule_materialized".to_string(),
            timestamp: None,
            stage_id: None,
            tool_id: None,
            correlation_id: correlation_id.to_string(),
            status: "observed".to_string(),
            attrs: serde_json::json!({
                "planned_stages": manifest.get("stages").cloned().unwrap_or(serde_json::Value::Null),
            }),
        });
        if manifest.get("cache_key").is_some_and(|value| !value.is_null()) {
            timeline.push(EvidenceTimelineEventV1 {
                category: EvidenceTimelineCategoryV1::Cache,
                event: "cache_key_declared".to_string(),
                timestamp: None,
                stage_id: None,
                tool_id: None,
                correlation_id: correlation_id.to_string(),
                status: "observed".to_string(),
                attrs: serde_json::json!({
                    "cache_key": manifest.get("cache_key").cloned().unwrap_or(serde_json::Value::Null),
                }),
            });
        } else {
            timeline.push(EvidenceTimelineEventV1 {
                category: EvidenceTimelineCategoryV1::Cache,
                event: "cache_miss_reason_unavailable".to_string(),
                timestamp: None,
                stage_id: None,
                tool_id: None,
                correlation_id: correlation_id.to_string(),
                status: "missing".to_string(),
                attrs: serde_json::json!({ "reason": "manifest does not declare cache outcome details" }),
            });
        }
        if manifest.get("execution_replay_identity").is_some() {
            timeline.push(EvidenceTimelineEventV1 {
                category: EvidenceTimelineCategoryV1::Replay,
                event: "replay_identity_recorded".to_string(),
                timestamp: None,
                stage_id: None,
                tool_id: None,
                correlation_id: correlation_id.to_string(),
                status: "observed".to_string(),
                attrs: manifest
                    .get("execution_replay_identity")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            });
        }
        if let Some(artifacts) =
            manifest.get("output_artifacts").and_then(serde_json::Value::as_array)
        {
            for artifact in artifacts {
                timeline.push(EvidenceTimelineEventV1 {
                    category: EvidenceTimelineCategoryV1::Artifact,
                    event: "artifact_manifest_entry".to_string(),
                    timestamp: None,
                    stage_id: artifact
                        .get("stage_id")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string),
                    tool_id: None,
                    correlation_id: correlation_id.to_string(),
                    status: "observed".to_string(),
                    attrs: artifact.clone(),
                });
            }
        }
    }
    for event in telemetry {
        timeline.push(EvidenceTimelineEventV1 {
            category: EvidenceTimelineCategoryV1::Execution,
            event: telemetry_event_label(&event.event_name).to_string(),
            timestamp: Some(event.timestamp.to_rfc3339()),
            stage_id: Some(event.stage_id.clone()),
            tool_id: Some(event.tool_id.clone()),
            correlation_id: correlation_id.to_string(),
            status: event.status.clone(),
            attrs: serde_json::json!({
                "trace_id": event.trace_id,
                "span_id": event.span_id,
                "attrs": event.attrs,
                "failure_code": event.failure_code,
            }),
        });
    }
    if let Some(summary) = summary {
        timeline.push(EvidenceTimelineEventV1 {
            category: EvidenceTimelineCategoryV1::Artifact,
            event: "run_summary_materialized".to_string(),
            timestamp: None,
            stage_id: None,
            tool_id: None,
            correlation_id: correlation_id.to_string(),
            status: "observed".to_string(),
            attrs: serde_json::json!({
                "run_count": summary.runs,
                "stage_count": summary.stages,
                "final_outputs": summary.final_outputs,
            }),
        });
    }
    if let Some(report) = report {
        timeline.push(EvidenceTimelineEventV1 {
            category: EvidenceTimelineCategoryV1::Artifact,
            event: "scientific_report_materialized".to_string(),
            timestamp: None,
            stage_id: None,
            tool_id: None,
            correlation_id: correlation_id.to_string(),
            status: "observed".to_string(),
            attrs: serde_json::json!({
                "completeness": report.get("completeness").cloned().unwrap_or(serde_json::Value::Null),
                "pipeline_verdict": report.get("pipeline_verdict").cloned().unwrap_or(serde_json::Value::Null),
            }),
        });
    }
    timeline.sort_by(|left, right| {
        (
            left.timestamp.as_deref().unwrap_or(""),
            category_order(&left.category),
            left.event.as_str(),
            left.stage_id.as_deref().unwrap_or(""),
            left.tool_id.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.timestamp.as_deref().unwrap_or(""),
                category_order(&right.category),
                right.event.as_str(),
                right.stage_id.as_deref().unwrap_or(""),
                right.tool_id.as_deref().unwrap_or(""),
            ))
    });
    timeline
}

#[allow(clippy::too_many_lines)]
fn build_health(
    base_dir: &Path,
    inputs: &EvidenceInputs,
    manifest: Option<&serde_json::Value>,
    report: Option<&serde_json::Value>,
    summary: Option<&crate::model::RunSummaryV1>,
    facts: Option<&[FactsRowV1]>,
    artifacts: &[EvidenceArtifactV1],
) -> EvidenceHealthV1 {
    let mut checks = Vec::new();
    let mut gaps = Vec::new();

    checks.push(EvidenceCheckV1 {
        check_id: "manifest_present".to_string(),
        ok: inputs.manifest_path.is_some(),
        message: if inputs.manifest_path.is_some() {
            "run manifest discovered".to_string()
        } else {
            "run manifest missing".to_string()
        },
    });
    if inputs.manifest_path.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_manifest".to_string(),
            severity: EvidenceSeverityV1::Blocking,
            message: "run manifest is required for auditable evidence".to_string(),
            path: None,
            blocks_audit: true,
        });
    }
    checks.push(EvidenceCheckV1 {
        check_id: "manifest_parse".to_string(),
        ok: manifest.is_some(),
        message: if manifest.is_some() {
            "run manifest parsed".to_string()
        } else {
            "run manifest could not be parsed".to_string()
        },
    });
    if manifest.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "invalid_manifest".to_string(),
            severity: EvidenceSeverityV1::Blocking,
            message: "run manifest could not be parsed into a governed evidence input".to_string(),
            path: inputs.manifest_path.as_ref().map(|path| relative_or_display(base_dir, path)),
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "report_present".to_string(),
        ok: report.is_some(),
        message: if report.is_some() {
            "scientific report present".to_string()
        } else {
            "scientific report missing".to_string()
        },
    });
    if report.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_report".to_string(),
            severity: EvidenceSeverityV1::Advisory,
            message: "run is usable but not fully auditable without report.json".to_string(),
            path: Some("report.json".to_string()),
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "run_summary_present".to_string(),
        ok: summary.is_some(),
        message: if summary.is_some() {
            "run summary present".to_string()
        } else {
            "run summary missing".to_string()
        },
    });
    if summary.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_run_summary".to_string(),
            severity: EvidenceSeverityV1::Advisory,
            message: "run summary is missing, so compact evidence views are degraded".to_string(),
            path: Some("run_summary.json".to_string()),
            blocks_audit: false,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "facts_present".to_string(),
        ok: facts.is_some(),
        message: if facts.is_some() {
            "facts rows present".to_string()
        } else {
            "facts rows missing".to_string()
        },
    });
    if facts.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_facts".to_string(),
            severity: EvidenceSeverityV1::Advisory,
            message: "facts.jsonl is missing, so evidence metrics and scientific failure classification are reduced".to_string(),
            path: Some("facts.jsonl".to_string()),
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "telemetry_present".to_string(),
        ok: !inputs.telemetry_paths.is_empty(),
        message: if inputs.telemetry_paths.is_empty() {
            "telemetry events missing".to_string()
        } else {
            format!("{} telemetry files discovered", inputs.telemetry_paths.len())
        },
    });
    if inputs.telemetry_paths.is_empty() {
        gaps.push(EvidenceGapV1 {
            code: "missing_telemetry".to_string(),
            severity: EvidenceSeverityV1::Advisory,
            message: "telemetry timeline is incomplete because no telemetry jsonl files were found"
                .to_string(),
            path: None,
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "artifact_inventory_present".to_string(),
        ok: inputs.artifact_inventory_path.is_some(),
        message: if inputs.artifact_inventory_path.is_some() {
            "artifact inventory present".to_string()
        } else {
            "artifact inventory missing".to_string()
        },
    });
    if inputs.artifact_inventory_path.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_artifact_inventory".to_string(),
            severity: EvidenceSeverityV1::Blocking,
            message: "artifact inventory is required for durable evidence reuse and audit"
                .to_string(),
            path: Some("artifact_inventory.json".to_string()),
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "hash_ledger_present".to_string(),
        ok: inputs.hash_ledger_path.is_some(),
        message: if inputs.hash_ledger_path.is_some() {
            "hash ledger present".to_string()
        } else {
            "hash ledger missing".to_string()
        },
    });
    if inputs.hash_ledger_path.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_hash_ledger".to_string(),
            severity: EvidenceSeverityV1::Blocking,
            message: "hash ledger is required for tamper-evident evidence".to_string(),
            path: Some("hash_ledger.json".to_string()),
            blocks_audit: true,
        });
    }

    checks.push(EvidenceCheckV1 {
        check_id: "replay_manifest_present".to_string(),
        ok: inputs.replay_manifest_path.is_some(),
        message: if inputs.replay_manifest_path.is_some() {
            "replay manifest present".to_string()
        } else {
            "replay manifest missing".to_string()
        },
    });
    if inputs.replay_manifest_path.is_none() {
        gaps.push(EvidenceGapV1 {
            code: "missing_replay_manifest".to_string(),
            severity: EvidenceSeverityV1::Advisory,
            message: "replay provenance is reduced because replay_manifest.json is missing"
                .to_string(),
            path: Some("replay_manifest.json".to_string()),
            blocks_audit: false,
        });
    }

    let mut artifact_integrity_ok = true;
    for artifact in artifacts {
        let full = base_dir.join(&artifact.path);
        if !full.exists() {
            artifact_integrity_ok = false;
            gaps.push(EvidenceGapV1 {
                code: "missing_artifact".to_string(),
                severity: EvidenceSeverityV1::Blocking,
                message: format!("artifact {} is missing from the run bundle", artifact.name),
                path: Some(artifact.path.clone()),
                blocks_audit: true,
            });
            continue;
        }
        if let Some(expected_hash) = &artifact.sha256 {
            match bijux_dna_infra::hash_file_sha256(&full) {
                Ok(actual_hash) if actual_hash == *expected_hash => {}
                Ok(_) | Err(_) => {
                    artifact_integrity_ok = false;
                    gaps.push(EvidenceGapV1 {
                        code: "artifact_hash_mismatch".to_string(),
                        severity: EvidenceSeverityV1::Blocking,
                        message: format!(
                            "artifact {} does not match its declared hash",
                            artifact.name
                        ),
                        path: Some(artifact.path.clone()),
                        blocks_audit: true,
                    });
                }
            }
        }
    }
    checks.push(EvidenceCheckV1 {
        check_id: "artifact_integrity".to_string(),
        ok: artifact_integrity_ok,
        message: if artifact_integrity_ok {
            "artifact inventory resolved and hashes matched".to_string()
        } else {
            "artifact inventory contains missing files or hash mismatches".to_string()
        },
    });

    let auditable = gaps.iter().all(|gap| !gap.blocks_audit);
    let status = if checks.iter().all(|check| check.ok) && gaps.is_empty() {
        "complete"
    } else if checks.iter().any(|check| !check.ok && check.check_id == "artifact_integrity")
        || gaps.iter().any(|gap| matches!(gap.severity, EvidenceSeverityV1::Blocking))
    {
        "failed"
    } else {
        "usable_with_gaps"
    };
    EvidenceHealthV1 { status: status.to_string(), auditable, checks, gaps }
}

fn build_metrics(
    timeline: &[EvidenceTimelineEventV1],
    report: Option<&serde_json::Value>,
    summary: Option<&crate::model::RunSummaryV1>,
    facts: Option<&[FactsRowV1]>,
    health: &EvidenceHealthV1,
) -> EvidenceMetricsV1 {
    let run_time_s = summary
        .map(|summary| summary.total_runtime_s)
        .or_else(|| facts.map(|rows| rows.iter().map(|row| row.runtime_s).sum()))
        .unwrap_or(0.0);
    let failed_stage_count =
        facts.map_or(0, |rows| usize_to_u64(rows.iter().filter(|row| row.exit_code != 0).count()));
    let queue_time_ms = queue_time_from_timeline(timeline);
    let cache_hit_count = timeline
        .iter()
        .filter(|event| {
            matches!(event.category, EvidenceTimelineCategoryV1::Cache) && event.status == "hit"
        })
        .count();
    let cache_miss_count = timeline
        .iter()
        .filter(|event| {
            matches!(event.category, EvidenceTimelineCategoryV1::Cache)
                && (event.status == "missing" || event.status == "miss")
        })
        .count();
    let retry_count = timeline
        .iter()
        .filter_map(|event| event.attrs.get("retry_count").and_then(serde_json::Value::as_u64))
        .sum();
    let mut scientific_failure_classes = BTreeMap::new();
    if failed_stage_count > 0 {
        scientific_failure_classes.insert("execution_failure".to_string(), failed_stage_count);
    }
    let audit_gaps = health.health_gap_count();
    if audit_gaps > 0 {
        scientific_failure_classes.insert("evidence_gap".to_string(), usize_to_u64(audit_gaps));
    }
    if let Some(report) = report {
        if report
            .get("pipeline_verdict")
            .and_then(|value| value.get("verdict"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|verdict| verdict.eq_ignore_ascii_case("fail"))
        {
            *scientific_failure_classes.entry("scientific_refusal".to_string()).or_insert(0) += 1;
        }
    }
    EvidenceMetricsV1 {
        queue_time_ms,
        run_time_s,
        retry_count,
        cache_hit_count: usize_to_u64(cache_hit_count),
        cache_miss_count: usize_to_u64(cache_miss_count),
        total_timeline_events: usize_to_u64(timeline.len()),
        scientific_failure_classes,
    }
}

fn build_compact_summary(
    summary: Option<&crate::model::RunSummaryV1>,
    manifest: Option<&serde_json::Value>,
    facts: Option<&[FactsRowV1]>,
    artifacts: &[EvidenceArtifactV1],
    health: &EvidenceHealthV1,
) -> EvidenceCompactSummaryV1 {
    let mut stage_ids: BTreeSet<String> = BTreeSet::new();
    if let Some(summary) = summary {
        stage_ids.extend(summary.stage_rows.iter().map(|row| row.stage_id.clone()));
    }
    if let Some(manifest) = manifest {
        if let Some(stages) = manifest.get("stages").and_then(serde_json::Value::as_array) {
            stage_ids.extend(stages.iter().filter_map(|stage| {
                stage.get("stage_id").and_then(serde_json::Value::as_str).map(str::to_string)
            }));
        }
    }
    if let Some(facts) = facts {
        stage_ids.extend(facts.iter().map(|row| row.stage_id.clone()));
    }
    let final_outputs = summary.map(|summary| summary.final_outputs.clone()).unwrap_or_default();
    let failed_stage_count =
        facts.map_or(0, |rows| rows.iter().filter(|row| row.exit_code != 0).count());
    EvidenceCompactSummaryV1 {
        stage_count: stage_ids.len(),
        artifact_count: artifacts.len(),
        failed_stage_count,
        advisory_gap_count: health
            .gaps
            .iter()
            .filter(|gap| matches!(gap.severity, EvidenceSeverityV1::Advisory))
            .count(),
        final_outputs,
        stage_ids: stage_ids.into_iter().collect(),
    }
}

fn build_provenance_graph(
    manifest: Option<&serde_json::Value>,
    summary: Option<&crate::model::RunSummaryV1>,
    facts: Option<&[FactsRowV1]>,
    artifacts: &[EvidenceArtifactV1],
) -> EvidenceProvenanceGraphV1 {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut seen_nodes = BTreeSet::new();
    let mut seen_edges = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    let mut input_ids = BTreeSet::new();

    if let Some(manifest) = manifest {
        if let Some(inputs) =
            manifest.get("dataset_fingerprints").and_then(serde_json::Value::as_array)
        {
            for input in inputs.iter().filter_map(serde_json::Value::as_str) {
                input_ids.insert(input.to_string());
            }
        }
        if let Some(stages) = manifest.get("stages").and_then(serde_json::Value::as_array) {
            for stage in stages.iter().filter_map(|value| {
                value.get("stage_id").and_then(serde_json::Value::as_str).map(str::to_string)
            }) {
                stage_ids.insert(stage);
            }
        }
    }
    if let Some(summary) = summary {
        stage_ids.extend(summary.stage_rows.iter().map(|row| row.stage_id.clone()));
    }
    if let Some(facts) = facts {
        stage_ids.extend(facts.iter().map(|row| row.stage_id.clone()));
        input_ids.extend(facts.iter().map(|row| row.input_hash.clone()));
    }

    for input in &input_ids {
        push_node(&mut nodes, &mut seen_nodes, format!("input:{input}"), "input", input.clone());
    }
    for stage in &stage_ids {
        push_node(&mut nodes, &mut seen_nodes, format!("stage:{stage}"), "stage", stage.clone());
    }
    for artifact in artifacts {
        push_node(
            &mut nodes,
            &mut seen_nodes,
            format!("artifact:{}", artifact.name),
            "artifact",
            artifact.name.clone(),
        );
    }

    for input in &input_ids {
        for stage in &stage_ids {
            push_edge(
                &mut edges,
                &mut seen_edges,
                format!("input:{input}"),
                format!("stage:{stage}"),
                "consumed_by",
            );
        }
    }
    for stage in &stage_ids {
        for artifact in artifacts {
            push_edge(
                &mut edges,
                &mut seen_edges,
                format!("stage:{stage}"),
                format!("artifact:{}", artifact.name),
                "produced",
            );
        }
    }
    EvidenceProvenanceGraphV1 { nodes, edges }
}

// Artifact provenance comes from declarations; publications do not add implicit sources.
fn collect_declared_artifacts(manifest: Option<&serde_json::Value>) -> Vec<EvidenceArtifactV1> {
    let mut artifacts = Vec::new();
    if let Some(entries) = manifest
        .and_then(|manifest| manifest.get("output_artifacts"))
        .and_then(serde_json::Value::as_array)
    {
        for entry in entries {
            let name = entry
                .get("name")
                .or_else(|| entry.get("kind"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("artifact")
                .to_string();
            let path = entry
                .get("path")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            let sha256 =
                entry.get("sha256").and_then(serde_json::Value::as_str).map(str::to_string);
            artifacts.push(EvidenceArtifactV1 { name, path, sha256 });
        }
    }
    artifacts.sort_by(|left, right| {
        (left.name.as_str(), left.path.as_str()).cmp(&(right.name.as_str(), right.path.as_str()))
    });
    artifacts.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    artifacts
}

fn load_telemetry_events(paths: &[PathBuf]) -> Result<Vec<TelemetryEventV1>> {
    let mut events = Vec::new();
    for path in paths {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read telemetry events {}", path.display()))?;
        for (index, line) in raw.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let event: TelemetryEventV1 = serde_json::from_str(line).with_context(|| {
                format!("parse telemetry event {} line {}", path.display(), index + 1)
            })?;
            events.push(event);
        }
    }
    Ok(events)
}

fn build_methods_summary(
    run_id: &str,
    correlation_id: &str,
    summary: Option<&crate::model::RunSummaryV1>,
    facts: Option<&[FactsRowV1]>,
    report: Option<&serde_json::Value>,
    health: &EvidenceHealthV1,
    citations: &[EvidenceCitationV1],
) -> EvidenceMethodsSummaryV1 {
    let mut tools: BTreeMap<(String, String), EvidenceMethodsToolV1> = BTreeMap::new();
    if let Some(summary) = summary {
        for row in &summary.stage_rows {
            let key = (row.stage_id.clone(), row.tool_id.clone());
            tools.entry(key).or_insert_with(|| EvidenceMethodsToolV1 {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                tool_version: row.tool_version.clone(),
                image_digest: row.image_digest.clone(),
                params_hash: row.params_hash.clone(),
            });
        }
    }
    if let Some(facts) = facts {
        for row in facts {
            let key = (row.stage_id.clone(), row.tool_id.clone());
            tools.entry(key).or_insert_with(|| EvidenceMethodsToolV1 {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                tool_version: row.tool_version.clone(),
                image_digest: row.image_digest.clone(),
                params_hash: row.params_hash.clone(),
            });
        }
    }

    let assumptions = report
        .and_then(|value| value.get("sections"))
        .and_then(|value| value.get("method_assumptions"))
        .and_then(|value| value.get("assumptions"))
        .and_then(serde_json::Value::as_array)
        .map_or_else(Vec::new, |rows| {
            rows.iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        });
    let caveats = health.gaps.iter().map(|gap| gap.message.clone()).collect::<Vec<_>>();
    EvidenceMethodsSummaryV1 {
        schema_version: "bijux.methods_summary.v1".to_string(),
        run_id: run_id.to_string(),
        correlation_id: correlation_id.to_string(),
        stage_count: tools.len(),
        tools: tools.into_values().collect(),
        assumptions,
        caveats,
        citation_count: citations.len(),
    }
}

fn build_citations(
    report: Option<&serde_json::Value>,
    summary: Option<&crate::model::RunSummaryV1>,
) -> Vec<EvidenceCitationV1> {
    let mut citations = Vec::new();
    let mut seen = BTreeSet::new();
    let stage_tools = summary
        .map(|value| {
            value
                .stage_rows
                .iter()
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let Some(report) = report else {
        return citations;
    };
    if let Some(tool_provenance) = report
        .get("sections")
        .and_then(|value| value.get("pipeline_defaults"))
        .and_then(|value| value.get("defaults_ledger"))
        .and_then(|value| value.get("tool_provenance"))
        .and_then(serde_json::Value::as_object)
    {
        for (stage_id, payload) in tool_provenance {
            if let Some(rows) = payload.get("citations").and_then(serde_json::Value::as_array) {
                for citation in rows.iter().filter_map(serde_json::Value::as_str) {
                    let key = format!("defaults:{stage_id}:{citation}");
                    if !seen.insert(key.clone()) {
                        continue;
                    }
                    citations.push(EvidenceCitationV1 {
                        citation_id: key,
                        citation: citation.to_string(),
                        citation_type: EvidenceCitationTypeV1::Defaults,
                        stage_id: Some(stage_id.clone()),
                        tool_id: stage_tools.get(stage_id).cloned(),
                    });
                }
            }
        }
    }
    if let Some(global) = report
        .get("sections")
        .and_then(|value| value.get("pipeline_defaults"))
        .and_then(|value| value.get("defaults_ledger"))
        .and_then(|value| value.get("citations"))
        .and_then(serde_json::Value::as_object)
    {
        for (key, value) in global {
            let entries = match value {
                serde_json::Value::String(single) => vec![single.clone()],
                serde_json::Value::Array(rows) => rows
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            for citation in entries {
                let citation_id = format!("global:{key}:{citation}");
                if !seen.insert(citation_id.clone()) {
                    continue;
                }
                citations.push(EvidenceCitationV1 {
                    citation_id,
                    citation,
                    citation_type: EvidenceCitationTypeV1::Reference,
                    stage_id: None,
                    tool_id: None,
                });
            }
        }
    }

    citations.sort_by(|left, right| {
        (
            left.stage_id.as_deref().unwrap_or(""),
            left.tool_id.as_deref().unwrap_or(""),
            left.citation.as_str(),
        )
            .cmp(&(
                right.stage_id.as_deref().unwrap_or(""),
                right.tool_id.as_deref().unwrap_or(""),
                right.citation.as_str(),
            ))
    });
    citations
}

fn load_optional_json(path: Option<&Path>) -> Result<Option<serde_json::Value>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(value))
}

fn first_existing(base_dir: &Path, candidates: &[&str]) -> Option<PathBuf> {
    candidates.iter().map(|candidate| base_dir.join(candidate)).find(|path| path.exists())
}

fn find_telemetry_paths(base_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_telemetry_paths(base_dir, &mut out);
    out.sort();
    out
}

fn collect_telemetry_paths(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_telemetry_paths(&path, out);
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "events.jsonl" || name == "telemetry.jsonl")
        {
            out.push(path);
        }
    }
}

fn to_relative_string(base_dir: &Path, path: Option<&Path>) -> Option<String> {
    path.map(|path| relative_or_display(base_dir, path))
}

fn relative_or_display(base_dir: &Path, path: &Path) -> String {
    path.strip_prefix(base_dir).unwrap_or(path).display().to_string()
}

fn push_node(
    nodes: &mut Vec<EvidenceNodeV1>,
    seen: &mut BTreeSet<String>,
    node_id: String,
    kind: &str,
    label: String,
) {
    if seen.insert(node_id.clone()) {
        nodes.push(EvidenceNodeV1 { node_id, kind: kind.to_string(), label });
    }
}

fn push_edge(
    edges: &mut Vec<EvidenceEdgeV1>,
    seen: &mut BTreeSet<(String, String, String)>,
    from: String,
    to: String,
    relation: &str,
) {
    let key = (from.clone(), to.clone(), relation.to_string());
    if seen.insert(key) {
        edges.push(EvidenceEdgeV1 { from, to, relation: relation.to_string() });
    }
}

fn queue_time_from_timeline(timeline: &[EvidenceTimelineEventV1]) -> Option<u64> {
    let planner = timeline
        .iter()
        .find(|event| matches!(event.category, EvidenceTimelineCategoryV1::Planner))?;
    let execution = timeline
        .iter()
        .find(|event| matches!(event.category, EvidenceTimelineCategoryV1::Execution))?;
    let planner_ts = planner.timestamp.as_deref()?;
    let execution_ts = execution.timestamp.as_deref()?;
    let planner = chrono::DateTime::parse_from_rfc3339(planner_ts).ok()?;
    let execution = chrono::DateTime::parse_from_rfc3339(execution_ts).ok()?;
    let millis = (execution - planner).num_milliseconds();
    u64::try_from(millis).ok()
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn telemetry_event_label(event: &TelemetryEventName) -> &'static str {
    match event {
        TelemetryEventName::RunStarted => "run_started",
        TelemetryEventName::StageStart => "stage_start",
        TelemetryEventName::ToolInvocation => "tool_invocation",
        TelemetryEventName::StdoutSummary => "stdout_summary",
        TelemetryEventName::StderrSummary => "stderr_summary",
        TelemetryEventName::InvariantResult => "invariant_result",
        TelemetryEventName::ArtifactWritten => "artifact_written",
        TelemetryEventName::MetricsEmitted => "metrics_emitted",
        TelemetryEventName::StageEnd => "stage_end",
        TelemetryEventName::RunFinished => "run_finished",
        TelemetryEventName::RunFailed => "run_failed",
        TelemetryEventName::MergeDecision => "merge_decision",
        TelemetryEventName::AdapterValidation => "adapter_validation",
        TelemetryEventName::ContaminantAction => "contaminant_action",
        TelemetryEventName::QualityGate => "quality_gate",
        TelemetryEventName::Error => "error",
    }
}

fn category_order(category: &EvidenceTimelineCategoryV1) -> u8 {
    match category {
        EvidenceTimelineCategoryV1::Planner => 0,
        EvidenceTimelineCategoryV1::Scheduler => 1,
        EvidenceTimelineCategoryV1::Execution => 2,
        EvidenceTimelineCategoryV1::Artifact => 3,
        EvidenceTimelineCategoryV1::Cache => 4,
        EvidenceTimelineCategoryV1::Replay => 5,
    }
}
