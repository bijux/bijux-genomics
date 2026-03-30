use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const INDEX_REFERENCE_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.index_reference.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IndexReferenceFileEntryV1 {
    pub relative_path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct IndexReferenceReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub threads: u32,
    pub index_format: String,
    pub reference_fasta: String,
    pub reference_bytes: u64,
    pub reference_index: String,
    pub report_json: String,
    pub index_prefix: Option<String>,
    pub emitted_files: Vec<IndexReferenceFileEntryV1>,
    pub index_file_count: u64,
    pub index_bytes: u64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{
        IndexReferenceFileEntryV1, IndexReferenceReportV1, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn index_reference_report_contract_round_trips() {
        let report = IndexReferenceReportV1 {
            schema_version: INDEX_REFERENCE_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.index_reference".to_string(),
            stage_id: "fastq.index_reference".to_string(),
            tool_id: "bowtie2_build".to_string(),
            threads: 4,
            index_format: "bowtie2_build".to_string(),
            reference_fasta: "reference.fa".to_string(),
            reference_bytes: 4096,
            reference_index: "reference_index/bowtie2/reference".to_string(),
            report_json: "index_reference_report.json".to_string(),
            index_prefix: Some("reference".to_string()),
            emitted_files: vec![
                IndexReferenceFileEntryV1 {
                    relative_path: "reference.1.bt2".to_string(),
                    bytes: 1024,
                },
                IndexReferenceFileEntryV1 {
                    relative_path: "reference.2.bt2".to_string(),
                    bytes: 2048,
                },
            ],
            index_file_count: 2,
            index_bytes: 3072,
            runtime_s: Some(1.5),
            memory_mb: Some(96.0),
            exit_code: Some(0),
            backend_metrics: Some(serde_json::json!({
                "index_directory": "reference_index/bowtie2",
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: IndexReferenceReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "bowtie2_build");
        assert_eq!(decoded.index_file_count, 2);
        assert_eq!(decoded.emitted_files[0].relative_path, "reference.1.bt2");
    }
}
