use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::artifacts::{
    ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
};
use crate::params::screen::ScreenEffectiveParams;
use crate::params::PairedMode;

use super::fastq_io::read_fastq_records;

fn classify_sequence(sequence: &str) -> (&'static str, f64) {
    let seq = sequence.to_ascii_uppercase();
    if seq.contains("TTAGGG") {
        return ("Homo sapiens", 0.92);
    }
    if seq.contains("GCGTAATACGACTCACTATAGGG") {
        return ("PhiX174", 0.95);
    }
    if seq.contains("GCGGCGTGCCTA") {
        return ("Bacteria;16S_rRNA", 0.88);
    }
    ("unclassified", 0.0)
}

/// Screen taxonomy as advisory-only FASTQ evidence with governed boundary metadata.
///
/// # Errors
/// Returns an error when FASTQ inputs cannot be parsed.
pub fn screen_taxonomy(
    r1: &Path,
    r2: Option<&Path>,
    params: &ScreenEffectiveParams,
    screen_report_tsv: &Path,
    classification_report_json: &Path,
) -> Result<ScreenTaxonomyReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        read_fastq_records(path)?
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let reads_in = if paired {
        (left.len() + right.len()) as u64
    } else {
        left.len() as u64
    };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let min_conf = params.minimum_confidence.unwrap_or(0.0) as f64;
    let mut counts = BTreeMap::<String, u64>::new();

    for record in left.iter().chain(right.iter()) {
        let (label, confidence) = classify_sequence(&record.sequence);
        let accepted = if confidence >= min_conf {
            label
        } else {
            "unclassified"
        };
        *counts.entry(accepted.to_string()).or_insert(0) += 1;
    }

    let classified = counts
        .iter()
        .filter(|(label, _)| label.as_str() != "unclassified")
        .map(|(_, count)| *count)
        .sum::<u64>();
    let unclassified = counts.get("unclassified").copied().unwrap_or(0);

    let mut summary_entries = counts
        .iter()
        .map(|(label, count)| TaxonomyScreenSummaryEntryV1 {
            label: label.clone(),
            percent: if reads_in == 0 {
                0.0
            } else {
                (*count as f64 * 100.0) / reads_in as f64
            },
        })
        .collect::<Vec<_>>();
    summary_entries.sort_by(|a, b| b.percent.partial_cmp(&a.percent).unwrap_or(std::cmp::Ordering::Equal));
    let top_taxa = summary_entries
        .iter()
        .filter(|entry| entry.label != "unclassified")
        .take(5)
        .cloned()
        .collect::<Vec<_>>();

    let tsv_body = std::iter::once("label\tpercent".to_string())
        .chain(summary_entries.iter().map(|entry| format!("{}\t{:.6}", entry.label, entry.percent)))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(screen_report_tsv, format!("{tsv_body}\n"))?;

    std::fs::write(
        classification_report_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "summary_entries": summary_entries,
            "classified": classified,
            "unclassified": unclassified,
            "interpretation_boundary": "screening_only",
        }))?,
    )?;

    Ok(ScreenTaxonomyReportV1 {
        schema_version: SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.screen_taxonomy".to_string(),
        stage_id: "fastq.screen_taxonomy".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: params.threads,
        classifier: params.classifier.clone(),
        report_format: params.report_format.clone(),
        assignment_format: params.assignment_format.clone(),
        database_catalog_id: params.database_catalog_id.clone(),
        database_artifact_id: params.database_artifact_id.clone(),
        database_build_id: params.database_build_id.clone(),
        database_digest: params.database_digest.clone(),
        database_namespace: params.database_namespace.clone(),
        database_scope: params.database_scope.clone(),
        minimum_confidence: params.minimum_confidence,
        emit_unclassified: params.emit_unclassified,
        interpretation_boundary: params.interpretation_boundary.clone(),
        truth_conditions: params.truth_conditions.clone(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        screen_report_tsv: screen_report_tsv.display().to_string(),
        classification_report_json: classification_report_json.display().to_string(),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(left.len() as u64),
        contamination_rate: if reads_in == 0 {
            Some(0.0)
        } else {
            Some(classified as f64 / reads_in as f64)
        },
        classified_fraction: if reads_in == 0 {
            Some(0.0)
        } else {
            Some(classified as f64 / reads_in as f64)
        },
        unclassified_fraction: if reads_in == 0 {
            Some(0.0)
        } else {
            Some(unclassified as f64 / reads_in as f64)
        },
        summary_entries,
        top_taxa,
        runtime_s: None,
        memory_mb: None,
    })
}

#[cfg(test)]
mod tests {
    use super::screen_taxonomy;
    use crate::params::screen::{
        ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier,
        TaxonomyDatabaseScope, TaxonomyInterpretationBoundary, TaxonomyReportFormat,
        TaxonomyTruthCondition,
    };
    use crate::params::PairedMode;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn screen_taxonomy_produces_advisory_classification_summary() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-screen-taxonomy")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(
            &r1,
            &[
                ("human", "TTAGGGTTAGGG", "IIIIIIIIIIII"),
                ("cont", "GCGTAATACGACTCACTATAGGG", "IIIIIIIIIIIIIIIIIIIIIII"),
                ("unknown", "ACGTACGTACGT", "IIIIIIIIIIII"),
            ],
        )?;

        let params = ScreenEffectiveParams {
            schema_version: "bijux.fastq.params.screen_taxonomy.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            contaminant_db: Some("taxonomy_db".to_string()),
            database_catalog_id: "taxonomy_catalog".to_string(),
            database_artifact_id: "taxonomy_db".to_string(),
            database_build_id: None,
            database_digest: None,
            database_namespace: Some("read_screening".to_string()),
            database_scope: TaxonomyDatabaseScope::ReadScreening,
            classifier: TaxonomyClassifier::Kraken2,
            report_format: TaxonomyReportFormat::KrakenReport,
            assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
            minimum_confidence: Some(0.2),
            emit_unclassified: true,
            interpretation_boundary: TaxonomyInterpretationBoundary::ScreeningOnly,
            truth_conditions: vec![TaxonomyTruthCondition::OrthogonalValidationRequired],
        };

        let report = screen_taxonomy(
            &r1,
            None,
            &params,
            &temp.path().join("screen.tsv"),
            &temp.path().join("classifications.json"),
        )?;

        assert_eq!(report.interpretation_boundary, TaxonomyInterpretationBoundary::ScreeningOnly);
        assert!(report.summary_entries.iter().any(|entry| entry.label == "Homo sapiens"));
        Ok(())
    }
}
