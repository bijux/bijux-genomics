use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.capture_provenance_snapshot.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceStageEntryV1 {
    pub stage_id: String,
    pub tool_id: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceFileEntryV1 {
    pub path: String,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureProvenanceSnapshotReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub plan_manifest_path: String,
    pub plan_manifest_sha256: String,
    pub stages: Vec<ProvenanceStageEntryV1>,
    pub declared_inputs: Vec<ProvenanceFileEntryV1>,
    pub declared_assets: Vec<ProvenanceFileEntryV1>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        CaptureProvenanceSnapshotReportV1, ProvenanceFileEntryV1, ProvenanceStageEntryV1,
        CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn capture_provenance_snapshot_report_round_trips() {
        let report = CaptureProvenanceSnapshotReportV1 {
            schema_version: CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.capture_provenance_snapshot".to_string(),
            stage_id: "fastq.capture_provenance_snapshot".to_string(),
            tool_id: "bijux".to_string(),
            plan_manifest_path: "reports/plan.json".to_string(),
            plan_manifest_sha256: "abcd1234".to_string(),
            stages: vec![ProvenanceStageEntryV1 {
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: Some("fastqvalidator".to_string()),
                image: Some("bijuxdna/fastqvalidator".to_string()),
            }],
            declared_inputs: vec![ProvenanceFileEntryV1 {
                path: "reads.fastq.gz".to_string(),
                sha256: Some("eeee".to_string()),
            }],
            declared_assets: vec![ProvenanceFileEntryV1 {
                path: "adapter_bank.fa".to_string(),
                sha256: Some("ffff".to_string()),
            }],
            warnings: Vec::new(),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: CaptureProvenanceSnapshotReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.stages.len(), 1);
        assert_eq!(decoded.declared_inputs.len(), 1);
        assert_eq!(decoded.stage_id, "fastq.capture_provenance_snapshot");
    }
}
