use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::artifacts::{SubsampleReadsReportV1, SUBSAMPLE_READS_REPORT_SCHEMA_VERSION};

use super::fastq_io::{parse_header_pairing, read_fastq_records, write_fastq_records};

pub enum SubsampleTarget {
    Count(u64),
    Fraction(f64),
}

/// Subsample reads deterministically by count or fraction while preserving pairing.
///
/// # Errors
/// Returns an error when inputs are incoherent or parameters are invalid.
pub fn subsample_reads(
    r1: &Path,
    r2: Option<&Path>,
    target: SubsampleTarget,
    seed: u64,
    output_r1: &Path,
    output_r2: Option<&Path>,
) -> Result<SubsampleReadsReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.subsample_reads refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        records
    } else {
        Vec::new()
    };

    let paired_mode = r2.is_some();
    if paired_mode && output_r2.is_none() {
        return Err(anyhow!(
            "fastq.subsample_reads requires output_r2 for paired-end input"
        ));
    }

    let total = left.len();
    let target_count = match target {
        SubsampleTarget::Count(count) => usize::min(count as usize, total),
        SubsampleTarget::Fraction(fraction) => {
            if !(0.0..=1.0).contains(&fraction) {
                return Err(anyhow!(
                    "fastq.subsample_reads requires fraction in range [0.0, 1.0]"
                ));
            }
            ((total as f64) * fraction).round() as usize
        }
    };

    let mut ranked = Vec::<(u64, usize)>::with_capacity(total);
    for (idx, record) in left.iter().enumerate() {
        let (base, _) = parse_header_pairing(record.header.trim_start_matches('@'));
        let mut hasher = Sha256::new();
        hasher.update(seed.to_le_bytes());
        hasher.update(base.as_bytes());
        let digest = hasher.finalize();
        let score = u64::from_le_bytes([
            digest[0], digest[1], digest[2], digest[3], digest[4], digest[5], digest[6], digest[7],
        ]);
        ranked.push((score, idx));
    }
    ranked.sort_by(|a, b| a.cmp(b));

    let selected = ranked.into_iter().take(target_count).map(|(_, idx)| idx).collect::<BTreeSet<_>>();

    let mut out_left = Vec::new();
    let mut out_right = Vec::new();
    for (idx, left_record) in left.iter().enumerate() {
        if selected.contains(&idx) {
            out_left.push(left_record.clone());
            if paired_mode {
                out_right.push(right[idx].clone());
            }
        }
    }

    write_fastq_records(output_r1, &out_left)?;
    if paired_mode {
        let out_r2 = output_r2.expect("validated above");
        write_fastq_records(out_r2, &out_right)?;
    }

    let (sampling_mode, target_count_report, target_fraction_report) = match target {
        SubsampleTarget::Count(count) => ("count".to_string(), Some(count), None),
        SubsampleTarget::Fraction(fraction) => ("fraction".to_string(), None, Some(fraction)),
    };

    Ok(SubsampleReadsReportV1 {
        schema_version: SUBSAMPLE_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.subsample_reads".to_string(),
        stage_id: "fastq.subsample_reads".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired_mode { "paired_end" } else { "single_end" }.to_string(),
        sampling_mode,
        seed,
        target_count: target_count_report,
        target_fraction: target_fraction_report,
        reads_in_r1: left.len() as u64,
        reads_in_r2: paired_mode.then_some(right.len() as u64),
        reads_out_r1: out_left.len() as u64,
        reads_out_r2: paired_mode.then_some(out_right.len() as u64),
        pairs_preserved: paired_mode,
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::{subsample_reads, SubsampleTarget};

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn subsample_reads_preserves_pairing_and_is_deterministic() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-subsample-reads")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");

        write_fastq(
            &r1,
            &[
                ("A/1", "AAAA", "!!!!"),
                ("B/1", "CCCC", "####"),
                ("C/1", "GGGG", "$$$$"),
                ("D/1", "TTTT", "++++"),
            ],
        )?;
        write_fastq(
            &r2,
            &[
                ("A/2", "TTTT", "!!!!"),
                ("B/2", "GGGG", "####"),
                ("C/2", "CCCC", "$$$$"),
                ("D/2", "AAAA", "++++"),
            ],
        )?;

        let report = subsample_reads(
            &r1,
            Some(&r2),
            SubsampleTarget::Count(2),
            42,
            &temp.path().join("out_r1.fastq"),
            Some(&temp.path().join("out_r2.fastq")),
        )?;

        assert_eq!(report.reads_out_r1, 2);
        assert_eq!(report.reads_out_r2, Some(2));

        let report2 = subsample_reads(
            &r1,
            Some(&r2),
            SubsampleTarget::Count(2),
            42,
            &temp.path().join("out2_r1.fastq"),
            Some(&temp.path().join("out2_r2.fastq")),
        )?;

        assert_eq!(report.reads_out_r1, report2.reads_out_r1);
        Ok(())
    }
}
