use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_runtime::{FactsRowV1, TelemetryEventName, TelemetryEventV1};
use chrono::Utc;
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBundleProfileV1 {
    Draft,
    Operational,
    Certification,
    Publication,
    PublicationStrict,
    CollaboratorRedacted,
    ArchiveRetention,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceCitationTypeV1 {
    Tool,
    Reference,
    Defaults,
    Method,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCitationV1 {
    pub citation_id: String,
    pub citation: String,
    pub citation_type: EvidenceCitationTypeV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMethodsToolV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    pub params_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceMethodsSummaryV1 {
    pub schema_version: String,
    pub run_id: String,
    pub correlation_id: String,
    pub stage_count: usize,
    #[serde(default)]
    pub tools: Vec<EvidenceMethodsToolV1>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
    pub citation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceBundleFileDigestV1 {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArchiveMigrationV1 {
    pub manifest_schema_version: String,
    pub evidence_schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_inventory_schema_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_ledger_schema_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileBundleV1 {
    pub schema_version: String,
    pub profile: EvidenceBundleProfileV1,
    pub generated_at: String,
    pub run_id: String,
    pub correlation_id: String,
    pub evidence_bundle: EvidenceBundleV1,
    pub evidence_verification: EvidenceVerificationV1,
    pub profile_validation: EvidenceProfileValidationV1,
    #[serde(default)]
    pub required_files: Vec<EvidenceBundleFileDigestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_migration: Option<EvidenceArchiveMigrationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileBundleVerificationV1 {
    pub schema_version: String,
    pub verified: bool,
    #[serde(default)]
    pub missing_paths: Vec<String>,
    #[serde(default)]
    pub hash_mismatches: Vec<String>,
    pub evidence_verified: bool,
    pub profile_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerChallengeRequestV1 {
    pub artifact_id: String,
    pub evidence_path: String,
    pub report_field: String,
    pub caveat: String,
    pub question: String,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerChallengeRecordV1 {
    pub schema_version: String,
    pub challenge_id: String,
    pub created_at: String,
    pub artifact_id: String,
    pub evidence_path: String,
    pub report_field: String,
    pub caveat: String,
    pub question: String,
    pub requested_by: String,
    pub state: String,
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
    pub plan_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_summary_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_policy_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_state_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_descriptor_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_inventory_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_manifest_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash_ledger_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_verification_path: Option<String>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<EvidenceCitationV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub methods_summary: Option<EvidenceMethodsSummaryV1>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileCheckV1 {
    pub check_id: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceProfileValidationV1 {
    pub schema_version: String,
    pub profile: EvidenceBundleProfileV1,
    pub ok: bool,
    pub required_paths_present: bool,
    pub tolerated_gap_codes: Vec<String>,
    pub blocking_gap_codes: Vec<String>,
    pub checks: Vec<EvidenceProfileCheckV1>,
}

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
    let artifacts = collect_artifacts(base_dir, manifest.as_ref());
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

/// Write a deterministic methods-summary JSON generated from evidence inputs.
///
/// # Errors
/// Returns an error if evidence construction or writing fails.
pub fn write_methods_summary_json(base_dir: &Path, facts_path: Option<&Path>) -> Result<PathBuf> {
    let bundle = build_evidence_bundle(base_dir, facts_path)?;
    let Some(summary) = bundle.methods_summary else {
        return Err(anyhow!("methods summary generation failed"));
    };
    let path = base_dir.join("methods_summary.json");
    bijux_dna_infra::atomic_write_json(&path, &summary)
        .with_context(|| format!("write methods summary {}", path.display()))?;
    Ok(path)
}

/// Write a profile-specific bundle suitable for publication, collaboration, or retention archives.
///
/// # Errors
/// Returns an error if evidence construction, verification, or writing fails.
pub fn write_profile_bundle_json(
    base_dir: &Path,
    facts_path: Option<&Path>,
    profile: EvidenceBundleProfileV1,
) -> Result<PathBuf> {
    let bundle = build_profile_bundle(base_dir, facts_path, profile)?;
    let path = base_dir.join(profile_bundle_file_name(profile));
    bijux_dna_infra::atomic_write_json(&path, &bundle)
        .with_context(|| format!("write profile bundle {}", path.display()))?;
    Ok(path)
}

/// Verify a profile bundle in an external checkout.
///
/// # Errors
/// Returns an error if bundle parsing fails.
pub fn verify_profile_bundle(
    profile_bundle_path: &Path,
) -> Result<EvidenceProfileBundleVerificationV1> {
    let raw = std::fs::read_to_string(profile_bundle_path)
        .with_context(|| format!("read {}", profile_bundle_path.display()))?;
    let bundle: EvidenceProfileBundleV1 = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", profile_bundle_path.display()))?;
    let base_dir =
        profile_bundle_path.parent().ok_or_else(|| anyhow!("profile bundle path has no parent"))?;

    let mut missing_paths = Vec::new();
    let mut hash_mismatches = Vec::new();
    for item in &bundle.required_files {
        let full = base_dir.join(&item.path);
        if !full.exists() {
            missing_paths.push(item.path.clone());
            continue;
        }
        let actual = bijux_dna_infra::hash_file_sha256(&full)
            .with_context(|| format!("hash required file {}", full.display()))?;
        if actual != item.sha256 {
            hash_mismatches.push(item.path.clone());
        }
    }

    let verified = missing_paths.is_empty()
        && hash_mismatches.is_empty()
        && bundle.evidence_verification.verified
        && bundle.profile_validation.ok;
    Ok(EvidenceProfileBundleVerificationV1 {
        schema_version: "bijux.profile_bundle_verification.v1".to_string(),
        verified,
        missing_paths,
        hash_mismatches,
        evidence_verified: bundle.evidence_verification.verified,
        profile_valid: bundle.profile_validation.ok,
    })
}

/// Submit a reviewer challenge tied to a governed evidence/report location.
///
/// # Errors
/// Returns an error if required evidence files are missing or challenge references are invalid.
pub fn submit_reviewer_challenge(
    base_dir: &Path,
    request: &ReviewerChallengeRequestV1,
) -> Result<ReviewerChallengeRecordV1> {
    if request.artifact_id.trim().is_empty() {
        return Err(anyhow!("artifact_id cannot be empty"));
    }
    if request.evidence_path.trim().is_empty() {
        return Err(anyhow!("evidence_path cannot be empty"));
    }
    if request.report_field.trim().is_empty() {
        return Err(anyhow!("report_field cannot be empty"));
    }
    if request.caveat.trim().is_empty() {
        return Err(anyhow!("caveat cannot be empty"));
    }
    if request.question.trim().is_empty() {
        return Err(anyhow!("question cannot be empty"));
    }
    if request.requested_by.trim().is_empty() {
        return Err(anyhow!("requested_by cannot be empty"));
    }

    let artifact_inventory_path = base_dir.join("artifact_inventory.json");
    let report_path = base_dir.join("report.json");
    let evidence_bundle_path = base_dir.join("evidence_bundle.json");
    if !artifact_inventory_path.exists() {
        return Err(anyhow!(
            "reviewer challenge requires artifact inventory at {}",
            artifact_inventory_path.display()
        ));
    }
    if !report_path.exists() {
        return Err(anyhow!("reviewer challenge requires report at {}", report_path.display()));
    }
    if !evidence_bundle_path.exists() {
        return Err(anyhow!(
            "reviewer challenge requires evidence bundle at {}",
            evidence_bundle_path.display()
        ));
    }

    let inventory: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_inventory_path)
            .with_context(|| format!("read {}", artifact_inventory_path.display()))?,
    )
    .with_context(|| format!("parse {}", artifact_inventory_path.display()))?;
    let artifact_exists =
        inventory.get("artifacts").and_then(serde_json::Value::as_array).is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some(request.artifact_id.as_str())
            })
        });
    if !artifact_exists {
        return Err(anyhow!(
            "artifact_id `{}` is not present in artifact_inventory.json",
            request.artifact_id
        ));
    }

    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .with_context(|| format!("read {}", report_path.display()))?,
    )
    .with_context(|| format!("parse {}", report_path.display()))?;
    if report_field(&report, &request.report_field).is_none() {
        return Err(anyhow!(
            "report field `{}` is not present in report.json",
            request.report_field
        ));
    }
    let evidence_relative = validate_reviewer_evidence_path(base_dir, &request.evidence_path)?;

    let bundle: EvidenceBundleV1 = serde_json::from_str(
        &std::fs::read_to_string(&evidence_bundle_path)
            .with_context(|| format!("read {}", evidence_bundle_path.display()))?,
    )
    .with_context(|| format!("parse {}", evidence_bundle_path.display()))?;
    let caveat_matches = bundle
        .health
        .gaps
        .iter()
        .any(|gap| gap.code == request.caveat || gap.message.contains(&request.caveat));
    if !caveat_matches {
        return Err(anyhow!(
            "caveat `{}` does not match any evidence gap code/message",
            request.caveat
        ));
    }

    let mut existing = list_reviewer_challenges(base_dir)?;
    existing.sort_by(|left, right| left.challenge_id.cmp(&right.challenge_id));
    let sequence = existing.len() + 1;
    let challenge_id = format!(
        "challenge-{}-{}-{:04}",
        Utc::now().format("%Y%m%d%H%M%S%3f"),
        sanitized_suffix(&request.artifact_id),
        sequence
    );
    let record = ReviewerChallengeRecordV1 {
        schema_version: "bijux.reviewer_challenge.v1".to_string(),
        challenge_id,
        created_at: Utc::now().to_rfc3339(),
        artifact_id: request.artifact_id.clone(),
        evidence_path: evidence_relative,
        report_field: request.report_field.clone(),
        caveat: request.caveat.clone(),
        question: request.question.clone(),
        requested_by: request.requested_by.clone(),
        state: "open".to_string(),
    };
    append_reviewer_challenge(base_dir, &record)?;
    Ok(record)
}

/// Load reviewer challenges from the run root.
///
/// # Errors
/// Returns an error if the challenge log exists but cannot be parsed.
pub fn list_reviewer_challenges(base_dir: &Path) -> Result<Vec<ReviewerChallengeRecordV1>> {
    let path = base_dir.join("reviewer_challenges.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let challenge_log =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in challenge_log.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let challenge: ReviewerChallengeRecordV1 = serde_json::from_str(line)
            .with_context(|| format!("parse {} line {}", path.display(), index + 1))?;
        rows.push(challenge);
    }
    Ok(rows)
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
    let base_dir =
        bundle_path.parent().ok_or_else(|| anyhow!("evidence bundle missing parent directory"))?;

    let mut checks = Vec::new();
    let mut missing_paths = Vec::new();
    for (label, path) in [
        ("manifest_path", bundle.sources.manifest_path.as_deref()),
        ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
        ("report_path", bundle.sources.report_path.as_deref()),
        ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
        ("facts_path", bundle.sources.facts_path.as_deref()),
        ("graph_path", bundle.sources.graph_path.as_deref()),
        ("environment_path", bundle.sources.environment_path.as_deref()),
        ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
        ("run_state_path", bundle.sources.run_state_path.as_deref()),
        ("executor_descriptor_path", bundle.sources.executor_descriptor_path.as_deref()),
        ("checkpoint_path", bundle.sources.checkpoint_path.as_deref()),
        ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
        ("replay_manifest_path", bundle.sources.replay_manifest_path.as_deref()),
        ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
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
            check_id: format!("telemetry:{path}"),
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

    if let Some(path) = bundle.sources.artifact_inventory_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_artifact_inventory_contract(&full);
        checks.push(EvidenceCheckV1 {
            check_id: "artifact_inventory_contract".to_string(),
            ok,
            message,
        });
    }
    if let Some(path) = bundle.sources.hash_ledger_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_hash_ledger_contract(base_dir, &full);
        checks.push(EvidenceCheckV1 { check_id: "hash_ledger_contract".to_string(), ok, message });
    }
    if let Some(path) = bundle.sources.report_path.as_deref() {
        let full = base_dir.join(path);
        let (ok, message) = verify_report_completeness(&full);
        checks.push(EvidenceCheckV1 { check_id: "report_completeness".to_string(), ok, message });
    }
    if let (Some(manifest_path), Some(run_state_path)) =
        (bundle.sources.manifest_path.as_deref(), bundle.sources.run_state_path.as_deref())
    {
        let (ok, message) = verify_advisory_and_enforced_consistency(
            &base_dir.join(manifest_path),
            &base_dir.join(run_state_path),
        );
        checks.push(EvidenceCheckV1 {
            check_id: "mode_state_consistency".to_string(),
            ok,
            message,
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
    let right_stages: BTreeSet<_> =
        right_bundle.compact_summary.stage_ids.iter().cloned().collect();
    let changed_stage_ids: Vec<String> =
        left_stages.symmetric_difference(&right_stages).cloned().collect();

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
        .map(std::string::ToString::to_string)
        .collect();

    let mut policy_change_hints = Vec::new();
    if !changed_stage_ids.is_empty() {
        policy_change_hints.push("stage set changed between evidence bundles".to_string());
    }
    if !changed_artifacts.is_empty() {
        policy_change_hints
            .push("artifact inventory or hashes changed between evidence bundles".to_string());
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
        evidence_gap_delta: i64::try_from(right_bundle.health.gaps.len()).unwrap_or(i64::MAX)
            - i64::try_from(left_bundle.health.gaps.len()).unwrap_or(i64::MAX),
        policy_change_hints,
    })
}

#[must_use]
pub fn validate_evidence_bundle_profile(
    bundle: &EvidenceBundleV1,
    profile: EvidenceBundleProfileV1,
) -> EvidenceProfileValidationV1 {
    let mut checks = Vec::new();
    let mut required_paths_present = true;
    for (check_id, ok, message) in profile_requirements(bundle, profile) {
        if !ok {
            required_paths_present = false;
        }
        checks.push(EvidenceProfileCheckV1 { check_id, ok, message });
    }

    let tolerated_gap_codes = tolerated_gap_codes(profile);
    let (tolerated, blocking): (Vec<_>, Vec<_>) = bundle
        .health
        .gaps
        .iter()
        .map(|gap| gap.code.clone())
        .partition(|code| tolerated_gap_codes.contains(code));
    let ok = required_paths_present && blocking.is_empty();

    EvidenceProfileValidationV1 {
        schema_version: "bijux.evidence_profile_validation.v1".to_string(),
        profile,
        ok,
        required_paths_present,
        tolerated_gap_codes: tolerated,
        blocking_gap_codes: blocking,
        checks,
    }
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

fn collect_artifacts(
    base_dir: &Path,
    manifest: Option<&serde_json::Value>,
) -> Vec<EvidenceArtifactV1> {
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
                serde_json::Value::String(single) => vec![single.to_string()],
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

fn build_profile_bundle(
    base_dir: &Path,
    facts_path: Option<&Path>,
    profile: EvidenceBundleProfileV1,
) -> Result<EvidenceProfileBundleV1> {
    let evidence_bundle_path = write_evidence_bundle_json(base_dir, facts_path)?;
    let raw = std::fs::read_to_string(&evidence_bundle_path)
        .with_context(|| format!("read {}", evidence_bundle_path.display()))?;
    let source_bundle: EvidenceBundleV1 = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", evidence_bundle_path.display()))?;
    let evidence_verification = verify_evidence_bundle(&evidence_bundle_path)?;
    let profile_validation = validate_evidence_bundle_profile(&source_bundle, profile);
    let required_files = build_required_files(base_dir, &source_bundle)?;
    let archive_migration = matches!(profile, EvidenceBundleProfileV1::ArchiveRetention)
        .then(|| build_archive_migration(base_dir, &source_bundle))
        .transpose()?;
    let signature_path = base_dir
        .join("bundle_signature.json")
        .exists()
        .then_some("bundle_signature.json".to_string());
    let evidence_bundle = if matches!(profile, EvidenceBundleProfileV1::CollaboratorRedacted) {
        redact_for_collaborator(source_bundle.clone())
    } else {
        source_bundle.clone()
    };
    Ok(EvidenceProfileBundleV1 {
        schema_version: "bijux.profile_bundle.v1".to_string(),
        profile,
        generated_at: Utc::now().to_rfc3339(),
        run_id: evidence_bundle.run_id.clone(),
        correlation_id: evidence_bundle.correlation_id.clone(),
        evidence_bundle,
        evidence_verification,
        profile_validation,
        required_files,
        signature_path,
        archive_migration,
    })
}

fn build_required_files(
    base_dir: &Path,
    bundle: &EvidenceBundleV1,
) -> Result<Vec<EvidenceBundleFileDigestV1>> {
    let mut required = BTreeSet::new();
    for path in [
        bundle.sources.manifest_path.as_deref(),
        bundle.sources.plan_manifest_path.as_deref(),
        bundle.sources.report_path.as_deref(),
        bundle.sources.run_summary_path.as_deref(),
        bundle.sources.facts_path.as_deref(),
        bundle.sources.environment_path.as_deref(),
        bundle.sources.runtime_policy_path.as_deref(),
        bundle.sources.run_state_path.as_deref(),
        bundle.sources.executor_descriptor_path.as_deref(),
        bundle.sources.checkpoint_path.as_deref(),
        bundle.sources.failure_path.as_deref(),
        bundle.sources.artifact_inventory_path.as_deref(),
        bundle.sources.replay_manifest_path.as_deref(),
        bundle.sources.hash_ledger_path.as_deref(),
        bundle.sources.evidence_verification_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        required.insert(path.to_string());
    }
    required.extend(bundle.sources.telemetry_paths.iter().cloned());
    required.extend(bundle.artifacts.iter().map(|artifact| artifact.path.clone()));
    if base_dir.join("bundle_signature.json").exists() {
        required.insert("bundle_signature.json".to_string());
    }
    if base_dir.join("reviewer_challenges.jsonl").exists() {
        required.insert("reviewer_challenges.jsonl".to_string());
    }
    if base_dir.join("reviewer_challenges.latest.json").exists() {
        required.insert("reviewer_challenges.latest.json".to_string());
    }

    let mut rows = Vec::new();
    for rel in required {
        let full = base_dir.join(&rel);
        if !full.exists() {
            continue;
        }
        rows.push(EvidenceBundleFileDigestV1 {
            path: rel,
            sha256: bijux_dna_infra::hash_file_sha256(&full)
                .with_context(|| format!("hash {}", full.display()))?,
        });
    }
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(rows)
}

fn redact_for_collaborator(bundle: EvidenceBundleV1) -> EvidenceBundleV1 {
    fn redact_path(path: Option<&str>) -> Option<String> {
        path.map(std::path::Path::new)
            .and_then(|path| path.file_name().and_then(|name| name.to_str()))
            .map(str::to_string)
    }
    let mut redacted = bundle;
    redacted.sources.manifest_path = redact_path(redacted.sources.manifest_path.as_deref());
    redacted.sources.plan_manifest_path =
        redact_path(redacted.sources.plan_manifest_path.as_deref());
    redacted.sources.report_path = redact_path(redacted.sources.report_path.as_deref());
    redacted.sources.run_summary_path = redact_path(redacted.sources.run_summary_path.as_deref());
    redacted.sources.facts_path = redact_path(redacted.sources.facts_path.as_deref());
    redacted.sources.graph_path = redact_path(redacted.sources.graph_path.as_deref());
    redacted.sources.environment_path = redact_path(redacted.sources.environment_path.as_deref());
    redacted.sources.runtime_policy_path =
        redact_path(redacted.sources.runtime_policy_path.as_deref());
    redacted.sources.run_state_path = redact_path(redacted.sources.run_state_path.as_deref());
    redacted.sources.executor_descriptor_path =
        redact_path(redacted.sources.executor_descriptor_path.as_deref());
    redacted.sources.checkpoint_path = redact_path(redacted.sources.checkpoint_path.as_deref());
    redacted.sources.failure_path = redact_path(redacted.sources.failure_path.as_deref());
    redacted.sources.artifact_inventory_path =
        redact_path(redacted.sources.artifact_inventory_path.as_deref());
    redacted.sources.replay_manifest_path =
        redact_path(redacted.sources.replay_manifest_path.as_deref());
    redacted.sources.hash_ledger_path = redact_path(redacted.sources.hash_ledger_path.as_deref());
    redacted.sources.evidence_verification_path =
        redact_path(redacted.sources.evidence_verification_path.as_deref());
    redacted.sources.telemetry_paths = redacted
        .sources
        .telemetry_paths
        .iter()
        .map(|value| {
            std::path::Path::new(value)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("redacted")
                .to_string()
        })
        .collect();
    for artifact in &mut redacted.artifacts {
        artifact.path = std::path::Path::new(&artifact.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("redacted")
            .to_string();
    }
    redacted
}

fn build_archive_migration(
    base_dir: &Path,
    bundle: &EvidenceBundleV1,
) -> Result<EvidenceArchiveMigrationV1> {
    let manifest_schema_version = load_schema_version(
        base_dir,
        bundle.sources.manifest_path.as_deref(),
        "bijux.run_manifest.v3",
    )?;
    let evidence_schema_version = bundle.schema_version.clone();
    let artifact_inventory_schema_version = load_schema_version(
        base_dir,
        bundle.sources.artifact_inventory_path.as_deref(),
        "bijux.artifact_inventory.v1",
    )?;
    let hash_ledger_schema_version = load_schema_version(
        base_dir,
        bundle.sources.hash_ledger_path.as_deref(),
        "bijux.hash_ledger.v1",
    )?;
    Ok(EvidenceArchiveMigrationV1 {
        manifest_schema_version,
        evidence_schema_version,
        artifact_inventory_schema_version: Some(artifact_inventory_schema_version),
        hash_ledger_schema_version: Some(hash_ledger_schema_version),
    })
}

fn load_schema_version(
    base_dir: &Path,
    relative_path: Option<&str>,
    fallback: &str,
) -> Result<String> {
    let Some(relative_path) = relative_path else {
        return Ok(fallback.to_string());
    };
    let full = base_dir.join(relative_path);
    let raw = std::fs::read_to_string(&full).with_context(|| format!("read {}", full.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", full.display()))?;
    Ok(value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(fallback)
        .to_string())
}

fn profile_bundle_file_name(profile: EvidenceBundleProfileV1) -> &'static str {
    match profile {
        EvidenceBundleProfileV1::Draft => "profile_bundle_draft.json",
        EvidenceBundleProfileV1::Operational => "profile_bundle_operational.json",
        EvidenceBundleProfileV1::Certification => "profile_bundle_certification.json",
        EvidenceBundleProfileV1::Publication => "profile_bundle_publication.json",
        EvidenceBundleProfileV1::PublicationStrict => "profile_bundle_publication_strict.json",
        EvidenceBundleProfileV1::CollaboratorRedacted => {
            "profile_bundle_collaborator_redacted.json"
        }
        EvidenceBundleProfileV1::ArchiveRetention => "profile_bundle_archive_retention.json",
    }
}

fn report_field<'a>(
    value: &'a serde_json::Value,
    dotted_path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in dotted_path.split('.').filter(|segment| !segment.is_empty()) {
        current = current.get(segment)?;
    }
    Some(current)
}

fn append_reviewer_challenge(base_dir: &Path, record: &ReviewerChallengeRecordV1) -> Result<()> {
    let path = base_dir.join("reviewer_challenges.jsonl");
    let mut rows = list_reviewer_challenges(base_dir)?;
    rows.push(record.clone());
    rows.sort_by(|left, right| left.challenge_id.cmp(&right.challenge_id));
    let payload = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n")
        + "\n";
    bijux_dna_infra::write_bytes(&path, payload)?;
    let latest_path = base_dir.join("reviewer_challenges.latest.json");
    bijux_dna_infra::atomic_write_json(&latest_path, record)
        .with_context(|| format!("write {}", latest_path.display()))?;
    Ok(())
}

fn validate_reviewer_evidence_path(base_dir: &Path, value: &str) -> Result<String> {
    let candidate = Path::new(value);
    if candidate.is_absolute() {
        return Err(anyhow!(
            "evidence_path must be repository-relative, got absolute path `{value}`"
        ));
    }
    if candidate.components().any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(anyhow!(
            "evidence_path must not contain parent traversal (`..`), got `{value}`"
        ));
    }
    let relative = candidate
        .to_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| anyhow!("evidence_path must be valid UTF-8"))?;
    let full = base_dir.join(relative);
    if !full.exists() {
        return Err(anyhow!("evidence_path `{relative}` is not present under run directory"));
    }
    Ok(relative.to_string())
}

fn sanitized_suffix(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == '-' {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "artifact".to_string()
    } else {
        out
    }
}

fn verify_artifact_inventory_contract(path: &Path) -> (bool, String) {
    let Ok((inventory, audit)) =
        bijux_dna_runtime::run_layout::read_supported_artifact_inventory(path)
    else {
        return (false, format!("artifact inventory missing at {}", path.display()));
    };
    let role_complete = inventory.artifacts.iter().all(|row| !row.role.trim().is_empty());
    let lineage_complete = inventory.artifacts.iter().all(|row| !row.input_lineage.is_empty());
    (
        role_complete && lineage_complete,
        if role_complete && lineage_complete {
            format!(
                "artifact inventory accepted via {} and lineage is complete",
                audit.from_schema_version
            )
        } else {
            format!(
                "artifact inventory accepted via {} but role or lineage detail is incomplete",
                audit.from_schema_version
            )
        },
    )
}

fn profile_requirements(
    bundle: &EvidenceBundleV1,
    profile: EvidenceBundleProfileV1,
) -> Vec<(String, bool, String)> {
    let required_paths = match profile {
        EvidenceBundleProfileV1::Draft => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Operational => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("run_state_path", bundle.sources.run_state_path.as_deref()),
            ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Certification => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("run_state_path", bundle.sources.run_state_path.as_deref()),
            ("runtime_policy_path", bundle.sources.runtime_policy_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::Publication | EvidenceBundleProfileV1::PublicationStrict => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("facts_path", bundle.sources.facts_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
        EvidenceBundleProfileV1::CollaboratorRedacted => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
        ],
        EvidenceBundleProfileV1::ArchiveRetention => vec![
            ("manifest_path", bundle.sources.manifest_path.as_deref()),
            ("plan_manifest_path", bundle.sources.plan_manifest_path.as_deref()),
            ("report_path", bundle.sources.report_path.as_deref()),
            ("run_summary_path", bundle.sources.run_summary_path.as_deref()),
            ("facts_path", bundle.sources.facts_path.as_deref()),
            ("artifact_inventory_path", bundle.sources.artifact_inventory_path.as_deref()),
            ("replay_manifest_path", bundle.sources.replay_manifest_path.as_deref()),
            ("hash_ledger_path", bundle.sources.hash_ledger_path.as_deref()),
            ("evidence_verification_path", bundle.sources.evidence_verification_path.as_deref()),
        ],
    };
    required_paths
        .into_iter()
        .map(|(check_id, value)| {
            let ok = value.is_some();
            (
                format!("{check_id}_required"),
                ok,
                if ok {
                    format!("{check_id} is present for {profile:?} evidence bundles")
                } else {
                    format!("{check_id} is required for {profile:?} evidence bundles")
                },
            )
        })
        .chain(std::iter::once((
            "telemetry_required".to_string(),
            !bundle.sources.telemetry_paths.is_empty(),
            if bundle.sources.telemetry_paths.is_empty() {
                format!("telemetry paths are required for {profile:?} evidence bundles")
            } else {
                format!("telemetry paths are present for {profile:?} evidence bundles")
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_methods_summary_required".to_string(),
            bundle.methods_summary.is_some(),
            if bundle.methods_summary.is_some() {
                "methods summary present for strict publication bundle".to_string()
            } else {
                "methods summary is required for strict publication bundle".to_string()
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_citations_required".to_string(),
            !bundle.citations.is_empty(),
            if bundle.citations.is_empty() {
                "citations are required for strict publication bundle".to_string()
            } else {
                "citations present for strict publication bundle".to_string()
            },
        )))
        .chain(matches!(profile, EvidenceBundleProfileV1::PublicationStrict).then_some((
            "strict_publication_no_gaps".to_string(),
            bundle.health.gaps.is_empty(),
            if bundle.health.gaps.is_empty() {
                "strict publication bundle has no evidence gaps".to_string()
            } else {
                "strict publication bundle must not contain evidence gaps".to_string()
            },
        )))
        .collect()
}

fn tolerated_gap_codes(profile: EvidenceBundleProfileV1) -> Vec<String> {
    match profile {
        EvidenceBundleProfileV1::Draft => vec![
            "missing_hash_ledger".to_string(),
            "missing_report".to_string(),
            "missing_run_summary".to_string(),
        ],
        EvidenceBundleProfileV1::Operational => vec!["missing_hash_ledger".to_string()],
        EvidenceBundleProfileV1::Certification
        | EvidenceBundleProfileV1::Publication
        | EvidenceBundleProfileV1::PublicationStrict
        | EvidenceBundleProfileV1::CollaboratorRedacted
        | EvidenceBundleProfileV1::ArchiveRetention => Vec::new(),
    }
}

fn verify_hash_ledger_contract(base_dir: &Path, path: &Path) -> (bool, String) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (false, format!("hash ledger missing at {}", path.display()));
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (false, format!("hash ledger is not valid json at {}", path.display()));
    };
    let schema_ok = value.get("schema_version").and_then(serde_json::Value::as_str)
        == Some("bijux.hash_ledger.v1");
    let entries = value.get("entries").and_then(serde_json::Value::as_array);
    let entries_ok = entries.is_some_and(|rows| {
        let mut previous = None;
        rows.iter().all(|row| {
            let Some(relative_path) = row.get("path").and_then(serde_json::Value::as_str) else {
                return false;
            };
            let Some(expected_hash) = row.get("sha256").and_then(serde_json::Value::as_str) else {
                return false;
            };
            let recorded_previous = row
                .get("previous_entry_sha256")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string);
            if recorded_previous != previous {
                return false;
            }
            let full = base_dir.join(relative_path);
            let Ok(actual_hash) = bijux_dna_infra::hash_file_sha256(&full) else {
                return false;
            };
            previous = Some(expected_hash.to_string());
            actual_hash == expected_hash
        })
    });
    (
        schema_ok && entries_ok,
        if schema_ok && entries_ok {
            "hash ledger ordering and file hashes verified".to_string()
        } else {
            "hash ledger failed schema or chained hash verification".to_string()
        },
    )
}

fn verify_report_completeness(path: &Path) -> (bool, String) {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return (false, format!("report missing at {}", path.display()));
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return (false, format!("report is not valid json at {}", path.display()));
    };
    let completeness = value.get("completeness");
    let missing_metrics_empty = completeness
        .and_then(|value| value.get("missing_metrics"))
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty);
    let missing_reports_empty = completeness
        .and_then(|value| value.get("missing_reports"))
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty);
    (
        missing_metrics_empty && missing_reports_empty,
        if missing_metrics_empty && missing_reports_empty {
            "report completeness is governed and empty of missing fields".to_string()
        } else {
            "report completeness declares missing metrics or reports".to_string()
        },
    )
}

fn verify_advisory_and_enforced_consistency(
    manifest_path: &Path,
    run_state_path: &Path,
) -> (bool, String) {
    let manifest = std::fs::read_to_string(manifest_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let run_state = std::fs::read_to_string(run_state_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    let manifest_mode =
        manifest.as_ref().and_then(|value| value.get("mode")).and_then(serde_json::Value::as_str);
    let state_mode =
        run_state.as_ref().and_then(|value| value.get("mode")).and_then(serde_json::Value::as_str);
    let ok = manifest_mode == state_mode;
    (
        ok,
        if ok {
            "manifest mode and run-state mode are consistent".to_string()
        } else {
            "manifest mode and run-state mode disagree".to_string()
        },
    )
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

impl EvidenceHealthV1 {
    fn health_gap_count(&self) -> usize {
        self.gaps.len()
    }
}
