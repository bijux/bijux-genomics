use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.prepare_host_reference_bundle.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HostReferenceBundleFileV1 {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrepareHostReferenceBundleReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reference_build: String,
    pub bundle_hash: String,
    pub bundle_file_count: u64,
    pub files: Vec<HostReferenceBundleFileV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        HostReferenceBundleFileV1, PrepareHostReferenceBundleReportV1,
        PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn prepare_host_reference_bundle_report_round_trips() {
        let report = PrepareHostReferenceBundleReportV1 {
            schema_version: PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.prepare_host_reference_bundle".to_string(),
            stage_id: "fastq.prepare_host_reference_bundle".to_string(),
            tool_id: "bijux".to_string(),
            reference_build: "hg38".to_string(),
            bundle_hash: "abc123".to_string(),
            bundle_file_count: 1,
            files: vec![HostReferenceBundleFileV1 {
                path: "refs/hg38.fa".to_string(),
                sha256: "deadbeef".to_string(),
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: PrepareHostReferenceBundleReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.bundle_file_count, 1);
        assert_eq!(decoded.reference_build, "hg38");
    }
}
