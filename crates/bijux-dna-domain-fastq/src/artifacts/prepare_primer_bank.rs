use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.prepare_primer_bank.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PreparePrimerBankReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub bank_id: String,
    pub bank_version: String,
    pub governance_hash: String,
    pub selection_logic: String,
    pub primer_set_count: u64,
    pub primer_set_ids: Vec<String>,
    pub assay_family: String,
}

#[cfg(test)]
mod tests {
    use super::{PreparePrimerBankReportV1, PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION};

    #[test]
    fn prepare_primer_bank_report_round_trips() {
        let report = PreparePrimerBankReportV1 {
            schema_version: PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.prepare_primer_bank".to_string(),
            stage_id: "fastq.prepare_primer_bank".to_string(),
            tool_id: "bijux".to_string(),
            bank_id: "primer-bank".to_string(),
            bank_version: "2026.04".to_string(),
            governance_hash: "abc123".to_string(),
            selection_logic: "select explicit marker-specific primer set".to_string(),
            primer_set_count: 2,
            primer_set_ids: vec!["16S_universal_v1".to_string(), "ITS2_plant_v1".to_string()],
            assay_family: "marker_amplicon".to_string(),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: PreparePrimerBankReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.primer_set_count, 2);
        assert_eq!(decoded.stage_id, "fastq.prepare_primer_bank");
    }
}
