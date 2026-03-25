use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CLUSTER_OTUS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.cluster_otus.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ClusterOtusReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub otu_identity: f64,
    pub threads: u32,
    pub input_reads: String,
    pub otu_table: String,
    pub otu_representatives: String,
    pub taxonomy_ready_fasta: String,
    pub taxonomy_ready_fastq: String,
    pub report_json: String,
    pub otu_count: u64,
    pub sample_count: u64,
    pub representative_sequence_count: u64,
    pub output_table_kind: String,
    pub used_fallback: bool,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{ClusterOtusReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION};

    #[test]
    fn cluster_otus_report_contract_round_trips() {
        let report = ClusterOtusReportV1 {
            schema_version: CLUSTER_OTUS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.cluster_otus".to_string(),
            stage_id: "fastq.cluster_otus".to_string(),
            tool_id: "vsearch".to_string(),
            otu_identity: 0.97,
            threads: 4,
            input_reads: "merged.fastq.gz".to_string(),
            otu_table: "otu_abundance.tsv".to_string(),
            otu_representatives: "otu_representatives.fasta".to_string(),
            taxonomy_ready_fasta: "taxonomy_ready.fasta".to_string(),
            taxonomy_ready_fastq: "taxonomy_ready.fastq".to_string(),
            report_json: "cluster_otus_report.json".to_string(),
            otu_count: 18,
            sample_count: 4,
            representative_sequence_count: 18,
            output_table_kind: "otu_abundance_table".to_string(),
            used_fallback: false,
            runtime_s: Some(3.4),
            memory_mb: Some(96.0),
            exit_code: Some(0),
            raw_backend_report: Some("otu_clusters.uc".to_string()),
            raw_backend_report_format: Some("vsearch_uc".to_string()),
            backend_metrics: Some(serde_json::json!({
                "cluster_memberships": 18_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ClusterOtusReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "vsearch");
        assert_eq!(decoded.otu_count, 18);
        assert_eq!(
            decoded.raw_backend_report_format.as_deref(),
            Some("vsearch_uc")
        );
    }
}
