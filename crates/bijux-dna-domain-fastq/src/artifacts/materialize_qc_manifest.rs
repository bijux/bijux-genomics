use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.materialize_qc_manifest.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QcManifestEntryV1 {
    pub source_path: String,
    pub source_sha256: String,
    pub stage_id: Option<String>,
    pub tool_id: Option<String>,
    pub schema_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaterializeQcManifestReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub report_count: u64,
    pub reads_in_total: Option<u64>,
    pub reads_out_total: Option<u64>,
    pub bases_in_total: Option<u64>,
    pub bases_out_total: Option<u64>,
    pub entries: Vec<QcManifestEntryV1>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        MaterializeQcManifestReportV1, QcManifestEntryV1,
        MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn materialize_qc_manifest_report_round_trips() {
        let report = MaterializeQcManifestReportV1 {
            schema_version: MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.materialize_qc_manifest".to_string(),
            stage_id: "fastq.materialize_qc_manifest".to_string(),
            tool_id: "bijux".to_string(),
            report_count: 1,
            reads_in_total: Some(100),
            reads_out_total: Some(90),
            bases_in_total: Some(10_000),
            bases_out_total: Some(8_900),
            entries: vec![QcManifestEntryV1 {
                source_path: "qc/report.json".to_string(),
                source_sha256: "abc123".to_string(),
                stage_id: Some("fastq.trim_reads".to_string()),
                tool_id: Some("fastp".to_string()),
                schema_version: Some("bijux.fastq.trim_reads.report.v1".to_string()),
            }],
            warnings: Vec::new(),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: MaterializeQcManifestReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.report_count, 1);
        assert_eq!(decoded.reads_out_total, Some(90));
    }
}
