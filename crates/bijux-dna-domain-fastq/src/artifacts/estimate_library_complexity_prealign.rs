use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.estimate_library_complexity_prealign.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EstimateLibraryComplexityPrealignReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub complexity_policy: String,
    pub estimate_method: String,
    pub modifies_reads: bool,
    pub advisory_only: bool,
    pub reads_in: u64,
    pub estimated_unique_fraction: f64,
    pub estimated_duplicate_fraction: f64,
    pub kmer_size: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::{
        EstimateLibraryComplexityPrealignReportV1,
        ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    #[test]
    fn estimate_library_complexity_report_round_trips() {
        let report = EstimateLibraryComplexityPrealignReportV1 {
            schema_version: ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.estimate_library_complexity_prealign".to_string(),
            stage_id: "fastq.estimate_library_complexity_prealign".to_string(),
            tool_id: "bijux".to_string(),
            paired_mode: PairedMode::SingleEnd,
            complexity_policy: "prealign_kmer".to_string(),
            estimate_method: "kmer_redundancy".to_string(),
            modifies_reads: false,
            advisory_only: true,
            reads_in: 100,
            estimated_unique_fraction: 0.82,
            estimated_duplicate_fraction: 0.18,
            kmer_size: Some(31),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: EstimateLibraryComplexityPrealignReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert!(!decoded.modifies_reads);
        assert_eq!(decoded.estimated_duplicate_fraction, 0.18);
    }
}
