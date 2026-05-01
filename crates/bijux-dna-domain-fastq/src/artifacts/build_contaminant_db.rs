use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.build_contaminant_db.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildContaminantDbSourceEntryV1 {
    pub path: String,
    pub sha256: String,
    pub sequence_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildContaminantDbReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub database_family: String,
    pub source_sequence_count: u64,
    pub database_hash: String,
    pub sources: Vec<BuildContaminantDbSourceEntryV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        BuildContaminantDbReportV1, BuildContaminantDbSourceEntryV1,
        BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn build_contaminant_db_report_round_trips() {
        let report = BuildContaminantDbReportV1 {
            schema_version: BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.build_contaminant_db".to_string(),
            stage_id: "fastq.build_contaminant_db".to_string(),
            tool_id: "bijux".to_string(),
            database_family: "bowtie2".to_string(),
            source_sequence_count: 42,
            database_hash: "abc123".to_string(),
            sources: vec![BuildContaminantDbSourceEntryV1 {
                path: "contaminants.fa".to_string(),
                sha256: "deadbeef".to_string(),
                sequence_count: 42,
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: BuildContaminantDbReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.source_sequence_count, 42);
    }
}
