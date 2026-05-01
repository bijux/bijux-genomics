use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VERIFY_ASSETS_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.verify_assets.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetVerificationStatusV1 {
    Verified,
    Missing,
    Mismatch,
    InvalidLock,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AssetVerificationEntryV1 {
    pub lock_path: String,
    pub asset_path: Option<String>,
    pub expected_sha256: Option<String>,
    pub observed_sha256: Option<String>,
    pub status: AssetVerificationStatusV1,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VerifyAssetsReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub verified_asset_count: u64,
    pub missing_asset_count: u64,
    pub mismatched_asset_count: u64,
    pub invalid_lock_count: u64,
    pub entries: Vec<AssetVerificationEntryV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        AssetVerificationEntryV1, AssetVerificationStatusV1, VerifyAssetsReportV1,
        VERIFY_ASSETS_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn verify_assets_report_round_trips() {
        let report = VerifyAssetsReportV1 {
            schema_version: VERIFY_ASSETS_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.verify_assets".to_string(),
            stage_id: "fastq.verify_assets".to_string(),
            tool_id: "bijux".to_string(),
            verified_asset_count: 1,
            missing_asset_count: 0,
            mismatched_asset_count: 0,
            invalid_lock_count: 0,
            entries: vec![AssetVerificationEntryV1 {
                lock_path: "asset.lock.json".to_string(),
                asset_path: Some("asset.fa".to_string()),
                expected_sha256: Some("abc".to_string()),
                observed_sha256: Some("abc".to_string()),
                status: AssetVerificationStatusV1::Verified,
                reason: None,
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: VerifyAssetsReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.verified_asset_count, 1);
        assert_eq!(decoded.entries.len(), 1);
    }
}
