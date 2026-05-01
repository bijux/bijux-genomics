use std::path::Path;

use anyhow::Result;

use crate::artifacts::{DeinterleaveReadsReportV1, DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION};

use super::fastq_io::{parse_header_pairing, read_fastq_records, write_fastq_records, FastqRecord};

/// Split interleaved FASTQ reads into R1/R2 with explicit rejected-record tracking.
///
/// # Errors
/// Returns an error when input cannot be parsed or output cannot be written.
pub fn deinterleave_reads(
    input_interleaved: &Path,
    output_r1: &Path,
    output_r2: &Path,
    rejected_path: &Path,
) -> Result<DeinterleaveReadsReportV1> {
    let records = read_fastq_records(input_interleaved)?;

    let mut out_r1 = Vec::<FastqRecord>::new();
    let mut out_r2 = Vec::<FastqRecord>::new();
    let mut rejected = Vec::<FastqRecord>::new();

    let mut idx = 0_usize;
    while idx < records.len() {
        let left = &records[idx];
        let Some(right) = records.get(idx + 1) else {
            rejected.push(left.clone());
            break;
        };

        let (left_base, left_mate) = parse_header_pairing(left.header.trim_start_matches('@'));
        let (right_base, right_mate) = parse_header_pairing(right.header.trim_start_matches('@'));

        let base_matches = left_base == right_base;
        let mate_order_valid = !matches!(left_mate, Some(2)) && !matches!(right_mate, Some(1));
        let explicit_pair_valid = match (left_mate, right_mate) {
            (Some(l), Some(r)) => l == 1 && r == 2,
            _ => true,
        };

        if base_matches && mate_order_valid && explicit_pair_valid {
            out_r1.push(left.clone());
            out_r2.push(right.clone());
        } else {
            rejected.push(left.clone());
            rejected.push(right.clone());
        }

        idx += 2;
    }

    write_fastq_records(output_r1, &out_r1)?;
    write_fastq_records(output_r2, &out_r2)?;
    write_fastq_records(rejected_path, &rejected)?;

    Ok(DeinterleaveReadsReportV1 {
        schema_version: DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.deinterleave_reads".to_string(),
        stage_id: "fastq.deinterleave_reads".to_string(),
        tool_id: "bijux".to_string(),
        reads_in: records.len() as u64,
        output_pairs: out_r1.len() as u64,
        output_r1_reads: out_r1.len() as u64,
        output_r2_reads: out_r2.len() as u64,
        rejected_records: rejected.len() as u64,
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.display().to_string(),
        rejected_path: rejected_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::deinterleave_reads;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn deinterleave_reads_splits_and_rejects_invalid_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-deinterleave-reads")?;
        let input = temp.path().join("interleaved.fastq");

        write_fastq(
            &input,
            &[
                ("A/1", "AAAA", "!!!!"),
                ("A/2", "TTTT", "####"),
                ("B/2", "CCCC", "!!!!"),
                ("B/1", "GGGG", "####"),
                ("C/1", "ACAC", "!!!!"),
            ],
        )?;

        let report = deinterleave_reads(
            &input,
            &temp.path().join("r1.fastq"),
            &temp.path().join("r2.fastq"),
            &temp.path().join("rejected.fastq"),
        )?;

        assert_eq!(report.output_pairs, 1);
        assert_eq!(report.rejected_records, 3);
        Ok(())
    }
}
