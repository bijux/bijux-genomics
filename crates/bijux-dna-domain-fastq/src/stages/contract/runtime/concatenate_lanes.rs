use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    ConcatenateLaneSummaryV1, ConcatenateLanesReportV1, CONCATENATE_LANES_REPORT_SCHEMA_VERSION,
};

use super::fastq_io::{parse_header_pairing, read_fastq_records, write_fastq_records, FastqRecord};

pub struct LaneInput<'a> {
    pub lane_id: &'a str,
    pub r1: &'a Path,
    pub r2: Option<&'a Path>,
}

/// Concatenate lane FASTQs while tracking lane provenance and duplicate run signals.
///
/// # Errors
/// Returns an error when lane inputs are incoherent or outputs cannot be written.
pub fn concatenate_lanes(
    lanes: &[LaneInput<'_>],
    output_r1: &Path,
    output_r2: Option<&Path>,
) -> Result<ConcatenateLanesReportV1> {
    if lanes.is_empty() {
        return Err(anyhow!("fastq.concatenate_lanes requires at least one lane input"));
    }

    let paired_mode = lanes.iter().any(|lane| lane.r2.is_some());
    if paired_mode && output_r2.is_none() {
        return Err(anyhow!(
            "fastq.concatenate_lanes requires output_r2 for paired-end lane inputs"
        ));
    }
    if paired_mode && lanes.iter().any(|lane| lane.r2.is_none()) {
        return Err(anyhow!(
            "fastq.concatenate_lanes refused mixed single-end and paired-end lane inputs"
        ));
    }

    let mut merged_r1 = Vec::<FastqRecord>::new();
    let mut merged_r2 = Vec::<FastqRecord>::new();
    let mut lane_summaries = Vec::<ConcatenateLaneSummaryV1>::new();
    let mut seen_ids = BTreeSet::<String>::new();
    let mut duplicate_count = 0_u64;
    let mut duplicate_examples = Vec::<String>::new();

    for lane in lanes {
        let r1_records = read_fastq_records(lane.r1)?;
        let r2_records = if let Some(r2_path) = lane.r2 {
            let records = read_fastq_records(r2_path)?;
            if records.len() != r1_records.len() {
                return Err(anyhow!(
                    "fastq.concatenate_lanes refused lane {}: R1 count {} != R2 count {}",
                    lane.lane_id,
                    r1_records.len(),
                    records.len()
                ));
            }
            records
        } else {
            Vec::new()
        };

        for record in &r1_records {
            let (base, _) = parse_header_pairing(record.header.trim_start_matches('@'));
            if !seen_ids.insert(base.clone()) {
                duplicate_count += 1;
                if duplicate_examples.len() < 10 {
                    duplicate_examples.push(base);
                }
            }
        }

        merged_r1.extend(r1_records.iter().cloned());
        if paired_mode {
            merged_r2.extend(r2_records.iter().cloned());
        }

        lane_summaries.push(ConcatenateLaneSummaryV1 {
            lane_id: lane.lane_id.to_string(),
            reads_r1: r1_records.len() as u64,
            reads_r2: paired_mode.then_some(r2_records.len() as u64),
        });
    }

    write_fastq_records(output_r1, &merged_r1)?;
    if paired_mode {
        let out_r2 = output_r2.expect("validated above");
        write_fastq_records(out_r2, &merged_r2)?;
    }

    Ok(ConcatenateLanesReportV1 {
        schema_version: CONCATENATE_LANES_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.concatenate_lanes".to_string(),
        stage_id: "fastq.concatenate_lanes".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired_mode { "paired_end" } else { "single_end" }.to_string(),
        lanes_concatenated: lanes.len() as u64,
        reads_in_r1: merged_r1.len() as u64,
        reads_in_r2: paired_mode.then_some(merged_r2.len() as u64),
        duplicate_read_ids_detected: duplicate_count,
        duplicate_read_id_examples: duplicate_examples,
        output_r1_reads: merged_r1.len() as u64,
        output_r2_reads: paired_mode.then_some(merged_r2.len() as u64),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        lanes: lane_summaries,
    })
}

#[cfg(test)]
mod tests {
    use super::{concatenate_lanes, LaneInput};

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn concatenate_lanes_merges_paired_lanes_and_detects_duplicates() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-concatenate-lanes")?;

        let lane1_r1 = temp.path().join("lane1_R1.fastq");
        let lane1_r2 = temp.path().join("lane1_R2.fastq");
        let lane2_r1 = temp.path().join("lane2_R1.fastq");
        let lane2_r2 = temp.path().join("lane2_R2.fastq");

        write_fastq(&lane1_r1, &[("A/1", "AAAA", "!!!!"), ("B/1", "CCCC", "####")])?;
        write_fastq(&lane1_r2, &[("A/2", "TTTT", "!!!!"), ("B/2", "GGGG", "####")])?;

        write_fastq(&lane2_r1, &[("B/1", "CCCC", "####"), ("C/1", "GGGG", "!!!!")])?;
        write_fastq(&lane2_r2, &[("B/2", "GGGG", "####"), ("C/2", "AAAA", "!!!!")])?;

        let report = concatenate_lanes(
            &[
                LaneInput {
                    lane_id: "L001",
                    r1: &lane1_r1,
                    r2: Some(&lane1_r2),
                },
                LaneInput {
                    lane_id: "L002",
                    r1: &lane2_r1,
                    r2: Some(&lane2_r2),
                },
            ],
            &temp.path().join("merged_R1.fastq"),
            Some(&temp.path().join("merged_R2.fastq")),
        )?;

        assert_eq!(report.lanes_concatenated, 2);
        assert_eq!(report.output_r1_reads, 4);
        assert_eq!(report.duplicate_read_ids_detected, 1);
        Ok(())
    }
}
