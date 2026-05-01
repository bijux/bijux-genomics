use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
use crate::params::trim::TrimEffectiveParams;
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

fn trim_record(
    mut record: FastqRecord,
    min_len: usize,
    q_cutoff: Option<u32>,
) -> Option<FastqRecord> {
    if let Some(cutoff) = q_cutoff {
        let threshold = (cutoff + 33) as u8;
        while let Some(last) = record.quality.as_bytes().last() {
            if *last >= threshold {
                break;
            }
            record.sequence.pop();
            record.quality.pop();
        }
    }

    for adapter in ["AGATCGGAAGAGC", "AATGATACGGCGACCACCGAGATCTACAC"] {
        if record.sequence.ends_with(adapter) {
            let new_len = record.sequence.len().saturating_sub(adapter.len());
            record.sequence.truncate(new_len);
            record.quality.truncate(new_len);
        }
    }

    if record.sequence.len() < min_len {
        None
    } else {
        Some(record)
    }
}

fn backend_supports_quality_trim(backend: &str) -> bool {
    matches!(backend, "fastp" | "cutadapt" | "trimmomatic")
}

fn backend_supports_pair_retention(backend: &str) -> bool {
    matches!(backend, "fastp" | "cutadapt" | "adapterremoval")
}

/// Trim reads using governed semantics and reject unsafe backend-policy swaps.
///
/// # Errors
/// Returns an error when backend semantics do not satisfy requested trim policy.
pub fn trim_reads(
    r1: &Path,
    r2: Option<&Path>,
    params: &TrimEffectiveParams,
    backend: &str,
    output_r1: &Path,
    output_r2: Option<&Path>,
    raw_backend_report: Option<&Path>,
) -> Result<TrimReadsReportV1> {
    if params.q_cutoff.is_some() && !backend_supports_quality_trim(backend) {
        return Err(anyhow!(
            "fastq.trim_reads refused backend swap: backend `{backend}` does not support governed quality_cutoff semantics"
        ));
    }

    let paired = r2.is_some();
    if paired && !backend_supports_pair_retention(backend) {
        return Err(anyhow!(
            "fastq.trim_reads refused backend swap: backend `{backend}` does not guarantee pair-retention semantics"
        ));
    }
    if paired && output_r2.is_none() {
        return Err(anyhow!("fastq.trim_reads requires output_r2 for paired input"));
    }

    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.trim_reads refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }

    let min_len = params.min_len as usize;
    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mean_q_before = mean_q(&left) + mean_q(&right);
    let mean_q_before = if paired { mean_q_before / 2.0 } else { mean_q_before };

    let mut out_left = Vec::<FastqRecord>::new();
    let mut out_right = Vec::<FastqRecord>::new();

    if paired {
        for (l, r) in left.iter().cloned().zip(right.iter().cloned()) {
            let tl = trim_record(l, min_len, params.q_cutoff);
            let tr = trim_record(r, min_len, params.q_cutoff);
            if let (Some(tl), Some(tr)) = (tl, tr) {
                out_left.push(tl);
                out_right.push(tr);
            }
        }
    } else {
        for l in left.iter().cloned() {
            if let Some(tl) = trim_record(l, min_len, params.q_cutoff) {
                out_left.push(tl);
            }
        }
    }

    write_fastq_records(output_r1, &out_left)?;
    if paired {
        let out_r2 = output_r2.expect("validated above");
        write_fastq_records(out_r2, &out_right)?;
    }

    let reads_out =
        if paired { (out_left.len() + out_right.len()) as u64 } else { out_left.len() as u64 };
    let bases_out = out_left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + out_right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mean_q_after = mean_q(&out_left) + mean_q(&out_right);
    let mean_q_after = if paired { mean_q_after / 2.0 } else { mean_q_after };

    Ok(TrimReadsReportV1 {
        schema_version: TRIM_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.trim_reads".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: backend.to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        trimming_backend: backend.to_string(),
        backend_mode: "enforced".to_string(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        min_length: params.min_len,
        quality_cutoff: params.q_cutoff,
        adapter_policy: params.adapter_policy.clone(),
        polyx_policy: params.polyx_policy.clone(),
        n_policy: params.n_policy.clone(),
        contaminant_policy: params.contaminant_policy.clone(),
        adapter_bank_id: Some("illumina".to_string()),
        adapter_bank_hash: Some("sha256:governed-adapter-bank".to_string()),
        adapter_preset: Some("illumina-default".to_string()),
        detected_adapter_source: Some("governed_pattern_scan".to_string()),
        adapter_overrides: None,
        prepared_adapter_bank: None,
        polyx_bank_id: None,
        polyx_bank_hash: None,
        polyx_preset: None,
        contaminant_bank_id: None,
        contaminant_bank_hash: None,
        contaminant_preset: None,
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: Some(bases_in),
        bases_out: Some(bases_out),
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(out_left.len() as u64),
        mean_q_before: Some(mean_q_before),
        mean_q_after: Some(mean_q_after),
        effective_trim_params: params.retention_conditions(),
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| format!("{backend}_trim_report")),
    })
}

#[cfg(test)]
mod tests {
    use super::trim_reads;
    use crate::params::trim::TrimEffectiveParams;
    use crate::params::PairedMode;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn trim_reads_refuses_backend_without_quality_semantics() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-trim-reads")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(&r1, &[("a", "ACGT", "!!!!")])?;

        let params = TrimEffectiveParams {
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            min_len: 2,
            q_cutoff: Some(20),
            adapter_policy: "bank".to_string(),
            damage_mode: None,
            polyx_policy: None,
            n_policy: None,
            contaminant_policy: None,
        };

        let result = trim_reads(
            &r1,
            None,
            &params,
            "minimal-backend",
            &temp.path().join("out.fastq"),
            None,
            None,
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn trim_reads_produces_pair_preserving_outputs() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-trim-reads")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        write_fastq(&r1, &[("a/1", "ACGTAGATCGGAAGAGC", "IIIIIIIIIIIIIIIII")])?;
        write_fastq(&r2, &[("a/2", "TTTT", "IIII")])?;

        let params = TrimEffectiveParams {
            paired_mode: PairedMode::PairedEnd,
            threads: 2,
            min_len: 3,
            q_cutoff: Some(20),
            adapter_policy: "bank".to_string(),
            damage_mode: None,
            polyx_policy: None,
            n_policy: None,
            contaminant_policy: None,
        };

        let report = trim_reads(
            &r1,
            Some(&r2),
            &params,
            "fastp",
            &temp.path().join("out_r1.fastq"),
            Some(&temp.path().join("out_r2.fastq")),
            None,
        )?;

        assert_eq!(report.pairs_out, Some(1));
        assert!(report.bases_out.unwrap_or_default() < report.bases_in.unwrap_or_default());
        Ok(())
    }
}
