use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
use crate::params::merge::{MergeEffectiveParams, UnmergedReadPolicy};
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

/// Merge paired-end reads with a deterministic native overlap join suitable for governed local
/// smoke execution.
///
/// # Errors
/// Returns an error when paired inputs are incoherent, output wiring is incomplete, or the merge
/// invariants cannot be satisfied.
pub fn merge_pairs(
    r1: &Path,
    r2: &Path,
    params: &MergeEffectiveParams,
    merged_reads: &Path,
    unmerged_reads_r1: Option<&Path>,
    unmerged_reads_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<MergePairsReportV1> {
    let left = read_fastq_records(r1)?;
    let right = read_fastq_records(r2)?;
    if left.len() != right.len() {
        return Err(anyhow!(
            "fastq.merge_pairs refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs
        && (unmerged_reads_r1.is_none() || unmerged_reads_r2.is_none())
    {
        return Err(anyhow!("fastq.merge_pairs requires unmerged outputs when policy emits pairs"));
    }

    let minimum_overlap = params.merge_overlap.unwrap_or(10).max(1) as usize;
    let mut merged_records = Vec::<FastqRecord>::new();
    let mut unmerged_left = Vec::<FastqRecord>::new();
    let mut unmerged_right = Vec::<FastqRecord>::new();

    for (left_record, right_record) in left.iter().cloned().zip(right.iter().cloned()) {
        if let Some((sequence, quality)) = merge_record_pair(&left_record, &right_record, minimum_overlap)
        {
            merged_records.push(FastqRecord {
                header: left_record.header.clone(),
                sequence,
                plus: left_record.plus.clone(),
                quality,
            });
        } else {
            unmerged_left.push(left_record);
            unmerged_right.push(right_record);
        }
    }

    if let Some(min_len) = params.min_len {
        merged_records.retain(|record| record.sequence.len() >= min_len as usize);
    }

    write_fastq_records(merged_reads, &merged_records)?;
    if params.unmerged_read_policy == UnmergedReadPolicy::EmitUnmergedPairs {
        write_fastq_records(
            unmerged_reads_r1.expect("validated above"),
            &unmerged_left,
        )?;
        write_fastq_records(
            unmerged_reads_r2.expect("validated above"),
            &unmerged_right,
        )?;
    }

    let reads_merged = merged_records.len() as u64;
    let reads_unmerged = unmerged_left.len() as u64;
    let reads_r1 = left.len() as u64;
    let reads_r2 = right.len() as u64;
    let merge_rate = if reads_r1 == 0 { 0.0 } else { reads_merged as f64 / reads_r1 as f64 };

    let report = MergePairsReportV1 {
        schema_version: MERGE_PAIRS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.merge_pairs".to_string(),
        stage_id: "fastq.merge_pairs".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: PairedMode::PairedEnd,
        merge_engine: params.merge_engine.clone(),
        threads: params.threads,
        merge_overlap: params.merge_overlap,
        min_len: params.min_len,
        unmerged_read_policy: params.unmerged_read_policy.clone(),
        input_r1: r1.display().to_string(),
        input_r2: r2.display().to_string(),
        merged_reads: merged_reads.display().to_string(),
        unmerged_reads_r1: unmerged_reads_r1.map(|path| path.display().to_string()),
        unmerged_reads_r2: unmerged_reads_r2.map(|path| path.display().to_string()),
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        merge_rate,
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "bijux_merge_trace".to_string()),
    };

    bijux_dna_infra::atomic_write_json(report_json, &report)?;
    if let Some(raw_backend_report) = raw_backend_report {
        std::fs::write(
            raw_backend_report,
            format!(
                "pairs_in\t{}\nreads_merged\t{}\nreads_unmerged\t{}\nminimum_overlap\t{}\n",
                reads_r1, reads_merged, reads_unmerged, minimum_overlap
            ),
        )?;
    }
    Ok(report)
}

fn merge_record_pair(
    left: &FastqRecord,
    right: &FastqRecord,
    minimum_overlap: usize,
) -> Option<(String, String)> {
    let right_sequence = reverse_complement(&right.sequence);
    let right_quality = right.quality.chars().rev().collect::<String>();
    let maximum_overlap = left.sequence.len().min(right_sequence.len());
    for overlap in (minimum_overlap..=maximum_overlap).rev() {
        if left.sequence.ends_with(&right_sequence[..overlap]) {
            let merged_sequence =
                format!("{}{}", left.sequence, &right_sequence[overlap..]);
            let merged_quality =
                format!("{}{}", left.quality, &right_quality[overlap..]);
            return Some((merged_sequence, merged_quality));
        }
    }
    None
}

fn reverse_complement(sequence: &str) -> String {
    sequence
        .chars()
        .rev()
        .map(|base| match base {
            'A' | 'a' => 'T',
            'C' | 'c' => 'G',
            'G' | 'g' => 'C',
            'T' | 't' => 'A',
            'N' | 'n' => 'N',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::merge_pairs;
    use crate::params::merge::{MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION};
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
    fn merge_pairs_reports_merged_and_unmerged_pairs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-merge-pairs")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        write_fastq(
            &r1,
            &[
                ("merge_pair/1", "ACGTACGTGGGG", "IIIIIIIIIIII"),
                ("separate_pair/1", "GGGGAAAACCCC", "JJJJJJJJJJJJ"),
            ],
        )?;
        write_fastq(
            &r2,
            &[
                ("merge_pair/2", "TTAACCCCACGT", "IIIIIIIIIIII"),
                ("separate_pair/2", "ATATATATATAT", "JJJJJJJJJJJJ"),
            ],
        )?;

        let report = merge_pairs(
            &r1,
            &r2,
            &MergeEffectiveParams {
                schema_version: MERGE_SCHEMA_VERSION.to_string(),
                paired_mode: PairedMode::PairedEnd,
                threads: 1,
                merge_overlap: Some(8),
                min_len: Some(12),
                merge_engine: MergeEngine::Pear,
                unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
            },
            &temp.path().join("merged.fastq.gz"),
            Some(&temp.path().join("unmerged_R1.fastq.gz")),
            Some(&temp.path().join("unmerged_R2.fastq.gz")),
            &temp.path().join("report.json"),
            Some(&temp.path().join("raw_backend_report.txt")),
        )?;

        assert_eq!(report.tool_id, "bijux");
        assert_eq!(report.reads_merged, 1);
        assert_eq!(report.reads_unmerged, 1);
        assert_eq!(report.merge_rate, 0.5);

        let merged = read_fastq_records(&temp.path().join("merged.fastq.gz"))?;
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].sequence, "ACGTACGTGGGGTTAA");

        let unmerged_r1 = read_fastq_records(&temp.path().join("unmerged_R1.fastq.gz"))?;
        let unmerged_r2 = read_fastq_records(&temp.path().join("unmerged_R2.fastq.gz"))?;
        assert_eq!(unmerged_r1.len(), 1);
        assert_eq!(unmerged_r2.len(), 1);
        assert_eq!(unmerged_r1[0].sequence, "GGGGAAAACCCC");
        assert_eq!(unmerged_r2[0].sequence, "ATATATATATAT");
        Ok(())
    }
}
