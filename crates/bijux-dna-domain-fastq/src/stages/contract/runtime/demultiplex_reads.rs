use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    DemultiplexReadsReportV1, DemultiplexSampleSummaryV1, DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION,
};

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

pub struct DemultiplexRule<'a> {
    pub sample_id: &'a str,
    pub barcode: &'a str,
}

/// Demultiplex FASTQ reads by barcode using a deterministic hamming-prefix policy.
///
/// # Errors
/// Returns an error when rules are invalid or outputs cannot be written.
pub fn demultiplex_reads(
    input: &Path,
    rules: &[DemultiplexRule<'_>],
    max_mismatches: u8,
    output_dir: &Path,
    undetermined_path: &Path,
) -> Result<DemultiplexReadsReportV1> {
    if rules.is_empty() {
        return Err(anyhow!("fastq.demultiplex_reads requires at least one demultiplex rule"));
    }

    let barcode_len = rules[0].barcode.len();
    if barcode_len == 0 {
        return Err(anyhow!("fastq.demultiplex_reads requires non-empty barcodes"));
    }

    let mut sample_ids = BTreeSet::new();
    let mut barcodes = BTreeSet::new();
    for rule in rules {
        if rule.barcode.len() != barcode_len {
            return Err(anyhow!(
                "fastq.demultiplex_reads requires all barcodes to have equal length"
            ));
        }
        if !sample_ids.insert(rule.sample_id.to_string()) {
            return Err(anyhow!(
                "fastq.demultiplex_reads found duplicate sample id `{}`",
                rule.sample_id
            ));
        }
        if !barcodes.insert(rule.barcode.to_ascii_uppercase()) {
            return Err(anyhow!(
                "fastq.demultiplex_reads found duplicate barcode `{}`",
                rule.barcode
            ));
        }
    }

    let reads = read_fastq_records(input)?;
    let mut assigned = BTreeMap::<String, Vec<FastqRecord>>::new();
    let mut undetermined = Vec::<FastqRecord>::new();

    for read in reads.iter().cloned() {
        let prefix =
            read.sequence.chars().take(barcode_len).collect::<String>().to_ascii_uppercase();
        if prefix.len() != barcode_len {
            undetermined.push(read);
            continue;
        }

        let mut best_sample: Option<&str> = None;
        let mut best_distance = usize::MAX;
        let mut tied = false;

        for rule in rules {
            let barcode = rule.barcode.to_ascii_uppercase();
            let distance = prefix.chars().zip(barcode.chars()).filter(|(a, b)| a != b).count();
            if distance > max_mismatches as usize {
                continue;
            }
            if distance < best_distance {
                best_distance = distance;
                best_sample = Some(rule.sample_id);
                tied = false;
            } else if distance == best_distance {
                tied = true;
            }
        }

        if let Some(sample_id) = best_sample {
            if tied {
                undetermined.push(read);
            } else {
                assigned.entry(sample_id.to_string()).or_default().push(read);
            }
        } else {
            undetermined.push(read);
        }
    }

    std::fs::create_dir_all(output_dir)?;

    let mut samples = Vec::<DemultiplexSampleSummaryV1>::new();
    let mut assigned_reads = 0_u64;
    for rule in rules {
        let sample_reads = assigned.remove(rule.sample_id).unwrap_or_default();
        let sample_path = output_dir.join(format!("{}.fastq.gz", rule.sample_id));
        write_fastq_records(&sample_path, &sample_reads)?;
        assigned_reads += sample_reads.len() as u64;
        samples.push(DemultiplexSampleSummaryV1 {
            sample_id: rule.sample_id.to_string(),
            barcode: rule.barcode.to_ascii_uppercase(),
            reads: sample_reads.len() as u64,
            output_path: sample_path.display().to_string(),
        });
    }

    write_fastq_records(undetermined_path, &undetermined)?;

    Ok(DemultiplexReadsReportV1 {
        schema_version: DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.demultiplex_reads".to_string(),
        stage_id: "fastq.demultiplex_reads".to_string(),
        tool_id: "bijux".to_string(),
        mismatch_policy: "hamming_prefix".to_string(),
        max_mismatches,
        sample_sheet_validated: true,
        reads_in: reads.len() as u64,
        assigned_reads,
        undetermined_reads: undetermined.len() as u64,
        undetermined_path: undetermined_path.display().to_string(),
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::{demultiplex_reads, DemultiplexRule};

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn demultiplex_reads_assigns_and_routes_undetermined_reads() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-demultiplex-reads")?;
        let input = temp.path().join("reads.fastq");

        write_fastq(
            &input,
            &[
                ("r1", "ACGTAAAA", "!!!!!!!!"),
                ("r2", "TGCATTTT", "!!!!!!!!"),
                ("r3", "ACGAAAAA", "!!!!!!!!"),
                ("r4", "NNNNGGGG", "!!!!!!!!"),
            ],
        )?;

        let report = demultiplex_reads(
            &input,
            &[
                DemultiplexRule { sample_id: "sample_a", barcode: "ACGT" },
                DemultiplexRule { sample_id: "sample_b", barcode: "TGCA" },
            ],
            1,
            &temp.path().join("demux"),
            &temp.path().join("undetermined.fastq"),
        )?;

        assert_eq!(report.assigned_reads, 3);
        assert_eq!(report.undetermined_reads, 1);
        Ok(())
    }

    #[test]
    fn demultiplex_reads_refuses_invalid_sample_sheet() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-demultiplex-reads")?;
        let input = temp.path().join("reads.fastq");
        write_fastq(&input, &[("r1", "ACGTAAAA", "!!!!!!!!")])?;

        let result = demultiplex_reads(
            &input,
            &[
                DemultiplexRule { sample_id: "sample_a", barcode: "ACGT" },
                DemultiplexRule { sample_id: "sample_b", barcode: "ACGT" },
            ],
            0,
            &temp.path().join("demux"),
            &temp.path().join("undetermined.fastq"),
        );

        assert!(result.is_err());
        Ok(())
    }
}
