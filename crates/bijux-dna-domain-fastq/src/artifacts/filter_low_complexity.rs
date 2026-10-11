use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

/// Schema identity for the local smoke observation envelope.
pub const LOCAL_FILTER_LOW_COMPLEXITY_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.filter_low_complexity.local_smoke.report.v1";

/// Local smoke observation; planned tool identity and actual report producer remain distinct.
/// This envelope alone does not prove execution of the planned backend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LocalFilterLowComplexitySmokeReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub sample_id: String,
    pub planned_tool_id: String,
    pub report_tool_id: String,
    pub entropy_threshold: f64,
    pub polyx_threshold: Option<u32>,
    pub input_reads: u64,
    pub reads_removed_low_complexity: u64,
    pub output_reads: u64,
    pub filtered_fastq_gz: String,
    pub case_report_json: String,
    pub raw_backend_report: String,
}

pub const FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.filter_low_complexity.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FilterLowComplexityReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub report_json: String,
    pub entropy_threshold: Option<f64>,
    pub polyx_threshold: Option<u32>,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed_low_complexity: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{FilterLowComplexityReportV1, FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION};
    use crate::params::PairedMode;

    #[test]
    fn filter_low_complexity_report_contract_round_trips() {
        let report = FilterLowComplexityReportV1 {
            schema_version: FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.filter_low_complexity".to_string(),
            stage_id: "fastq.filter_low_complexity".to_string(),
            tool_id: "bbduk".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 8,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "filtered_R1.fastq.gz".to_string(),
            output_r2: Some("filtered_R2.fastq.gz".to_string()),
            report_json: "low_complexity_report.json".to_string(),
            entropy_threshold: Some(0.5),
            polyx_threshold: Some(20),
            reads_in: 100,
            reads_out: 92,
            reads_removed_low_complexity: 8,
            bases_in: 10_000,
            bases_out: 9_100,
            pairs_in: Some(50),
            pairs_out: Some(46),
            mean_q_before: 28.0,
            mean_q_after: 29.5,
            runtime_s: Some(1.8),
            memory_mb: Some(96.0),
            exit_code: Some(0),
            raw_backend_report: Some("bbduk.low_complexity.stats".to_string()),
            raw_backend_report_format: Some("bbduk_stats".to_string()),
            backend_metrics: Some(serde_json::json!({
                "reads_removed_reported": 8_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: FilterLowComplexityReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "bbduk");
        assert_eq!(decoded.reads_removed_low_complexity, 8);
        assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("bbduk_stats"));
    }
}

#[cfg(test)]
mod local_smoke_contract_tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn local_smoke_observation_preserves_wire_shape_and_producer_identity() -> Result<()> {
        let original: serde_json::Value = serde_json::from_str(
            r#"{
    "schema_version": "bijux.fastq.filter_low_complexity.local_smoke.report.v1",
    "stage_id": "fastq.filter_low_complexity",
    "sample_id": "authored_contract_value",
    "planned_tool_id": "planned_backend",
    "report_tool_id": "local_emitter",
    "entropy_threshold": 0.5,
    "polyx_threshold": null,
    "input_reads": 1,
    "reads_removed_low_complexity": 1,
    "output_reads": 1,
    "filtered_fastq_gz": "authored_contract_value",
    "case_report_json": "authored_contract_value",
    "raw_backend_report": "authored_contract_value"
}"#,
        )?;
        let report: LocalFilterLowComplexitySmokeReportV1 =
            serde_json::from_value(original.clone())?;
        assert_eq!(report.schema_version, LOCAL_FILTER_LOW_COMPLEXITY_SMOKE_REPORT_SCHEMA_VERSION);
        assert_ne!(report.planned_tool_id, report.report_tool_id);
        assert_eq!(serde_json::to_value(&report)?, original);
        let mut unsupported = original;
        unsupported["unrecognized"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<LocalFilterLowComplexitySmokeReportV1>(unsupported).is_err()
        );
        let schema =
            serde_json::to_value(schemars::schema_for!(LocalFilterLowComplexitySmokeReportV1))?;
        assert_eq!(schema["additionalProperties"], serde_json::json!(false));
        Ok(())
    }
}
