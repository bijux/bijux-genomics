use std::collections::HashMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    FilterLowComplexityReportV1, FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION,
};
use crate::params::quality::filter_low_complexity::FilterLowComplexityEffectiveParams;
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

fn mean_q(records: &[FastqRecord]) -> f64 {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for record in records {
        for byte in record.quality.bytes() {
            if byte >= 33 {
                total += (byte - 33) as u64;
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn normalized_shannon_entropy(sequence: &str) -> f64 {
    if sequence.is_empty() {
        return 0.0;
    }
    let mut counts = HashMap::<u8, usize>::new();
    for base in sequence.bytes().map(|base| base.to_ascii_uppercase()) {
        *counts.entry(base).or_default() += 1;
    }
    let len = sequence.len() as f64;
    let entropy = counts
        .values()
        .map(|count| {
            let p = *count as f64 / len;
            -(p * p.log2())
        })
        .sum::<f64>();
    entropy / 2.0
}

fn max_homopolymer_run(sequence: &str) -> u32 {
    let mut max_run = 0_u32;
    let mut current_run = 0_u32;
    let mut previous = None;
    for base in sequence.bytes().map(|base| base.to_ascii_uppercase()) {
        if Some(base) == previous {
            current_run += 1;
        } else {
            current_run = 1;
            previous = Some(base);
        }
        max_run = max_run.max(current_run);
    }
    max_run
}

fn record_is_low_complexity(
    record: &FastqRecord,
    params: &FilterLowComplexityEffectiveParams,
) -> (bool, bool) {
    let polyx_triggered = params
        .polyx_threshold
        .is_some_and(|threshold| max_homopolymer_run(&record.sequence) >= threshold);
    let entropy_triggered = normalized_shannon_entropy(&record.sequence) < params.entropy_threshold;
    (polyx_triggered, entropy_triggered)
}

/// Filter low-complexity reads with deterministic governed semantics suitable for local smoke
/// execution.
///
/// # Errors
/// Returns an error when paired inputs are incoherent or governed output wiring is incomplete.
pub fn filter_low_complexity(
    r1: &Path,
    r2: Option<&Path>,
    params: &FilterLowComplexityEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<FilterLowComplexityReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.filter_low_complexity refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        if output_r2.is_none() {
            return Err(anyhow!(
                "fastq.filter_low_complexity requires output_r2 when paired input is provided"
            ));
        }
        records
    } else {
        Vec::new()
    };

    let paired = r2.is_some();
    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|record| record.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|record| record.sequence.len() as u64).sum::<u64>();
    let mean_q_before = if paired { (mean_q(&left) + mean_q(&right)) / 2.0 } else { mean_q(&left) };

    let mut output_left = Vec::<FastqRecord>::new();
    let mut output_right = Vec::<FastqRecord>::new();
    let mut reads_removed_low_complexity = 0_u64;
    let mut reads_removed_by_polyx = 0_u64;
    let mut reads_removed_by_entropy = 0_u64;

    if paired {
        for (left_record, right_record) in left.iter().cloned().zip(right.iter().cloned()) {
            let (left_polyx, left_entropy) = record_is_low_complexity(&left_record, params);
            let (right_polyx, right_entropy) = record_is_low_complexity(&right_record, params);
            if left_polyx || left_entropy || right_polyx || right_entropy {
                reads_removed_low_complexity += 2;
                reads_removed_by_polyx += u64::from(left_polyx) + u64::from(right_polyx);
                reads_removed_by_entropy += u64::from(left_entropy) + u64::from(right_entropy);
            } else {
                output_left.push(left_record);
                output_right.push(right_record);
            }
        }
    } else {
        for record in left.iter().cloned() {
            let (polyx_triggered, entropy_triggered) = record_is_low_complexity(&record, params);
            if polyx_triggered || entropy_triggered {
                reads_removed_low_complexity += 1;
                reads_removed_by_polyx += u64::from(polyx_triggered);
                reads_removed_by_entropy += u64::from(entropy_triggered);
            } else {
                output_left.push(record);
            }
        }
    }

    write_fastq_records(output_r1, &output_left)?;
    if let (Some(output_r2), true) = (output_r2, paired) {
        write_fastq_records(output_r2, &output_right)?;
    }

    let reads_out = if paired {
        (output_left.len() + output_right.len()) as u64
    } else {
        output_left.len() as u64
    };
    let bases_out = output_left.iter().map(|record| record.sequence.len() as u64).sum::<u64>()
        + output_right.iter().map(|record| record.sequence.len() as u64).sum::<u64>();
    let mean_q_after = if paired {
        (mean_q(&output_left) + mean_q(&output_right)) / 2.0
    } else {
        mean_q(&output_left)
    };

    if let Some(raw_backend_report) = raw_backend_report {
        std::fs::write(
            raw_backend_report,
            format!(
                "reads_in\t{reads_in}\nreads_out\t{reads_out}\nreads_removed_low_complexity\t{reads_removed_low_complexity}\nreads_removed_by_polyx\t{reads_removed_by_polyx}\nreads_removed_by_entropy\t{reads_removed_by_entropy}\nentropy_threshold\t{}\npolyx_threshold\t{}\n",
                params.entropy_threshold,
                params
                    .polyx_threshold
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ),
        )?;
    }

    let report = FilterLowComplexityReportV1 {
        schema_version: FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.filter_low_complexity".to_string(),
        stage_id: "fastq.filter_low_complexity".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        entropy_threshold: Some(params.entropy_threshold),
        polyx_threshold: params.polyx_threshold,
        reads_in,
        reads_out,
        reads_removed_low_complexity,
        bases_in,
        bases_out,
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(output_left.len() as u64),
        mean_q_before,
        mean_q_after,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report
            .map(|_| "bijux_filter_low_complexity_trace".to_string()),
        backend_metrics: Some(serde_json::json!({
            "reads_removed_by_polyx": reads_removed_by_polyx,
            "reads_removed_by_entropy": reads_removed_by_entropy,
        })),
    };
    bijux_dna_infra::atomic_write_json(report_json, &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::filter_low_complexity;
    use crate::params::quality::filter_low_complexity::{
        FilterLowComplexityEffectiveParams, FILTER_LOW_COMPLEXITY_SCHEMA_VERSION,
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
    fn filter_low_complexity_drops_polyx_and_low_entropy_reads() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-filter-low-complexity")?;
        let r1 = temp.path().join("reads.fastq");
        write_fastq(
            &r1,
            &[
                ("polyx_drop", "AAAAAAAAAAAA", "IIIIIIIIIIII"),
                ("entropy_drop", "ATATATATATAT", "JJJJJJJJJJJJ"),
                ("diverse_keep", "ACGTTGCAAGTC", "HHHHHHHHHHHH"),
            ],
        )?;

        let report = filter_low_complexity(
            &r1,
            None,
            &FilterLowComplexityEffectiveParams {
                schema_version: FILTER_LOW_COMPLEXITY_SCHEMA_VERSION.to_string(),
                paired_mode: PairedMode::SingleEnd,
                threads: 1,
                entropy_threshold: 0.6,
                polyx_threshold: Some(8),
            },
            &temp.path().join("filtered.fastq.gz"),
            None,
            &temp.path().join("report.json"),
            Some(&temp.path().join("raw_backend_report.txt")),
        )?;

        assert_eq!(report.tool_id, "bijux");
        assert_eq!(report.reads_in, 3);
        assert_eq!(report.reads_out, 1);
        assert_eq!(report.reads_removed_low_complexity, 2);

        let filtered = read_fastq_records(&temp.path().join("filtered.fastq.gz"))?;
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].header, "@diverse_keep");
        Ok(())
    }
}
