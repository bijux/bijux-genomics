//! Owner: bijux-dna-bench-model
//! Admission of expected result rows against acquired catalog coverage and governed baselines.

use crate::{AllDomainExpectedBenchmarkResultRow, BenchError};
use bijux_dna_core::id_catalog;
use std::collections::BTreeSet;

/// Validates expected result projection without acquiring domain catalogs again.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for missing or inconsistent governed rows.
pub fn validate_all_domain_expected_result_rows(
    rows: &[AllDomainExpectedBenchmarkResultRow],
    expected_fastq_bam_count: usize,
    expected_vcf_count: usize,
) -> Result<(), BenchError> {
    let unique_result_ids = rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    if unique_result_ids.len() != rows.len() {
        return Err(BenchError::InvalidObservation {
            reason: "all-domain expected-result table must keep a unique stable result_id per row"
                .to_string(),
        });
    }

    for row in rows {
        if row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.report_section.trim().is_empty()
        {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "all-domain expected-result row `{}` contains a blank required column",
                    row.result_id
                ),
            });
        }
        if row.expected_outputs.is_empty() {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "all-domain expected-result row `{}` is missing expected_outputs",
                    row.result_id
                ),
            });
        }
        if row.expected_metrics.is_empty() {
            return Err(BenchError::InvalidObservation {
                reason: format!(
                    "all-domain expected-result row `{}` is missing expected_metrics",
                    row.result_id
                ),
            });
        }
    }

    let expected_count =
        expected_fastq_bam_count.checked_add(expected_vcf_count).ok_or_else(|| {
            BenchError::InvalidObservation {
                reason: "all-domain expected-result source row counts overflow".to_string(),
            }
        })?;
    if rows.len() != expected_count {
        return Err(BenchError::InvalidObservation { reason: format!(
            "all-domain expected-result table must retain every benchmark-ready FASTQ, BAM, and VCF row, found {} of {}",
            rows.len(), expected_count,
        ) });
    }
    for expected in governed_results() {
        require_expected_row(rows, &expected)?;
    }
    Ok(())
}

struct GovernedExpectedResult {
    result_id: &'static str,
    domain: &'static str,
    stage_id: &'static str,
    tool_id: &'static str,
    corpus_id: &'static str,
    asset_profile_id: &'static str,
    expected_output: &'static str,
    expected_metric: &'static str,
    report_section: &'static str,
}

fn governed_results() -> [GovernedExpectedResult; 3] {
    [
        GovernedExpectedResult {
            result_id: "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            domain: "fastq",
            stage_id: id_catalog::FASTQ_SCREEN,
            tool_id: id_catalog::TOOL_KRAKEN2,
            corpus_id: "corpus-02-edna-mini",
            asset_profile_id: "database_artifact_id+taxonomy_database_root",
            expected_output: "classification_report_json",
            expected_metric: "classification_report_json",
            report_section: "contamination_screening",
        },
        GovernedExpectedResult {
            result_id: "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
            domain: "bam",
            stage_id: id_catalog::BAM_KINSHIP,
            tool_id: id_catalog::TOOL_KING,
            corpus_id: "corpus-01-kinship-mini",
            asset_profile_id: "reference_fasta+reference_panel",
            expected_output: "kinship_report",
            expected_metric: "kinship_report",
            report_section: "sample_identity",
        },
        GovernedExpectedResult {
            result_id: "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
            domain: "vcf",
            stage_id: id_catalog::VCF_CALL,
            tool_id: id_catalog::TOOL_BCFTOOLS,
            corpus_id: "vcf_production_regression",
            asset_profile_id: "bam_bundle",
            expected_output: "called_vcf",
            expected_metric: "variant_count",
            report_section: "variant_calling",
        },
    ]
}

fn require_expected_row(
    rows: &[AllDomainExpectedBenchmarkResultRow],
    expected: &GovernedExpectedResult,
) -> Result<(), BenchError> {
    let result_id = expected.result_id;
    let row = rows.iter().find(|row| row.result_id == result_id).ok_or_else(|| {
        BenchError::InvalidObservation {
            reason: format!("all-domain expected-result table is missing `{result_id}`"),
        }
    })?;
    if row.domain != expected.domain
        || row.stage_id != expected.stage_id
        || row.tool_id != expected.tool_id
        || row.corpus_id != expected.corpus_id
        || row.asset_profile_id != expected.asset_profile_id
        || !row.expected_outputs.iter().any(|value| value == expected.expected_output)
        || !row.expected_metrics.iter().any(|value| value == expected.expected_metric)
        || row.report_section != expected.report_section
    {
        return Err(BenchError::InvalidObservation { reason: format!("all-domain expected-result row `{result_id}` drifted from the governed result contract") });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn evidence() -> Result<Vec<AllDomainExpectedBenchmarkResultRow>, serde_json::Error> {
        serde_json::from_str(
            r#"[
  {
    "result_id": "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
    "domain": "fastq",
    "stage_id": "fastq.screen_taxonomy",
    "tool_id": "kraken2",
    "corpus_id": "corpus-02-edna-mini",
    "asset_profile_id": "database_artifact_id+taxonomy_database_root",
    "report_section": "contamination_screening",
    "expected_outputs": [
      "classification_report_json"
    ],
    "expected_metrics": [
      "classification_report_json"
    ]
  },
  {
    "result_id": "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
    "domain": "bam",
    "stage_id": "bam.kinship",
    "tool_id": "king",
    "corpus_id": "corpus-01-kinship-mini",
    "asset_profile_id": "reference_fasta+reference_panel",
    "report_section": "sample_identity",
    "expected_outputs": [
      "kinship_report"
    ],
    "expected_metrics": [
      "kinship_report"
    ]
  },
  {
    "result_id": "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
    "domain": "vcf",
    "stage_id": "vcf.call",
    "tool_id": "bcftools",
    "corpus_id": "vcf_production_regression",
    "asset_profile_id": "bam_bundle",
    "report_section": "variant_calling",
    "expected_outputs": [
      "called_vcf"
    ],
    "expected_metrics": [
      "variant_count"
    ]
  }
]"#,
        )
    }

    #[test]
    fn expected_result_admission_preserves_governed_rows() -> anyhow::Result<()> {
        let rows = evidence()?;
        let before = rows.clone();
        validate_all_domain_expected_result_rows(&rows, 2, 1)?;
        assert_eq!(rows, before);
        Ok(())
    }

    #[test]
    fn expected_result_admission_rejects_missing_duplicate_and_overflowing_coverage(
    ) -> anyhow::Result<()> {
        let rows = evidence()?;
        assert!(validate_all_domain_expected_result_rows(&rows[1..], 2, 1).is_err());
        assert!(validate_all_domain_expected_result_rows(&rows, usize::MAX, 1).is_err());
        let mut duplicate = rows.clone();
        duplicate[1].result_id = rows[0].result_id.clone();
        assert!(validate_all_domain_expected_result_rows(&duplicate, 2, 1).is_err());
        Ok(())
    }

    #[test]
    fn expected_result_admission_rejects_empty_columns_outputs_and_metrics() -> anyhow::Result<()> {
        for field in 0..8 {
            let mut rows = evidence()?;
            let row = &mut rows[0];
            match field {
                0 => row.domain.clear(),
                1 => row.stage_id.clear(),
                2 => row.tool_id.clear(),
                3 => row.corpus_id.clear(),
                4 => row.asset_profile_id.clear(),
                5 => row.report_section.clear(),
                6 => row.expected_outputs.clear(),
                _ => row.expected_metrics.clear(),
            }
            assert!(validate_all_domain_expected_result_rows(&rows, 2, 1).is_err());
        }
        Ok(())
    }

    #[test]
    fn expected_result_admission_rejects_changed_governed_result_contracts() -> anyhow::Result<()> {
        for field in 0..9 {
            let mut rows = evidence()?;
            let row = &mut rows[0];
            match field {
                0 => row.result_id = "wrong".to_string(),
                1 => row.domain = "wrong".to_string(),
                2 => row.stage_id = "wrong".to_string(),
                3 => row.tool_id = "wrong".to_string(),
                4 => row.corpus_id = "wrong".to_string(),
                5 => row.asset_profile_id = "wrong".to_string(),
                6 => row.expected_outputs = vec!["wrong".to_string()],
                7 => row.expected_metrics = vec!["wrong".to_string()],
                _ => row.report_section = "wrong".to_string(),
            }
            assert!(validate_all_domain_expected_result_rows(&rows, 2, 1).is_err());
        }
        Ok(())
    }
}
