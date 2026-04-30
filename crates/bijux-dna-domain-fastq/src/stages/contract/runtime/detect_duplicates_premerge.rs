use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    DetectDuplicatesPremergeReportV1, DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION,
};
use crate::params::PairedMode;

use super::fastq_io::read_fastq_records;

/// Detect pre-merge duplicate signal from read-sequence signatures.
///
/// # Errors
/// Returns an error when paired inputs are incoherent.
pub fn detect_duplicates_premerge(
    r1: &Path,
    r2: Option<&Path>,
) -> Result<DetectDuplicatesPremergeReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.detect_duplicates_premerge refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        records
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let (reads_in, duplicate_signal_reads, compared_read_pairs) = if paired {
        let mut signatures = BTreeSet::<String>::new();
        for idx in 0..left.len() {
            let signature = format!(
                "{}|{}",
                left[idx].sequence.to_ascii_uppercase(),
                right[idx].sequence.to_ascii_uppercase()
            );
            signatures.insert(signature);
        }
        let pair_count = left.len() as u64;
        let duplicate_pairs = pair_count.saturating_sub(signatures.len() as u64);
        let duplicate_reads = duplicate_pairs.saturating_mul(2);
        (pair_count.saturating_mul(2), duplicate_reads, Some(pair_count))
    } else {
        let mut signatures = BTreeSet::<String>::new();
        for record in &left {
            signatures.insert(record.sequence.to_ascii_uppercase());
        }
        let total = left.len() as u64;
        let duplicate_reads = total.saturating_sub(signatures.len() as u64);
        (total, duplicate_reads, None)
    };

    let duplicate_signal_fraction = if reads_in == 0 {
        0.0
    } else {
        duplicate_signal_reads as f64 / reads_in as f64
    };

    Ok(DetectDuplicatesPremergeReportV1 {
        schema_version: DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.detect_duplicates_premerge".to_string(),
        stage_id: "fastq.detect_duplicates_premerge".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        duplicate_detection_policy: "report_only".to_string(),
        measurement_scope: "premerge_sequence_signature".to_string(),
        modifies_reads: false,
        advisory_only: true,
        reads_in,
        duplicate_signal_reads,
        duplicate_signal_fraction,
        compared_read_pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::detect_duplicates_premerge;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn detect_duplicates_premerge_reports_duplicate_signal_for_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-dedup-premerge")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");

        write_fastq(
            &r1,
            &[
                ("A/1", "AAAA", "!!!!"),
                ("B/1", "CCCC", "####"),
                ("C/1", "AAAA", "!!!!"),
            ],
        )?;
        write_fastq(
            &r2,
            &[
                ("A/2", "TTTT", "!!!!"),
                ("B/2", "GGGG", "####"),
                ("C/2", "TTTT", "!!!!"),
            ],
        )?;

        let report = detect_duplicates_premerge(&r1, Some(&r2))?;
        assert_eq!(report.reads_in, 6);
        assert_eq!(report.duplicate_signal_reads, 2);
        assert_eq!(report.compared_read_pairs, Some(3));
        Ok(())
    }
}
