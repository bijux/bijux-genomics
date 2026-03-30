use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.normalize_abundance.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct NormalizeAbundanceReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub method: String,
    pub input_table: String,
    pub normalized_abundance_tsv: String,
    pub expected_columns: Vec<String>,
    pub input_value_column: String,
    pub normalized_value_column: String,
    pub compositional_rule: String,
    pub scale_factor: Option<f64>,
    pub table_rows: u64,
    pub sample_count: u64,
    pub feature_count: u64,
    pub zero_fraction: f64,
    pub per_sample_sums: Vec<(String, f64)>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub used_fallback: bool,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION};

    #[test]
    fn normalize_abundance_report_contract_round_trips() {
        let report = NormalizeAbundanceReportV1 {
            schema_version: NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.normalize_abundance".to_string(),
            stage_id: "fastq.normalize_abundance".to_string(),
            tool_id: "seqkit".to_string(),
            method: "relative_abundance".to_string(),
            input_table: "otu_abundance.tsv".to_string(),
            normalized_abundance_tsv: "abundance_normalized.tsv".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "normalized_abundance".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
            scale_factor: None,
            table_rows: 12,
            sample_count: 3,
            feature_count: 4,
            zero_fraction: 0.25,
            per_sample_sums: vec![("sample_a".to_string(), 1.0), ("sample_b".to_string(), 1.0)],
            runtime_s: Some(1.2),
            memory_mb: Some(32.0),
            raw_backend_report: None,
            raw_backend_report_format: None,
            used_fallback: false,
            backend_metrics: Some(serde_json::json!({
                "normalization_rows": 12_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: NormalizeAbundanceReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.method, "relative_abundance");
        assert_eq!(decoded.feature_count, 4);
        assert_eq!(decoded.per_sample_sums[0].0, "sample_a");
    }
}
