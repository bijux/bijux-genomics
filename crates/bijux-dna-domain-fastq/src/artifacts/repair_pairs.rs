use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const REPAIR_PAIRS_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.repair_pairs.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RepairPairsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub reads_in_r1: u64,
    pub reads_in_r2: u64,
    pub retained_pairs: u64,
    pub rescued_pairs: u64,
    pub singleton_r1: u64,
    pub singleton_r2: u64,
    pub rejected_records: u64,
    pub retained_r1: String,
    pub retained_r2: String,
    pub rescued_r1: String,
    pub rescued_r2: String,
    pub singleton_r1_path: String,
    pub singleton_r2_path: String,
    pub rejected_path: String,
}

#[cfg(test)]
mod tests {
    use super::{RepairPairsReportV1, REPAIR_PAIRS_REPORT_SCHEMA_VERSION};

    #[test]
    fn repair_pairs_report_round_trips() {
        let report = RepairPairsReportV1 {
            schema_version: REPAIR_PAIRS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.repair_pairs".to_string(),
            stage_id: "fastq.repair_pairs".to_string(),
            tool_id: "bijux".to_string(),
            reads_in_r1: 100,
            reads_in_r2: 95,
            retained_pairs: 80,
            rescued_pairs: 10,
            singleton_r1: 10,
            singleton_r2: 5,
            rejected_records: 0,
            retained_r1: "retained_R1.fastq.gz".to_string(),
            retained_r2: "retained_R2.fastq.gz".to_string(),
            rescued_r1: "rescued_R1.fastq.gz".to_string(),
            rescued_r2: "rescued_R2.fastq.gz".to_string(),
            singleton_r1_path: "singletons_R1.fastq.gz".to_string(),
            singleton_r2_path: "singletons_R2.fastq.gz".to_string(),
            rejected_path: "rejected.fastq.gz".to_string(),
        };
        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: RepairPairsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.rescued_pairs, 10);
    }
}
