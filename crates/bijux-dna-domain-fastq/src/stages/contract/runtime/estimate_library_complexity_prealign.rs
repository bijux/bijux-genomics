use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    EstimateLibraryComplexityPrealignReportV1,
    ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION,
    PREALIGN_COMPLEXITY_INSUFFICIENT_DATA_REASON,
};
use crate::params::PairedMode;

use super::fastq_io::read_fastq_records;

fn leading_kmer(sequence: &str, k: usize) -> String {
    sequence.chars().take(k).collect::<String>().to_ascii_uppercase()
}

/// Estimate prealignment library complexity from read-level k-mer redundancy.
///
/// # Errors
/// Returns an error when paired inputs are incoherent.
pub fn estimate_library_complexity_prealign(
    r1: &Path,
    r2: Option<&Path>,
    kmer_size: Option<u32>,
) -> Result<EstimateLibraryComplexityPrealignReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.estimate_library_complexity_prealign refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        records
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let requested_k = kmer_size.unwrap_or(31).max(1) as usize;
    let observed_min_len = left
        .iter()
        .map(|record| record.sequence.len())
        .chain(right.iter().map(|record| record.sequence.len()))
        .min()
        .unwrap_or(requested_k);
    let effective_k = usize::min(requested_k, observed_min_len.max(1));

    let mut signatures = BTreeSet::<String>::new();
    let unit_count = if paired {
        for idx in 0..left.len() {
            let sig = format!(
                "{}|{}",
                leading_kmer(&left[idx].sequence, effective_k),
                leading_kmer(&right[idx].sequence, effective_k)
            );
            signatures.insert(sig);
        }
        left.len() as u64
    } else {
        for record in &left {
            signatures.insert(leading_kmer(&record.sequence, effective_k));
        }
        left.len() as u64
    };

    let unique_units = signatures.len() as u64;
    let estimated_unique_fraction =
        if unit_count == 0 { 0.0 } else { unique_units as f64 / unit_count as f64 };
    let estimated_duplicate_fraction =
        if unit_count == 0 { 0.0 } else { 1.0 - estimated_unique_fraction };

    Ok(EstimateLibraryComplexityPrealignReportV1 {
        schema_version: ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.estimate_library_complexity_prealign".to_string(),
        stage_id: "fastq.estimate_library_complexity_prealign".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        complexity_policy: "prealign_kmer".to_string(),
        estimate_method: "kmer_redundancy".to_string(),
        modifies_reads: false,
        advisory_only: true,
        reads_in: if paired { (left.len() as u64).saturating_mul(2) } else { left.len() as u64 },
        estimated_unique_fraction,
        estimated_duplicate_fraction,
        insufficient_data_reason: if unit_count == 0 {
            Some(PREALIGN_COMPLEXITY_INSUFFICIENT_DATA_REASON.to_string())
        } else {
            None
        },
        kmer_size: Some(effective_k as u32),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        estimate_library_complexity_prealign, PREALIGN_COMPLEXITY_INSUFFICIENT_DATA_REASON,
    };

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn estimate_library_complexity_detects_redundancy() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-estimate-complexity")?;
        let r1 = temp.path().join("r1.fastq");

        write_fastq(
            &r1,
            &[
                ("A", "ACGTACGT", "!!!!!!!!"),
                ("B", "ACGTACGT", "!!!!!!!!"),
                ("C", "TTTTCCCC", "!!!!!!!!"),
            ],
        )?;

        let report = estimate_library_complexity_prealign(&r1, None, Some(4))?;
        assert_eq!(report.reads_in, 3);
        assert!(report.estimated_duplicate_fraction > 0.0);
        assert_eq!(report.insufficient_data_reason, None);
        Ok(())
    }

    #[test]
    fn estimate_library_complexity_reports_insufficient_data_for_empty_input() -> anyhow::Result<()>
    {
        let temp = bijux_dna_infra::temp_dir("bijux-estimate-complexity-empty")?;
        let r1 = temp.path().join("r1.fastq");
        std::fs::write(&r1, "")?;

        let report = estimate_library_complexity_prealign(&r1, None, Some(4))?;
        assert_eq!(report.reads_in, 0);
        assert_eq!(report.estimated_unique_fraction, 0.0);
        assert_eq!(report.estimated_duplicate_fraction, 0.0);
        assert_eq!(
            report.insufficient_data_reason.as_deref(),
            Some(PREALIGN_COMPLEXITY_INSUFFICIENT_DATA_REASON)
        );
        Ok(())
    }
}
