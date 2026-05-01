use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.scientific_drift.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScientificDriftChangeKind {
    DefaultsChange,
    BackendVersionChange,
    ToolSelectionChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScientificDriftSnapshotV1 {
    pub label: String,
    pub stage_id: String,
    pub tool_id: String,
    pub backend_version: Option<String>,
    pub defaults_fingerprint: Option<String>,
    pub metrics: BTreeMap<String, f64>,
    pub artifacts: BTreeMap<String, String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScientificDriftMetricDeltaV1 {
    pub metric_id: String,
    pub baseline_value: f64,
    pub candidate_value: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScientificDriftArtifactDeltaV1 {
    pub artifact_id: String,
    pub baseline_hash: Option<String>,
    pub candidate_hash: Option<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FastqScientificDriftReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub baseline_tool_id: String,
    pub candidate_tool_id: String,
    pub change_kinds: Vec<ScientificDriftChangeKind>,
    pub metric_deltas: Vec<ScientificDriftMetricDeltaV1>,
    pub artifact_deltas: Vec<ScientificDriftArtifactDeltaV1>,
    pub caveats: Vec<String>,
}

#[must_use]
pub fn build_fastq_scientific_drift_report(
    baseline: &ScientificDriftSnapshotV1,
    candidate: &ScientificDriftSnapshotV1,
) -> FastqScientificDriftReportV1 {
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
            Some(ScientificDriftMetricDeltaV1 {
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
            ScientificDriftArtifactDeltaV1 {
                artifact_id,
                changed: baseline_hash != candidate_hash,
                baseline_hash,
                candidate_hash,
            }
        })
        .collect::<Vec<_>>();

    let mut change_kinds = Vec::new();
    if baseline.defaults_fingerprint != candidate.defaults_fingerprint {
        change_kinds.push(ScientificDriftChangeKind::DefaultsChange);
    }
    if baseline.backend_version != candidate.backend_version {
        change_kinds.push(ScientificDriftChangeKind::BackendVersionChange);
    }
    if baseline.tool_id != candidate.tool_id {
        change_kinds.push(ScientificDriftChangeKind::ToolSelectionChange);
    }

    let mut caveats = baseline.caveats.clone();
    caveats.extend(candidate.caveats.clone());
    if change_kinds.contains(&ScientificDriftChangeKind::DefaultsChange) {
        caveats.push(
            "default-setting drift detected; downstream scientific summaries must be compared cautiously"
                .to_string(),
        );
    }
    if change_kinds.contains(&ScientificDriftChangeKind::BackendVersionChange) {
        caveats.push(
            "backend version drift detected; observed metric shifts may blend scientific and implementation effects"
                .to_string(),
        );
    }
    caveats.sort();
    caveats.dedup();

    FastqScientificDriftReportV1 {
        schema_version: FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: baseline.stage_id.clone(),
        baseline_label: baseline.label.clone(),
        candidate_label: candidate.label.clone(),
        baseline_tool_id: baseline.tool_id.clone(),
        candidate_tool_id: candidate.tool_id.clone(),
        change_kinds,
        metric_deltas,
        artifact_deltas,
        caveats,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_fastq_scientific_drift_report, FastqScientificDriftReportV1,
        ScientificDriftArtifactDeltaV1, ScientificDriftChangeKind, ScientificDriftSnapshotV1,
        FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;

    #[test]
    fn scientific_drift_report_highlights_default_and_backend_changes() {
        let baseline = ScientificDriftSnapshotV1 {
            label: "baseline".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            backend_version: Some("0.23.4".to_string()),
            defaults_fingerprint: Some("defaults-a".to_string()),
            metrics: BTreeMap::from([
                ("read_retention".to_string(), 0.91),
                ("mean_q".to_string(), 31.0),
            ]),
            artifacts: BTreeMap::from([("report_json".to_string(), "sha256:a".to_string())]),
            caveats: vec!["adapter-heavy samples bias read-retention deltas".to_string()],
        };
        let candidate = ScientificDriftSnapshotV1 {
            label: "candidate".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            backend_version: Some("0.24.0".to_string()),
            defaults_fingerprint: Some("defaults-b".to_string()),
            metrics: BTreeMap::from([
                ("read_retention".to_string(), 0.88),
                ("mean_q".to_string(), 32.4),
            ]),
            artifacts: BTreeMap::from([("report_json".to_string(), "sha256:b".to_string())]),
            caveats: vec!["mean quality increases can accompany stronger read loss".to_string()],
        };

        let report = build_fastq_scientific_drift_report(&baseline, &candidate);
        assert_eq!(report.schema_version, FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION);
        assert!(report.change_kinds.contains(&ScientificDriftChangeKind::DefaultsChange));
        assert!(report.change_kinds.contains(&ScientificDriftChangeKind::BackendVersionChange));
        assert_eq!(report.metric_deltas.len(), 2);
        assert!(report.artifact_deltas.contains(&ScientificDriftArtifactDeltaV1 {
            artifact_id: "report_json".to_string(),
            baseline_hash: Some("sha256:a".to_string()),
            candidate_hash: Some("sha256:b".to_string()),
            changed: true,
        }));
        assert!(report
            .caveats
            .iter()
            .any(|caveat| caveat.contains("default-setting drift detected")));
    }

    #[test]
    fn scientific_drift_report_round_trips() {
        let report = FastqScientificDriftReportV1 {
            schema_version: FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            baseline_label: "baseline".to_string(),
            candidate_label: "candidate".to_string(),
            baseline_tool_id: "fastp".to_string(),
            candidate_tool_id: "cutadapt".to_string(),
            change_kinds: vec![ScientificDriftChangeKind::ToolSelectionChange],
            metric_deltas: Vec::new(),
            artifact_deltas: Vec::new(),
            caveats: vec!["compare only when stage schemas match".to_string()],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: FastqScientificDriftReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.candidate_tool_id, "cutadapt");
        assert_eq!(decoded.change_kinds.len(), 1);
    }
}
