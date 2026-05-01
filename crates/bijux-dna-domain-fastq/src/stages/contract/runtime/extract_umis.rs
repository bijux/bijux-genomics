use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{ExtractUmisReportV1, EXTRACT_UMIS_REPORT_SCHEMA_VERSION};
use crate::params::umi::{
    FastqUmiParams, UmiExtractionLocation, UmiFailedExtractionPolicy, UmiReadNameTransform,
};
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

fn pattern_len(params: &FastqUmiParams) -> usize {
    params
        .umi_pattern
        .as_deref()
        .map(|pattern| pattern.chars().filter(|ch| *ch == 'N').count())
        .filter(|count| *count > 0)
        .unwrap_or(8)
}

fn extract_umi(
    record: &FastqRecord,
    location: &UmiExtractionLocation,
    size: usize,
) -> Option<String> {
    match location {
        UmiExtractionLocation::Read1Prefix | UmiExtractionLocation::Read2Prefix => {
            let prefix = record.sequence.chars().take(size).collect::<String>();
            (prefix.len() == size && prefix.chars().all(|base| "ACGTNacgtn".contains(base)))
                .then_some(prefix.to_ascii_uppercase())
        }
        UmiExtractionLocation::HeaderTag => record
            .header
            .split_whitespace()
            .find_map(|token| token.strip_prefix("umi:"))
            .map(ToString::to_string),
        UmiExtractionLocation::IndexRead => None,
    }
}

fn apply_header_transform(
    record: &mut FastqRecord,
    transform: &UmiReadNameTransform,
    umi: Option<&str>,
) {
    let Some(umi) = umi else {
        return;
    };

    let header = record.header.trim_start_matches('@');
    let updated = match transform {
        UmiReadNameTransform::AppendToHeader => format!("@{} umi:{}", header, umi),
        UmiReadNameTransform::ReplaceHeader => format!("@umi:{}", umi),
        UmiReadNameTransform::None => record.header.clone(),
    };
    record.header = updated;
}

/// Extract and group UMIs from FASTQ streams with governed failure handling.
///
/// # Errors
/// Returns an error when input is incoherent or configured policy refuses failed extraction.
pub fn extract_umis(
    r1: &Path,
    r2: Option<&Path>,
    params: &FastqUmiParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<ExtractUmisReportV1> {
    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.extract_umis refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if paired && output_r2.is_none() {
        return Err(anyhow!("fastq.extract_umis requires output_r2 for paired input"));
    }

    let umi_len = pattern_len(params);
    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mut out_left = Vec::<FastqRecord>::new();
    let mut out_right = Vec::<FastqRecord>::new();
    let mut reads_with_umi = 0_u64;
    let mut failed_extractions = 0_u64;
    let mut umi_groups = BTreeSet::<String>::new();

    for idx in 0..left.len() {
        let mut l = left[idx].clone();
        let mut r = paired.then(|| right[idx].clone());

        let left_source = match params.extraction_location {
            UmiExtractionLocation::Read2Prefix if paired => r.as_ref().unwrap_or(&l),
            _ => &l,
        };

        let umi = extract_umi(left_source, &params.extraction_location, umi_len);
        if umi.is_none() {
            failed_extractions += if paired { 2 } else { 1 };
            match params.failed_extraction_policy {
                UmiFailedExtractionPolicy::RefuseStage => {
                    return Err(anyhow!(
                        "fastq.extract_umis refused because UMI extraction failed under refuse_stage policy"
                    ));
                }
                UmiFailedExtractionPolicy::RouteToRejected => {
                    continue;
                }
                UmiFailedExtractionPolicy::RetainUnmodified => {
                    out_left.push(l);
                    if let Some(r) = r {
                        out_right.push(r);
                    }
                    continue;
                }
            }
        }

        let umi = umi.expect("checked above");
        umi_groups.insert(umi.clone());
        apply_header_transform(&mut l, &params.read_name_transform, Some(&umi));
        if let Some(ref mut record_r2) = r {
            apply_header_transform(record_r2, &params.read_name_transform, Some(&umi));
        }

        reads_with_umi += if paired { 2 } else { 1 };
        out_left.push(l);
        if let Some(r) = r {
            out_right.push(r);
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

    let mean_q_before = {
        let sum = mean_q(&left) + mean_q(&right);
        if paired {
            sum / 2.0
        } else {
            sum
        }
    };
    let mean_q_after = {
        let sum = mean_q(&out_left) + mean_q(&out_right);
        if paired {
            sum / 2.0
        } else {
            sum
        }
    };

    Ok(ExtractUmisReportV1 {
        schema_version: EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.extract_umis".to_string(),
        stage_id: "fastq.extract_umis".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        umi_pattern: params.umi_pattern.clone().unwrap_or_else(|| "NNNNNNNN".to_string()),
        extraction_location: params.extraction_location.clone(),
        read_name_transform: params.read_name_transform.clone(),
        failed_extraction_policy: params.failed_extraction_policy.clone(),
        grouping_policy: params.grouping_policy.clone(),
        downstream_dedup_policy: params.downstream_dedup_policy.clone(),
        downstream_propagation: params.downstream_propagation.clone(),
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(out_left.len() as u64),
        reads_with_umi,
        failed_extractions: Some(failed_extractions),
        mean_q_before,
        mean_q_after,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "umi_tools_log".to_string()),
        backend_metrics: Some(serde_json::json!({
            "umi_group_count": umi_groups.len() as u64,
            "reads_with_umi_fraction": if reads_out == 0 { 0.0 } else { reads_with_umi as f64 / reads_out as f64 },
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::extract_umis;
    use crate::params::umi::{
        FastqUmiParams, UmiDedupPolicy, UmiDownstreamPropagation, UmiExtractionLocation,
        UmiFailedExtractionPolicy, UmiGroupingPolicy, UmiReadNameTransform,
    };
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
    fn extract_umis_appends_umi_and_counts_groups() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-extract-umis")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(&r1, &[("a", "ACGTGGGG", "IIIIIIII"), ("b", "ACGTTTTT", "IIIIIIII")])?;

        let params = FastqUmiParams {
            schema_version: "bijux.fastq.params.umi.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            umi_pattern: Some("NNNN".to_string()),
            extraction_location: UmiExtractionLocation::Read1Prefix,
            read_name_transform: UmiReadNameTransform::AppendToHeader,
            failed_extraction_policy: UmiFailedExtractionPolicy::RetainUnmodified,
            grouping_policy: UmiGroupingPolicy::ExactSequence,
            downstream_dedup_policy: UmiDedupPolicy::ObservationOnly,
            downstream_propagation: UmiDownstreamPropagation::HeaderAndReport,
        };

        let report = extract_umis(
            &r1,
            None,
            &params,
            &temp.path().join("out.fastq"),
            None,
            &temp.path().join("report.json"),
            None,
        )?;

        assert_eq!(report.reads_with_umi, 2);
        assert_eq!(report.failed_extractions, Some(0));
        Ok(())
    }

    #[test]
    fn extract_umis_refuses_when_policy_requires_success() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-extract-umis")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(&r1, &[("a", "AAA", "III")])?;

        let params = FastqUmiParams {
            schema_version: "bijux.fastq.params.umi.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            umi_pattern: Some("NNNN".to_string()),
            extraction_location: UmiExtractionLocation::Read1Prefix,
            read_name_transform: UmiReadNameTransform::AppendToHeader,
            failed_extraction_policy: UmiFailedExtractionPolicy::RefuseStage,
            grouping_policy: UmiGroupingPolicy::ExactSequence,
            downstream_dedup_policy: UmiDedupPolicy::ObservationOnly,
            downstream_propagation: UmiDownstreamPropagation::HeaderOnly,
        };

        let result = extract_umis(
            &r1,
            None,
            &params,
            &temp.path().join("out.fastq"),
            None,
            &temp.path().join("report.json"),
            None,
        );

        assert!(result.is_err());
        Ok(())
    }
}
