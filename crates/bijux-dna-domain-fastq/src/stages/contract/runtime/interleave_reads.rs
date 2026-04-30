use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{InterleaveReadsReportV1, INTERLEAVE_READS_REPORT_SCHEMA_VERSION};

use super::fastq_io::{parse_header_pairing, read_fastq_records, write_fastq_records, FastqRecord};

/// Interleave paired FASTQ reads with deterministic order and pairing validation.
///
/// # Errors
/// Returns an error when inputs are incoherent or output cannot be written.
pub fn interleave_reads(
    r1: &Path,
    r2: &Path,
    output_interleaved: &Path,
) -> Result<InterleaveReadsReportV1> {
    let left = read_fastq_records(r1)?;
    let right = read_fastq_records(r2)?;

    if left.len() != right.len() {
        return Err(anyhow!(
            "fastq.interleave_reads refused incoherent input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }

    let mut output = Vec::<FastqRecord>::with_capacity(left.len() * 2);
    for (idx, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        let (left_base, _) = parse_header_pairing(l.header.trim_start_matches('@'));
        let (right_base, _) = parse_header_pairing(r.header.trim_start_matches('@'));
        if left_base != right_base {
            return Err(anyhow!(
                "fastq.interleave_reads refused incoherent pairing at pair {}: `{}` vs `{}`",
                idx + 1,
                left_base,
                right_base
            ));
        }
        output.push(l.clone());
        output.push(r.clone());
    }

    write_fastq_records(output_interleaved, &output)?;

    Ok(InterleaveReadsReportV1 {
        schema_version: INTERLEAVE_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.interleave_reads".to_string(),
        stage_id: "fastq.interleave_reads".to_string(),
        tool_id: "bijux".to_string(),
        reads_in_r1: left.len() as u64,
        reads_in_r2: right.len() as u64,
        interleaved_pairs: left.len() as u64,
        output_reads: output.len() as u64,
        pairing_validated: true,
        output_interleaved: output_interleaved.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::interleave_reads;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn interleave_reads_writes_deterministic_pair_order() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-interleave-reads")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        let out = temp.path().join("interleaved.fastq");

        write_fastq(&r1, &[("A/1", "AAAA", "!!!!"), ("B/1", "CCCC", "####")])?;
        write_fastq(&r2, &[("A/2", "TTTT", "!!!!"), ("B/2", "GGGG", "####")])?;

        let report = interleave_reads(&r1, &r2, &out)?;
        let records = super::super::fastq_io::read_fastq_records(&out)?;

        assert_eq!(records.len(), 4);
        assert_eq!(records[0].header, "@A/1");
        assert_eq!(records[1].header, "@A/2");
        assert_eq!(report.interleaved_pairs, 2);
        Ok(())
    }

    #[test]
    fn interleave_reads_refuses_mismatched_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-interleave-reads")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        let out = temp.path().join("interleaved.fastq");

        write_fastq(&r1, &[("A/1", "AAAA", "!!!!")])?;
        write_fastq(&r2, &[("B/2", "TTTT", "!!!!")])?;

        let result = interleave_reads(&r1, &r2, &out);
        assert!(result.is_err());
        Ok(())
    }
}
