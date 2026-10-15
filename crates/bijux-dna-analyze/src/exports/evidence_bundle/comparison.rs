//! Owner: bijux-dna-analyze
//! Acquire two bundles and compare their completed-run evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{EvidenceBundleV1, EvidenceComparisonV1};

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

fn read_bundle(path: &Path) -> Result<EvidenceBundleV1> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read evidence bundle {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse evidence bundle {}", path.display()))
}
