use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION: &str =
    "bijux.vcf.scientific_drift.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VcfScientificDriftChangeKind {
    DefaultsChange,
    BackendChange,
    FilterPolicyChange,
    NormalizationPolicyChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftSnapshotV1 {
    pub label: String,
    pub stage_id: String,
    pub tool_id: String,
    pub backend_version: Option<String>,
    pub defaults_fingerprint: Option<String>,
    pub normalization_policy_id: Option<String>,
    pub filter_policy_id: Option<String>,
    pub metrics: BTreeMap<String, f64>,
    pub artifacts: BTreeMap<String, String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftMetricDeltaV1 {
    pub metric_id: String,
    pub baseline_value: f64,
    pub candidate_value: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftArtifactDeltaV1 {
    pub artifact_id: String,
    pub baseline_hash: Option<String>,
    pub candidate_hash: Option<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub baseline_tool_id: String,
    pub candidate_tool_id: String,
    pub change_kinds: Vec<VcfScientificDriftChangeKind>,
    pub metric_deltas: Vec<VcfScientificDriftMetricDeltaV1>,
    pub artifact_deltas: Vec<VcfScientificDriftArtifactDeltaV1>,
    pub downstream_risks: Vec<String>,
    pub caveats: Vec<String>,
}

#[must_use]
pub fn build_vcf_scientific_drift_report(
    baseline: &VcfScientificDriftSnapshotV1,
    candidate: &VcfScientificDriftSnapshotV1,
) -> VcfScientificDriftReportV1 {
    let mut metric_ids =
        baseline.metrics.keys().chain(candidate.metrics.keys()).cloned().collect::<Vec<_>>();
    metric_ids.sort();
    metric_ids.dedup();
    let metric_deltas = metric_ids
        .into_iter()
        .filter_map(|metric_id| {
            let baseline_value = *baseline.metrics.get(&metric_id)?;
            let candidate_value = *candidate.metrics.get(&metric_id)?;
            let absolute_delta = candidate_value - baseline_value;
            let relative_delta = if baseline_value.abs() > f64::EPSILON {
                Some(absolute_delta / baseline_value)
            } else {
                None
            };
            Some(VcfScientificDriftMetricDeltaV1 {
                metric_id,
                baseline_value,
                candidate_value,
                absolute_delta,
                relative_delta,
            })
        })
        .collect::<Vec<_>>();

    let mut artifact_ids =
        baseline.artifacts.keys().chain(candidate.artifacts.keys()).cloned().collect::<Vec<_>>();
    artifact_ids.sort();
    artifact_ids.dedup();
    let artifact_deltas = artifact_ids
        .into_iter()
        .map(|artifact_id| {
            let baseline_hash = baseline.artifacts.get(&artifact_id).cloned();
            let candidate_hash = candidate.artifacts.get(&artifact_id).cloned();
            VcfScientificDriftArtifactDeltaV1 {
                artifact_id,
                changed: baseline_hash != candidate_hash,
                baseline_hash,
                candidate_hash,
            }
        })
        .collect::<Vec<_>>();

    let mut change_kinds = Vec::new();
    if baseline.defaults_fingerprint != candidate.defaults_fingerprint {
        change_kinds.push(VcfScientificDriftChangeKind::DefaultsChange);
    }
    if baseline.backend_version != candidate.backend_version
        || baseline.tool_id != candidate.tool_id
    {
        change_kinds.push(VcfScientificDriftChangeKind::BackendChange);
    }
    if baseline.filter_policy_id != candidate.filter_policy_id {
        change_kinds.push(VcfScientificDriftChangeKind::FilterPolicyChange);
    }
    if baseline.normalization_policy_id != candidate.normalization_policy_id {
        change_kinds.push(VcfScientificDriftChangeKind::NormalizationPolicyChange);
    }

    let mut downstream_risks = Vec::new();
    if metric_deltas.iter().any(|delta| delta.metric_id == "variants_total" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("variant_count_shift".to_string());
    }
    if metric_deltas.iter().any(|delta| delta.metric_id == "annotation_coverage" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("annotation_coverage_shift".to_string());
    }
    if metric_deltas.iter().any(|delta| delta.metric_id == "missingness_post" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("cohort_readiness_shift".to_string());
    }
    if artifact_deltas.iter().any(|delta| delta.changed) {
        downstream_risks.push("artifact_identity_shift".to_string());
    }
    downstream_risks.sort();
    downstream_risks.dedup();

    let mut caveats = baseline.caveats.clone();
    caveats.extend(candidate.caveats.clone());
    if change_kinds.contains(&VcfScientificDriftChangeKind::DefaultsChange) {
        caveats.push(
            "default-setting drift detected; compare downstream cohort and variant summaries before promotion"
                .to_string(),
        );
    }
    if change_kinds.contains(&VcfScientificDriftChangeKind::BackendChange) {
        caveats.push(
            "backend drift detected; metric shifts may combine scientific and implementation causes"
                .to_string(),
        );
    }
    caveats.sort();
    caveats.dedup();

    VcfScientificDriftReportV1 {
        schema_version: VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: baseline.stage_id.clone(),
        baseline_label: baseline.label.clone(),
        candidate_label: candidate.label.clone(),
        baseline_tool_id: baseline.tool_id.clone(),
        candidate_tool_id: candidate.tool_id.clone(),
        change_kinds,
        metric_deltas,
        artifact_deltas,
        downstream_risks,
        caveats,
    }
}
