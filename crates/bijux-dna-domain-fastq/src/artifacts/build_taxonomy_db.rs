use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.build_taxonomy_db.report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildTaxonomyDbSourceEntryV1 {
    pub path: String,
    pub sha256: String,
    pub record_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BuildTaxonomyDbReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub database_family: String,
    pub source_record_count: u64,
    pub database_hash: String,
    pub sources: Vec<BuildTaxonomyDbSourceEntryV1>,
}

#[cfg(test)]
mod tests {
    use super::{
        BuildTaxonomyDbReportV1, BuildTaxonomyDbSourceEntryV1,
        BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION,
    };

    #[test]
    fn build_taxonomy_db_report_round_trips() {
        let report = BuildTaxonomyDbReportV1 {
            schema_version: BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.build_taxonomy_db".to_string(),
            stage_id: "fastq.build_taxonomy_db".to_string(),
            tool_id: "bijux".to_string(),
            database_family: "kraken2".to_string(),
            source_record_count: 200,
            database_hash: "abc123".to_string(),
            sources: vec![BuildTaxonomyDbSourceEntryV1 {
                path: "taxonomy.tsv".to_string(),
                sha256: "deadbeef".to_string(),
                record_count: 200,
            }],
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: BuildTaxonomyDbReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.source_record_count, 200);
    }
}
