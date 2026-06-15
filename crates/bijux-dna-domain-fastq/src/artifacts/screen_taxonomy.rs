use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::{
    screen::{
        TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
        TaxonomyInterpretationBoundary, TaxonomyReportFormat, TaxonomyTruthCondition,
    },
    PairedMode,
};

pub const SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION: &str = "bijux.fastq.screen_taxonomy.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TaxonomyScreenSummaryEntryV1 {
    pub label: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ScreenTaxonomyReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub classifier: TaxonomyClassifier,
    pub report_format: TaxonomyReportFormat,
    pub assignment_format: TaxonomyAssignmentFormat,
    pub database_catalog_id: String,
    pub database_artifact_id: String,
    pub database_build_id: Option<String>,
    pub database_digest: Option<String>,
    pub database_namespace: Option<String>,
    pub database_scope: TaxonomyDatabaseScope,
    pub minimum_confidence: Option<f32>,
    pub emit_unclassified: bool,
    pub interpretation_boundary: TaxonomyInterpretationBoundary,
    #[serde(default)]
    pub truth_conditions: Vec<TaxonomyTruthCondition>,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub screen_report_tsv: String,
    pub classification_report_json: String,
    pub unclassified_reads_r1: Option<String>,
    pub unclassified_reads_r2: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
    pub contamination_rate: Option<f64>,
    pub classified_fraction: Option<f64>,
    pub unclassified_fraction: Option<f64>,
    pub summary_entries: Vec<TaxonomyScreenSummaryEntryV1>,
    pub top_taxa: Vec<TaxonomyScreenSummaryEntryV1>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::{
        ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
    };
    use crate::params::{
        screen::{
            TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
            TaxonomyInterpretationBoundary, TaxonomyReportFormat, TaxonomyTruthCondition,
        },
        PairedMode,
    };

    #[test]
    fn screen_taxonomy_report_contract_round_trips() {
        let report = ScreenTaxonomyReportV1 {
            schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.screen_taxonomy".to_string(),
            stage_id: "fastq.screen_taxonomy".to_string(),
            tool_id: "kraken2".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 8,
            classifier: TaxonomyClassifier::Kraken2,
            report_format: TaxonomyReportFormat::KrakenReport,
            assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
            database_catalog_id: "taxonomy_reference".to_string(),
            database_artifact_id: "taxonomy_db".to_string(),
            database_build_id: Some("2026.03".to_string()),
            database_digest: Some("sha256:taxonomy-db".to_string()),
            database_namespace: Some("read_screening".to_string()),
            database_scope: TaxonomyDatabaseScope::ReadScreening,
            minimum_confidence: Some(0.2),
            emit_unclassified: true,
            interpretation_boundary: TaxonomyInterpretationBoundary::ScreeningOnly,
            truth_conditions: vec![
                TaxonomyTruthCondition::LockedReferenceDatabase,
                TaxonomyTruthCondition::OrthogonalValidationRequired,
            ],
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            screen_report_tsv: "kraken2.report.tsv".to_string(),
            classification_report_json: "kraken2.classifications.json".to_string(),
            unclassified_reads_r1: Some("kraken2.unclassified_reads_1.fastq".to_string()),
            unclassified_reads_r2: Some("kraken2.unclassified_reads_2.fastq".to_string()),
            reads_in: Some(200),
            reads_out: Some(200),
            bases_in: Some(20_000),
            bases_out: Some(20_000),
            pairs_in: Some(100),
            pairs_out: Some(100),
            contamination_rate: Some(0.18),
            classified_fraction: Some(0.18),
            unclassified_fraction: Some(0.82),
            summary_entries: vec![
                TaxonomyScreenSummaryEntryV1 { label: "Homo sapiens".to_string(), percent: 12.5 },
                TaxonomyScreenSummaryEntryV1 { label: "unclassified".to_string(), percent: 82.0 },
            ],
            top_taxa: vec![TaxonomyScreenSummaryEntryV1 {
                label: "Homo sapiens".to_string(),
                percent: 12.5,
            }],
            runtime_s: Some(15.2),
            memory_mb: Some(512.0),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: ScreenTaxonomyReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "kraken2");
        assert_eq!(decoded.classifier, TaxonomyClassifier::Kraken2);
        assert_eq!(decoded.top_taxa.len(), 1);
        assert_eq!(decoded.interpretation_boundary, TaxonomyInterpretationBoundary::ScreeningOnly);
    }
}
