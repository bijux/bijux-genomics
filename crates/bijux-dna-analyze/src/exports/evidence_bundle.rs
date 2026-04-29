use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_runtime::{FactsRowV1, TelemetryEventName, TelemetryEventV1};
use serde::{Deserialize, Serialize};

use crate::load::{load_facts, load_run_summary};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTimelineCategoryV1 {
    Planner,
    Scheduler,
    Execution,
    Artifact,
    Cache,
    Replay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSeverityV1 {
    Advisory,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceTimelineEventV1 {
    pub category: EvidenceTimelineCategoryV1,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    pub correlation_id: String,
    pub status: String,
    #[serde(default)]
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifactV1 {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGapV1 {
    pub code: String,
    pub severity: EvidenceSeverityV1,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub blocks_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCheckV1 {
    pub check_id: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceHealthV1 {
    pub status: String,
    pub auditable: bool,
    pub checks: Vec<EvidenceCheckV1>,
    pub gaps: Vec<EvidenceGapV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMetricsV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue_time_ms: Option<u64>,
    pub run_time_s: f64,
    pub retry_count: u64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub total_timeline_events: u64,
    pub scientific_failure_classes: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCompactSummaryV1 {
    pub stage_count: usize,
    pub artifact_count: usize,
    pub failed_stage_count: usize,
    pub advisory_gap_count: usize,
    pub final_outputs: Vec<String>,
    pub stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNodeV1 {
    pub node_id: String,
    pub kind: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEdgeV1 {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProvenanceGraphV1 {
    pub nodes: Vec<EvidenceNodeV1>,
    pub edges: Vec<EvidenceEdgeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSourcesV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_path: Option<String>,
    #[serde(default)]
    pub telemetry_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundleV1 {
    pub schema_version: String,
    pub run_id: String,
    pub correlation_id: String,
    pub sources: EvidenceSourcesV1,
    pub compact_summary: EvidenceCompactSummaryV1,
    pub health: EvidenceHealthV1,
    pub metrics: EvidenceMetricsV1,
    pub timeline: Vec<EvidenceTimelineEventV1>,
    pub artifacts: Vec<EvidenceArtifactV1>,
    pub provenance_graph: EvidenceProvenanceGraphV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceVerificationV1 {
    pub schema_version: String,
    pub verified: bool,
    pub checks: Vec<EvidenceCheckV1>,
    pub missing_paths: Vec<String>,
    pub gap_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceComparisonV1 {
    pub schema_version: String,
    pub left_run_id: String,
    pub right_run_id: String,
    pub left_correlation_id: String,
    pub right_correlation_id: String,
    pub changed_stage_ids: Vec<String>,
    pub changed_artifacts: Vec<String>,
    pub runtime_delta_s: f64,
    pub evidence_gap_delta: i64,
    pub policy_change_hints: Vec<String>,
}

#[derive(Debug, Clone)]
struct EvidenceInputs {
    manifest_path: Option<PathBuf>,
    report_path: Option<PathBuf>,
    run_summary_path: Option<PathBuf>,
    facts_path: Option<PathBuf>,
    graph_path: Option<PathBuf>,
    telemetry_paths: Vec<PathBuf>,
}

/// Build an evidence bundle from an existing run directory.
///
/// # Errors
/// Returns an error if required evidence inputs cannot be parsed.
pub fn build_evidence_bundle(base_dir: &Path, facts_path: Option<&Path>) -> Result<EvidenceBundleV1> {
    let inputs = discover_inputs(base_dir, facts_path);
    let manifest = load_optional_json(inputs.manifest_path.as_deref())
        .context("load evidence manifest")?;
    let report = load_optional_json(inputs.report_path.as_deref()).context("load evidence report")?;
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
        .or_else(|| summary.as_ref().and_then(|value| value.stage_rows.first().map(|row| row.run_id.as_str())))
        .or_else(|| facts.as_ref().and_then(|rows| rows.first().map(|row| row.run_id.as_str())))
        .unwrap_or("unknown-run")
        .to_string();
    let correlation_id = manifest
        .as_ref()
        .and_then(|value| value.get("correlation_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| run_id.clone());

    let timeline = build_timeline(
        &correlation_id,
        manifest.as_ref(),
        &telemetry,
        summary.as_ref(),
        report.as_ref(),
    );
    let artifacts = collect_artifacts(base_dir, manifest.as_ref());
    let provenance_graph = build_provenance_graph(manifest.as_ref(), summary.as_ref(), facts.as_deref(), &artifacts);
    let health = build_health(base_dir, &inputs, manifest.as_ref(), report.as_ref(), summary.as_ref(), facts.as_deref(), &artifacts);
    let metrics = build_metrics(&timeline, report.as_ref(), summary.as_ref(), facts.as_deref(), &health);
    let compact_summary = build_compact_summary(summary.as_ref(), manifest.as_ref(), facts.as_deref(), &artifacts, &health);

    Ok(EvidenceBundleV1 {
        schema_version: "bijux.evidence_bundle.v1".to_string(),
        run_id,
        correlation_id,
        sources: EvidenceSourcesV1 {
            manifest_path: to_relative_string(base_dir, inputs.manifest_path.as_deref()),
            report_path: to_relative_string(base_dir, inputs.report_path.as_deref()),
            run_summary_path: to_relative_string(base_dir, inputs.run_summary_path.as_deref()),
            facts_path: to_relative_string(base_dir, inputs.facts_path.as_deref()),
            graph_path: to_relative_string(base_dir, inputs.graph_path.as_deref()),
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
    })
}

/// Write a deterministic evidence bundle JSON to the run root.
///
/// # Errors
/// Returns an error if bundle construction or writing fails.
pub fn write_evidence_bundle_json(base_dir: &Path, facts_path: Option<&Path>) -> Result<PathBuf> {
    let bundle = build_evidence_bundle(base_dir, facts_path)?;
    let path = base_dir.join("evidence_bundle.json");
    bijux_dna_infra::atomic_write_json(&path, &bundle)
        .with_context(|| format!("write evidence bundle {}", path.display()))?;
    Ok(path)
}

/// Verify an evidence bundle and its referenced sources/artifacts.
///
/// # Errors
/// Returns an error if the bundle cannot be read or parsed.
pub fn verify_evidence_bundle(bundle_path: &Path) -> Result<EvidenceVerificationV1> {
    let bundle_raw = std::fs::read_to_string(bundle_path)
        .with_context(|| format!("read evidence bundle {}", bundle_path.display()))?;
    let bundle: EvidenceBundleV1 = serde_json::from_str(&bundle_raw)
        .with_context(|| format!("parse evidence bundle {}", bundle_path.display()))?;
    let base_dir = bundle_path
        .parent()
        .ok_or_else(|| anyhow!("evidence bundle missing parent directory"))?;

    let mut checks = Vec::new();
    let mut missing_paths = Vec::new();
    for (label, path) in [
        ("manifest_path", bundle.sources.manifest_path.as_deref()),
        ("report_path", bundle.sources.report_path.as_deref()),
        ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
        ("facts_path", bundle.sources.facts_path.as_deref()),
        ("graph_path", bundle.sources.graph_path.as_deref()),
    ] {
        if let Some(path) = path {
            let full = base_dir.join(path);
            let ok = full.exists();
            if !ok {
                missing_paths.push(path.to_string());
            }
            checks.push(EvidenceCheckV1 {
                check_id: label.to_string(),
                ok,
                message: if ok {
                    format!("{label} present")
                } else {
                    format!("{label} missing at {path}")
                },
            });
        }
    }
    for path in &bundle.sources.telemetry_paths {
        let full = base_dir.join(path);
        let ok = full.exists();
        if !ok {
            missing_paths.push(path.clone());
        }
        checks.push(EvidenceCheckV1 {
            check_id: format!("telemetry:{}", path),
            ok,
            message: if ok {
                format!("telemetry source present at {path}")
            } else {
                format!("telemetry source missing at {path}")
            },
        });
    }

    for artifact in &bundle.artifacts {
        let full = base_dir.join(&artifact.path);
        let ok = full.exists();
        if !ok {
            missing_paths.push(artifact.path.clone());
        }
        let hash_ok = match (&artifact.sha256, ok) {
            (Some(expected), true) => bijux_dna_infra::hash_file_sha256(&full)
                .map(|actual| actual == *expected)
                .unwrap_or(false),
            (None, true) => true,
            (_, false) => false,
        };
        checks.push(EvidenceCheckV1 {
            check_id: format!("artifact:{}", artifact.name),
            ok: ok && hash_ok,
            message: if ok && hash_ok {
                format!("artifact {} verified", artifact.name)
            } else if ok {
                format!("artifact {} hash mismatch", artifact.name)
            } else {
                format!("artifact {} missing at {}", artifact.name, artifact.path)
            },
        });
    }

    let verified = checks.iter().all(|check| check.ok) && bundle.health.auditable;
    Ok(EvidenceVerificationV1 {
        schema_version: "bijux.evidence_verification.v1".to_string(),
        verified,
        checks,
        missing_paths,
        gap_count: bundle.health.gaps.len(),
    })
}

/// Compare two evidence bundles to surface runtime, artifact, and audit drift.
///
/// # Errors
/// Returns an error if either bundle cannot be read or parsed.
pub fn compare_evidence_bundles(left: &Path, right: &Path) -> Result<EvidenceComparisonV1> {
    let left_bundle = read_bundle(left)?;
    let right_bundle = read_bundle(right)?;
    let left_stages: BTreeSet<_> = left_bundle.compact_summary.stage_ids.iter().cloned().collect();
    let right_stages: BTreeSet<_> = right_bundle.compact_summary.stage_ids.iter().cloned().collect();
    let changed_stage_ids: Vec<String> = left_stages
        .symmetric_difference(&right_stages)
        .cloned()
        .collect();

    let left_artifacts: BTreeMap<_, _> = left_bundle
        .artifacts
        .iter()
        .map(|artifact| (artifact.name.clone(), artifact.sha256.clone()))
        .collect();
    let right_artifacts: BTreeMap<_, _> = right_bundle
        .artifacts
        .iter()
        .map(|artifact| (artifact.name.clone(), artifact.sha256.clone()))
        .collect();
    let changed_artifacts: Vec<String> = left_artifacts
        .keys()
        .chain(right_artifacts.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|name| left_artifacts.get(*name) != right_artifacts.get(*name))
        .map(|name| name.to_string())
        .collect();

    let mut policy_change_hints = Vec::new();
    if !changed_stage_ids.is_empty() {
        policy_change_hints.push("stage set changed between evidence bundles".to_string());
    }
    if !changed_artifacts.is_empty() {
        policy_change_hints.push("artifact inventory or hashes changed between evidence bundles".to_string());
    }
    if left_bundle.correlation_id != right_bundle.correlation_id {
        policy_change_hints.push("correlation identifiers differ across compared runs".to_string());
    }

    Ok(EvidenceComparisonV1 {
        schema_version: "bijux.evidence_comparison.v1".to_string(),
        left_run_id: left_bundle.run_id,
        right_run_id: right_bundle.run_id,
        left_correlation_id: left_bundle.correlation_id,
        right_correlation_id: right_bundle.correlation_id,
        changed_stage_ids,
        changed_artifacts,
        runtime_delta_s: right_bundle.metrics.run_time_s - left_bundle.metrics.run_time_s,
        evidence_gap_delta: right_bundle.health.gaps.len() as i64 - left_bundle.health.gaps.len() as i64,
        policy_change_hints,
    })
}

fn discover_inputs(base_dir: &Path, facts_path: Option<&Path>) -> EvidenceInputs {
    EvidenceInputs {
        manifest_path: first_existing(base_dir, &["run_manifest.json", "execution_manifest.json"]),
        report_path: first_existing(base_dir, &["report.json"]),
        run_summary_path: first_existing(base_dir, &["run_summary.json", "summary/run_summary.json"]),
        facts_path: facts_path
            .map(Path::to_path_buf)
            .or_else(|| first_existing(base_dir, &["facts.jsonl", "summary/facts.jsonl"])),
        graph_path: first_existing(base_dir, &["graph.json", "run_artifacts/graph.json"]),
        telemetry_paths: find_telemetry_paths(base_dir),
    }
}

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
        if let Some(artifacts) = manifest.get("output_artifacts").and_then(serde_json::Value::as_array) {
            for artifact in artifacts {
                timeline.push(EvidenceTimelineEventV1 {
                    category: EvidenceTimelineCategoryV1::Artifact,
                    event: "artifact_manifest_entry".to_string(),
                    timestamp: None,
                    stage_id: artifact.get("stage_id").and_then(serde_json::Value::as_str).map(str::to_string),
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
            message: "telemetry timeline is incomplete because no telemetry jsonl files were found".to_string(),
            path: None,
            blocks_audit: true,
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
                        message: format!("artifact {} does not match its declared hash", artifact.name),
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
    let failed_stage_count = facts
        .map(|rows| rows.iter().filter(|row| row.exit_code != 0).count() as u64)
        .unwrap_or(0);
    let queue_time_ms = queue_time_from_timeline(timeline);
    let cache_hit_count = timeline
        .iter()
        .filter(|event| {
            matches!(event.category, EvidenceTimelineCategoryV1::Cache) && event.status == "hit"
        })
        .count() as u64;
    let cache_miss_count = timeline
        .iter()
        .filter(|event| {
            matches!(event.category, EvidenceTimelineCategoryV1::Cache)
                && (event.status == "missing" || event.status == "miss")
        })
        .count() as u64;
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
        scientific_failure_classes.insert("evidence_gap".to_string(), audit_gaps as u64);
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
        cache_hit_count,
        cache_miss_count,
        total_timeline_events: timeline.len() as u64,
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
    let final_outputs = summary
        .map(|summary| summary.final_outputs.clone())
        .unwrap_or_default();
    let failed_stage_count = facts
        .map(|rows| rows.iter().filter(|row| row.exit_code != 0).count())
        .unwrap_or(0);
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
        if let Some(inputs) = manifest.get("dataset_fingerprints").and_then(serde_json::Value::as_array) {
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
        push_node(
            &mut nodes,
            &mut seen_nodes,
            format!("stage:{stage}"),
            "stage",
            stage.clone(),
        );
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

fn collect_artifacts(base_dir: &Path, manifest: Option<&serde_json::Value>) -> Vec<EvidenceArtifactV1> {
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
            let sha256 = entry.get("sha256").and_then(serde_json::Value::as_str).map(str::to_string);
            artifacts.push(EvidenceArtifactV1 { name, path, sha256 });
        }
    }
    artifacts.sort_by(|left, right| (left.name.as_str(), left.path.as_str()).cmp(&(right.name.as_str(), right.path.as_str())));
    artifacts.dedup_by(|left, right| left.name == right.name && left.path == right.path);
    if artifacts.is_empty() {
        let bundle_path = base_dir.join("evidence_bundle.json");
        if bundle_path.exists() {
            artifacts.push(EvidenceArtifactV1 {
                name: "evidence_bundle".to_string(),
                path: "evidence_bundle.json".to_string(),
                sha256: bijux_dna_infra::hash_file_sha256(&bundle_path).ok(),
            });
        }
    }
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

fn read_bundle(path: &Path) -> Result<EvidenceBundleV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read evidence bundle {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse evidence bundle {}", path.display()))
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
    candidates
        .iter()
        .map(|candidate| base_dir.join(candidate))
        .find(|path| path.exists())
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
    path.strip_prefix(base_dir)
        .unwrap_or(path)
        .display()
        .to_string()
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
    let planner = timeline.iter().find(|event| matches!(event.category, EvidenceTimelineCategoryV1::Planner))?;
    let execution = timeline
        .iter()
        .find(|event| matches!(event.category, EvidenceTimelineCategoryV1::Execution))?;
    let planner_ts = planner.timestamp.as_deref()?;
    let execution_ts = execution.timestamp.as_deref()?;
    let planner = chrono::DateTime::parse_from_rfc3339(planner_ts).ok()?;
    let execution = chrono::DateTime::parse_from_rfc3339(execution_ts).ok()?;
    let millis = (execution - planner).num_milliseconds();
    (millis >= 0).then_some(millis as u64)
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

impl EvidenceHealthV1 {
    fn health_gap_count(&self) -> usize {
        self.gaps.len()
    }
}
