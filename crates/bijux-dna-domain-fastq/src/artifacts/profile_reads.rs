use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::metrics::SeqkitToolMetricsV1;
use crate::params::PairedMode;

pub const PROFILE_READS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.profile_reads.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProfileReadsHistogramBinV1 {
    pub length: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProfileReadsMateSummaryV1 {
    pub label: String,
    pub reads: u64,
    pub bases: u64,
    pub mean_q: Option<f64>,
    pub gc_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProfileReadsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub qc_json: String,
    pub qc_tsv: String,
    pub qc_plots_dir: Option<String>,
    pub length_histogram_source: String,
    pub reads_total: u64,
    pub bases_total: u64,
    pub mean_q: f64,
    pub gc_percent: f64,
    pub length_histogram: Vec<ProfileReadsHistogramBinV1>,
    pub mate_summaries: Vec<ProfileReadsMateSummaryV1>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub exit_code: Option<i32>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub backend_metrics: Option<Vec<SeqkitToolMetricsV1>>,
}

#[cfg(test)]
mod tests {
    use super::{
        ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1,
        PROFILE_READS_REPORT_SCHEMA_VERSION,
    };
    use crate::metrics::SeqkitToolMetricsV1;
    use crate::params::PairedMode;

    #[test]
    fn profile_reads_report_contract_round_trips() {
        let report = ProfileReadsReportV1 {
            schema_version: PROFILE_READS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.profile_reads".to_string(),
            stage_id: "fastq.profile_reads".to_string(),
            tool_id: "seqkit_stats".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 2,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            qc_json: "qc.json".to_string(),
            qc_tsv: "qc.tsv".to_string(),
            qc_plots_dir: Some("plots".to_string()),
            length_histogram_source: "seqkit_fx2tab".to_string(),
            reads_total: 200,
            bases_total: 20_000,
            mean_q: 31.2,
            gc_percent: 42.0,
            length_histogram: vec![ProfileReadsHistogramBinV1 { length: 100, count: 200 }],
            mate_summaries: vec![
                ProfileReadsMateSummaryV1 {
                    label: "reads_r1".to_string(),
                    reads: 100,
                    bases: 10_000,
                    mean_q: Some(31.0),
                    gc_percent: Some(41.0),
                },
                ProfileReadsMateSummaryV1 {
                    label: "reads_r2".to_string(),
                    reads: 100,
                    bases: 10_000,
                    mean_q: Some(31.4),
                    gc_percent: Some(43.0),
                },
            ],
            runtime_s: Some(1.3),
            memory_mb: Some(24.0),
            exit_code: Some(0),
            raw_backend_report: Some("qc.tsv".to_string()),
            raw_backend_report_format: Some("seqkit_stats_tsv".to_string()),
            backend_metrics: Some(vec![
                SeqkitToolMetricsV1 {
                    schema_version: "bijux.seqkit.metrics.v1".to_string(),
                    reads: 100,
                    bases: 10_000,
                    mean_q: Some(31.0),
                    gc_percent: Some(41.0),
                },
                SeqkitToolMetricsV1 {
                    schema_version: "bijux.seqkit.metrics.v1".to_string(),
                    reads: 100,
                    bases: 10_000,
                    mean_q: Some(31.4),
                    gc_percent: Some(43.0),
                },
            ]),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ProfileReadsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "seqkit_stats");
        assert_eq!(decoded.paired_mode, PairedMode::PairedEnd);
        assert_eq!(decoded.length_histogram.len(), 1);
        assert_eq!(decoded.mate_summaries.len(), 2);
    }
}
