use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    DuplicateClassEntryV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
    REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
};
use crate::params::remove_duplicates::{DedupMode, RemoveDuplicatesEffectiveParams};
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

/// Remove exact duplicate reads with a deterministic native implementation suitable for governed
/// local smoke execution.
///
/// # Errors
/// Returns an error when paired inputs are incoherent, unsupported local-smoke options are
/// requested, or governed output wiring is incomplete.
#[allow(clippy::too_many_arguments)]
pub fn remove_duplicates(
    r1: &Path,
    r2: Option<&Path>,
    params: &RemoveDuplicatesEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    duplicate_classes_tsv: &Path,
    duplicate_provenance_json: &Path,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<RemoveDuplicatesReportV1> {
    if params.keep_order != true {
        return Err(anyhow!(
            "fastq.remove_duplicates local smoke currently requires keep_order=true"
        ));
    }
    if params.dedup_mode != DedupMode::Exact {
        return Err(anyhow!(
            "fastq.remove_duplicates local smoke currently supports only dedup_mode=exact"
        ));
    }

    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.remove_duplicates refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        if output_r2.is_none() {
            return Err(anyhow!(
                "fastq.remove_duplicates requires output_r2 when paired input is provided"
            ));
        }
        records
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let mut seen = BTreeSet::<String>::new();
    let mut dedup_left = Vec::<FastqRecord>::new();
    let mut dedup_right = Vec::<FastqRecord>::new();

    for index in 0..left.len() {
        let signature = if paired {
            format!(
                "{}|{}",
                left[index].sequence.to_ascii_uppercase(),
                right[index].sequence.to_ascii_uppercase()
            )
        } else {
            left[index].sequence.to_ascii_uppercase()
        };
        if seen.insert(signature) {
            dedup_left.push(left[index].clone());
            if paired {
                dedup_right.push(right[index].clone());
            }
        }
    }

    write_fastq_records(output_r1, &dedup_left)?;
    if let (Some(output_r2), true) = (output_r2, paired) {
        write_fastq_records(output_r2, &dedup_right)?;
    }

    let reads_in_r1 = left.len() as u64;
    let reads_out_r1 = dedup_left.len() as u64;
    let reads_in_r2 = paired.then_some(right.len() as u64);
    let reads_out_r2 = paired.then_some(dedup_right.len() as u64);
    let reads_in = reads_in_r1 + reads_in_r2.unwrap_or(0);
    let reads_out = reads_out_r1 + reads_out_r2.unwrap_or(0);
    let duplicates_removed = reads_in.saturating_sub(reads_out);
    let dedup_rate = if reads_in == 0 { 0.0 } else { duplicates_removed as f64 / reads_in as f64 };
    let duplicate_class = DuplicateClassEntryV1 {
        class: "exact_duplicate".to_string(),
        reads_removed: duplicates_removed,
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
    };

    if let Some(parent) = duplicate_classes_tsv.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        duplicate_classes_tsv,
        format!(
            "class\treads_removed\tpaired_mode\n{}\t{}\t{}\n",
            duplicate_class.class,
            duplicate_class.reads_removed,
            if paired { "paired_end" } else { "single_end" }
        ),
    )?;

    let provenance = RemoveDuplicatesProvenanceV1 {
        schema_version: REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION.to_string(),
        stage_id: "fastq.remove_duplicates".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        dedup_mode: params.dedup_mode.clone(),
        keep_order: params.keep_order,
        duplicates_removed,
        dedup_rate,
        backend_log: raw_backend_report.map(|path| path.display().to_string()),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report
            .map(|_| "bijux_remove_duplicates_trace".to_string()),
    };
    bijux_dna_infra::atomic_write_json(duplicate_provenance_json, &provenance)?;

    if let Some(raw_backend_report) = raw_backend_report {
        std::fs::write(
            raw_backend_report,
            format!(
                "reads_in\t{}\nreads_out\t{}\nduplicates_removed\t{}\ndedup_mode\texact\nkeep_order\ttrue\n",
                reads_in, reads_out, duplicates_removed
            ),
        )?;
    }

    let report = RemoveDuplicatesReportV1 {
        schema_version: REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.remove_duplicates".to_string(),
        stage_id: "fastq.remove_duplicates".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        dedup_mode: params.dedup_mode.clone(),
        keep_order: params.keep_order,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        reads_in,
        reads_out,
        reads_in_r2,
        reads_out_r2,
        pairs_in: paired.then_some(reads_in_r1),
        pairs_out: paired.then_some(reads_out_r1),
        pair_count_match: paired.then_some(reads_out_r1 == reads_out_r2.unwrap_or(0)),
        duplicates_removed,
        dedup_rate,
        duplicate_classes_tsv: Some(duplicate_classes_tsv.display().to_string()),
        duplicate_provenance_json: Some(duplicate_provenance_json.display().to_string()),
        duplicate_classes: vec![duplicate_class],
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report
            .map(|_| "bijux_remove_duplicates_trace".to_string()),
        runtime_s: None,
        memory_mb: None,
    };
    bijux_dna_infra::atomic_write_json(report_json, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::remove_duplicates;
    use crate::params::remove_duplicates::{
        DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;
    use crate::stages::contract::runtime::fastq_io::read_fastq_records;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, sequence, quality) in records {
            payload.push_str(&format!("@{header}\n{sequence}\n+\n{quality}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn remove_duplicates_preserves_first_unique_records() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-remove-duplicates")?;
        let r1 = temp.path().join("reads.fastq");
        write_fastq(
            &r1,
            &[
                ("dup_a_1", "ACGTACGT", "IIIIIIII"),
                ("dup_b_1", "GGGGTTTT", "JJJJJJJJ"),
                ("dup_a_2", "ACGTACGT", "HHHHHHHH"),
                ("unique_c", "TTCCAAGG", "FFFFFFFF"),
            ],
        )?;

        let report = remove_duplicates(
            &r1,
            None,
            &RemoveDuplicatesEffectiveParams {
                schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
                paired_mode: PairedMode::SingleEnd,
                threads: 1,
                dedup_mode: DedupMode::Exact,
                keep_order: true,
            },
            &temp.path().join("dedup.fastq.gz"),
            None,
            &temp.path().join("duplicate_classes.tsv"),
            &temp.path().join("duplicate_provenance.json"),
            &temp.path().join("report.json"),
            Some(&temp.path().join("raw_backend_report.txt")),
        )?;

        assert_eq!(report.tool_id, "bijux");
        assert_eq!(report.reads_in, 4);
        assert_eq!(report.reads_out, 3);
        assert_eq!(report.duplicates_removed, 1);

        let dedup = read_fastq_records(&temp.path().join("dedup.fastq.gz"))?;
        assert_eq!(dedup.len(), 3);
        assert_eq!(dedup[0].header, "@dup_a_1");
        assert_eq!(dedup[1].header, "@dup_b_1");
        assert_eq!(dedup[2].header, "@unique_c");
        Ok(())
    }
}
