use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.prepare_adapter_bank.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PrepareAdapterBankReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub bank_id: String,
    pub bank_version: String,
    pub bank_hash: String,
    pub presets_hash: String,
    pub preset: String,
    pub preset_hash: String,
    pub enabled_categories: Vec<String>,
    pub disabled_categories: Vec<String>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub enabled_adapter_ids: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::{PrepareAdapterBankReportV1, PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION};

    #[test]
    fn prepare_adapter_bank_report_round_trips() {
        let report = PrepareAdapterBankReportV1 {
            schema_version: PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.prepare_adapter_bank".to_string(),
            stage_id: "fastq.prepare_adapter_bank".to_string(),
            tool_id: "bijux".to_string(),
            bank_id: "illumina".to_string(),
            bank_version: "2026.04".to_string(),
            bank_hash: "sha256:bank".to_string(),
            presets_hash: "sha256:presets".to_string(),
            preset: "illumina-default".to_string(),
            preset_hash: "sha256:preset".to_string(),
            enabled_categories: vec!["illumina".to_string()],
            disabled_categories: vec!["polyA".to_string()],
            enable_adapters: vec!["AGATCGGAAGAGC".to_string()],
            disable_adapters: vec!["polyA".to_string()],
            enabled_adapter_ids: vec!["illumina_r1".to_string(), "illumina_r2".to_string()],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: PrepareAdapterBankReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.stage_id, "fastq.prepare_adapter_bank");
        assert_eq!(decoded.enabled_adapter_ids.len(), 2);
    }
}
