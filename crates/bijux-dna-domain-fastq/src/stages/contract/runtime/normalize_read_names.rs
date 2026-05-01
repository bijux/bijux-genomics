use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{NormalizeReadNamesReportV1, NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION};

use super::fastq_io::{read_fastq_records, write_fastq_records};

/// Normalize FASTQ read names while preserving reversible provenance.
///
/// # Errors
/// Returns an error when inputs are incoherent or output cannot be written.
pub fn normalize_read_names(
    r1: &Path,
    r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
) -> Result<NormalizeReadNamesReportV1> {
    let mut left = read_fastq_records(r1)?;
    let reads_in = if let Some(r2) = r2 {
        let mut right = read_fastq_records(r2)?;
        if left.len() != right.len() {
            return Err(anyhow!(
                "fastq.normalize_read_names refused incoherent input: R1 count {} != R2 count {}",
                left.len(),
                right.len()
            ));
        }
        for (idx, (l, r)) in left.iter_mut().zip(right.iter_mut()).enumerate() {
            let original_left = l.header.trim_start_matches('@').to_string();
            let original_right = r.header.trim_start_matches('@').to_string();
            l.header = format!("@bijux:{}/1 orig:{}", idx + 1, original_left);
            r.header = format!("@bijux:{}/2 orig:{}", idx + 1, original_right);
        }
        write_fastq_records(output_r1, &left)?;
        let out_r2 = output_r2.ok_or_else(|| {
            anyhow!("fastq.normalize_read_names requires output_r2 for paired-end input")
        })?;
        write_fastq_records(out_r2, &right)?;
        (left.len() + right.len()) as u64
    } else {
        for (idx, record) in left.iter_mut().enumerate() {
            let original = record.header.trim_start_matches('@').to_string();
            record.header = format!("@bijux:{} orig:{}", idx + 1, original);
        }
        write_fastq_records(output_r1, &left)?;
        left.len() as u64
    };

    let reads_out = reads_in;
    Ok(NormalizeReadNamesReportV1 {
        schema_version: NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.normalize_read_names".to_string(),
        stage_id: "fastq.normalize_read_names".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if r2.is_some() { "paired_end" } else { "single_end" }.to_string(),
        reads_in,
        reads_out,
        reversible_provenance_embedded: true,
        mate_identity_preserved: r2.is_some(),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_read_names;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn normalize_read_names_embeds_original_ids_for_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-normalize-read-names")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        let out_r1 = temp.path().join("norm_r1.fastq");
        let out_r2 = temp.path().join("norm_r2.fastq");

        write_fastq(&r1, &[("A/1", "ACGT", "!!!!"), ("B/1", "TTTT", "####")])?;
        write_fastq(&r2, &[("A/2", "TGCA", "!!!!"), ("B/2", "AAAA", "####")])?;

        let report = normalize_read_names(&r1, Some(&r2), &out_r1, Some(&out_r2))?;
        let out = std::fs::read_to_string(&out_r1)?;
        assert!(out.contains("@bijux:1/1 orig:A/1"));
        assert_eq!(report.reads_in, 4);
        Ok(())
    }

    #[test]
    fn normalize_read_names_refuses_unbalanced_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-normalize-read-names")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        let out_r1 = temp.path().join("norm_r1.fastq");
        let out_r2 = temp.path().join("norm_r2.fastq");

        write_fastq(&r1, &[("A/1", "ACGT", "!!!!"), ("B/1", "TTTT", "####")])?;
        write_fastq(&r2, &[("A/2", "TGCA", "!!!!")])?;

        let result = normalize_read_names(&r1, Some(&r2), &out_r1, Some(&out_r2));
        assert!(result.is_err());
        Ok(())
    }
}
