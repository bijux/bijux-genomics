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

impl ScreenTaxonomyReportV1 {
    /// Derive classified and unclassified read counts from the observed fractions.
    ///
    /// Prefer input reads over output reads and the unclassified fraction over
    /// the classified fraction. Round and bound the inferred count to the total,
    /// then derive its complement. Missing totals or both fractions yield no counts.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
    pub fn derived_read_counts(&self) -> (Option<u64>, Option<u64>) {
        let total_reads = self.reads_in.or(self.reads_out);
        match (total_reads, self.unclassified_fraction, self.classified_fraction) {
            (Some(total_reads), Some(unclassified_fraction), _) => {
                let unclassified_reads = ((total_reads as f64) * unclassified_fraction)
                    .round()
                    .clamp(0.0, total_reads as f64) as u64;
                let classified_reads = total_reads.saturating_sub(unclassified_reads);
                (Some(classified_reads), Some(unclassified_reads))
            }
            (Some(total_reads), None, Some(classified_fraction)) => {
                let classified_reads = ((total_reads as f64) * classified_fraction)
                    .round()
                    .clamp(0.0, total_reads as f64) as u64;
                let unclassified_reads = total_reads.saturating_sub(classified_reads);
                (Some(classified_reads), Some(unclassified_reads))
            }
            _ => (None, None),
        }
    }
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

    fn report_with_counts(
        reads_in: Option<u64>,
        reads_out: Option<u64>,
        unclassified: Option<f64>,
        classified: Option<f64>,
    ) -> ScreenTaxonomyReportV1 {
        ScreenTaxonomyReportV1 {
            schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.screen_taxonomy".to_string(),
            stage_id: "fastq.screen_taxonomy".to_string(),
            tool_id: "kraken2".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 1,
            classifier: TaxonomyClassifier::Kraken2,
            report_format: TaxonomyReportFormat::KrakenReport,
            assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
            database_catalog_id: "taxonomy_reference".to_string(),
            database_artifact_id: "taxonomy_db".to_string(),
            database_build_id: None,
            database_digest: None,
            database_namespace: None,
            database_scope: TaxonomyDatabaseScope::ReadScreening,
            minimum_confidence: None,
            emit_unclassified: true,
            interpretation_boundary: TaxonomyInterpretationBoundary::ScreeningOnly,
            truth_conditions: Vec::new(),
            input_r1: "reads.fastq".to_string(),
            input_r2: None,
            screen_report_tsv: "screen.tsv".to_string(),
            classification_report_json: "classification.json".to_string(),
            unclassified_reads_r1: None,
            unclassified_reads_r2: None,
            reads_in,
            reads_out,
            bases_in: None,
            bases_out: None,
            pairs_in: None,
            pairs_out: None,
            contamination_rate: None,
            classified_fraction: classified,
            unclassified_fraction: unclassified,
            summary_entries: Vec::new(),
            top_taxa: Vec::new(),
            runtime_s: None,
            memory_mb: None,
        }
    }

    #[test]
    fn derived_counts_prefer_input_total_and_unclassified_fraction() {
        let report = report_with_counts(Some(100), Some(40), Some(0.23), Some(0.99));
        assert_eq!(report.derived_read_counts(), (Some(77), Some(23)));
        let empty_input = report_with_counts(Some(0), Some(100), Some(0.23), Some(0.99));
        assert_eq!(empty_input.derived_read_counts(), (Some(0), Some(0)));
    }

    #[test]
    fn derived_counts_use_fallbacks_only_when_preferred_observations_are_absent() {
        let fallback = report_with_counts(None, Some(10), None, Some(0.65));
        assert_eq!(fallback.derived_read_counts(), (Some(7), Some(3)));
        let missing_total = report_with_counts(None, None, Some(0.2), Some(0.8));
        assert_eq!(missing_total.derived_read_counts(), (None, None));
        let missing_fraction = report_with_counts(Some(10), Some(5), None, None);
        assert_eq!(missing_fraction.derived_read_counts(), (None, None));
    }

    #[test]
    fn derived_counts_round_clamp_and_saturate_the_complement() {
        for (unclassified, expected) in [
            (0.249, (Some(8), Some(2))),
            (0.25, (Some(7), Some(3))),
            (-0.2, (Some(10), Some(0))),
            (1.5, (Some(0), Some(10))),
            (f64::INFINITY, (Some(0), Some(10))),
            (f64::NAN, (Some(10), Some(0))),
        ] {
            let report = report_with_counts(Some(10), None, Some(unclassified), None);
            assert_eq!(report.derived_read_counts(), expected, "fraction {unclassified}");
        }
        let large_total = report_with_counts(Some(u64::MAX), None, Some(1.0), None);
        assert_eq!(large_total.derived_read_counts(), (Some(0), Some(u64::MAX)));
        let classified_clamp = report_with_counts(Some(10), None, None, Some(1.5));
        assert_eq!(classified_clamp.derived_read_counts(), (Some(10), Some(0)));
    }
}
