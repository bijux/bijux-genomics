use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    screen::{RrnaReportFormat, RrnaScreeningEngine},
    PairedMode,
};

pub const DEPLETE_RRNA_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.deplete_rrna.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DepleteRrnaReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub rrna_db: Option<String>,
    pub database_artifact_id: String,
    pub database_build_id: Option<String>,
    pub database_digest: Option<String>,
    pub screening_engine: RrnaScreeningEngine,
    pub report_format: RrnaReportFormat,
    pub emit_removed_reads: bool,
    pub min_identity: Option<f64>,
    pub retained_read_role: String,
    pub rejected_read_role: String,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub rrna_report_tsv: String,
    pub rrna_report_json: String,
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub bases_removed: u64,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub rrna_fraction_removed: f64,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{DepleteRrnaReportV1, DEPLETE_RRNA_REPORT_SCHEMA_VERSION};
    use crate::params::{
        screen::{RrnaReportFormat, RrnaScreeningEngine},
        PairedMode,
    };

    #[test]
    fn deplete_rrna_report_contract_round_trips() {
        let report = DepleteRrnaReportV1 {
            schema_version: DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.deplete_rrna".to_string(),
            stage_id: "fastq.deplete_rrna".to_string(),
            tool_id: "sortmerna".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 8,
            rrna_db: Some("/refs/silva".to_string()),
            database_artifact_id: "silva_nr99".to_string(),
            database_build_id: Some("2026.03".to_string()),
            database_digest: Some("sha256:silva".to_string()),
            screening_engine: RrnaScreeningEngine::Sortmerna,
            report_format: RrnaReportFormat::SummaryTsvAndJson,
            emit_removed_reads: false,
            min_identity: Some(0.95),
            retained_read_role: "rrna_filtered_reads".to_string(),
            rejected_read_role: "removed_rrna_reads".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "rrna_filtered_R1.fastq.gz".to_string(),
            output_r2: Some("rrna_filtered_R2.fastq.gz".to_string()),
            rrna_report_tsv: "rrna_report.tsv".to_string(),
            rrna_report_json: "rrna_report.json".to_string(),
            reads_in: 200,
            reads_out: 150,
            reads_removed: 50,
            bases_in: 20_000,
            bases_out: 14_800,
            bases_removed: 5_200,
            pairs_in: Some(100),
            pairs_out: Some(75),
            rrna_fraction_removed: 0.25,
            runtime_s: Some(12.3),
            memory_mb: Some(256.0),
            exit_code: Some(0),
            raw_backend_report: Some("sortmerna.log".to_string()),
            raw_backend_report_format: Some("sortmerna_log".to_string()),
            backend_metrics: Some(serde_json::json!({
                "reads_removed": 50_u64,
            })),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: DepleteRrnaReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "sortmerna");
        assert_eq!(decoded.database_artifact_id, "silva_nr99");
        assert_eq!(decoded.database_digest.as_deref(), Some("sha256:silva"));
        assert_eq!(decoded.reads_removed, 50);
    }
}
