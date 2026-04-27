use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_fastq::metrics::{
    FastqDeduplicateMetricsV1, FastqDeltaMetricsV1, FastqMergeMetricsV1, FastqValidateMetricsV1,
    RetentionReportMetricV1,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    f64_from_u64, path_from_params, retention_conditions_from_effective, stats_for_paths,
    zero_seqkit_metrics,
};

pub(super) fn deduplicate_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
    ])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("deduplicate_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_remove_duplicates_report(&raw).ok());
    let reads_in = parsed_report.as_ref().map_or(input.reads, |report| report.reads_in);
    let reads_out = parsed_report.as_ref().map_or(output.reads, |report| report.reads_out);
    let (pairs_in, pairs_out) = (
        parsed_report.as_ref().and_then(|report| report.pairs_in),
        parsed_report.as_ref().and_then(|report| report.pairs_out),
    );
    let read_retention =
        if reads_in > 0 { f64_from_u64(reads_out) / f64_from_u64(reads_in) } else { 0.0 };
    let base_retention =
        if input.bases > 0 { f64_from_u64(output.bases) / f64_from_u64(input.bases) } else { 0.0 };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: output.mean_q - input.mean_q,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: reads_out,
        denominator_reads: reads_in,
        numerator_bases: output.bases,
        denominator_bases: input.bases,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: plan.stage_id.to_string(),
        conditions: retention_conditions_from_effective(
            &plan.stage_id,
            &plan.effective_params,
            &plan.params,
        ),
    };
    Ok(serde_json::to_value(FastqDeduplicateMetricsV1 {
        reads_in,
        reads_out,
        reads_removed_duplicates: reads_in.saturating_sub(reads_out),
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        paired_mode: parsed_report
            .as_ref()
            .and_then(|report| serde_json::to_value(report.paired_mode).ok())
            .and_then(|value| value.as_str().map(ToString::to_string)),
        dedup_mode: parsed_report.as_ref().map(|report| match &report.dedup_mode {
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::Exact => {
                "exact".to_string()
            }
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::SequenceIdentity => {
                "sequence_identity".to_string()
            }
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::OpticalAware => {
                "optical_aware".to_string()
            }
        }),
        keep_order: parsed_report.as_ref().map(|report| report.keep_order),
        pair_count_match: parsed_report.as_ref().and_then(|report| report.pair_count_match),
        duplicate_class_count: parsed_report
            .as_ref()
            .map(|report| report.duplicate_classes.len() as u64),
        duplicate_provenance_json: parsed_report
            .as_ref()
            .and_then(|report| report.duplicate_provenance_json.clone()),
        raw_backend_report_format: parsed_report
            .as_ref()
            .and_then(|report| report.raw_backend_report_format.clone()),
        delta_metrics: delta,
        retention,
    })?)
}

pub(super) fn merge_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        inputs.get(1).map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
        outputs.get(1).map(PathBuf::as_path),
        outputs.get(2).map(PathBuf::as_path),
    ])?;
    let r1 = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let r2 = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let merged = stats.get(2).copied().unwrap_or_else(zero_seqkit_metrics);
    let unmerged_r1 = stats.get(3).copied().unwrap_or_else(zero_seqkit_metrics);
    let unmerged_r2 = stats.get(4).copied().unwrap_or_else(zero_seqkit_metrics);
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("merge_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_merge_pairs_report(&raw).ok());
    let reads_r1 = parsed_report.as_ref().map_or(r1.reads, |report| report.reads_r1);
    let reads_r2 = parsed_report.as_ref().map_or(r2.reads, |report| report.reads_r2);
    let reads_merged = parsed_report.as_ref().map_or(merged.reads, |report| report.reads_merged);
    let reads_unmerged = parsed_report
        .as_ref()
        .map_or_else(|| unmerged_r1.reads.min(unmerged_r2.reads), |report| report.reads_unmerged);
    let min_reads = reads_r1.min(reads_r2);
    let merge_rate = parsed_report.as_ref().map_or_else(
        || {
            if min_reads > 0 {
                f64_from_u64(reads_merged) / f64_from_u64(min_reads)
            } else {
                0.0
            }
        },
        |report| report.merge_rate,
    );
    let bases_in = r1.bases.min(r2.bases);
    let mean_q_in = (r1.mean_q + r2.mean_q) / 2.0;
    let merge_q_delta = merged.mean_q - mean_q_in;
    Ok(serde_json::to_value(FastqMergeMetricsV1 {
        reads_in: min_reads,
        reads_out: reads_merged,
        bases_in,
        bases_out: merged.bases,
        pairs_in: Some(min_reads),
        pairs_out: Some(reads_merged),
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        reads_discarded: 0,
        merge_rate,
        merge_q_delta,
    })?)
}

pub(super) fn validate_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        inputs.get(1).map(PathBuf::as_path),
    ])?;
    let input_r1 = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let input_r2 = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let pairs_in = inputs.get(1).map(|_| input_r1.reads.min(input_r2.reads));
    let pairs_out = pairs_in;
    let reads_in = input_r1.reads + inputs.get(1).map_or(0, |_| input_r2.reads);
    let bases_in = input_r1.bases + inputs.get(1).map_or(0, |_| input_r2.bases);
    let mean_q = if inputs.get(1).is_some() {
        (input_r1.mean_q + input_r2.mean_q) / 2.0
    } else {
        input_r1.mean_q
    };
    let report_metrics = path_from_params(&plan.params, "report_json")
        .and_then(|report_path| std::fs::read_to_string(&report_path).ok())
        .and_then(|raw| crate::observer::parse_validation_report(&raw).ok())
        .map(|report| {
            let reads_total = report.validated_reads_r1 + report.validated_reads_r2.unwrap_or(0);
            let reads_invalid = match report.failure_class {
                bijux_dna_domain_fastq::ValidateFailureClass::None
                | bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => 0,
                bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => {
                    report.validated_reads_r1.abs_diff(report.validated_reads_r2.unwrap_or(0))
                }
                bijux_dna_domain_fastq::ValidateFailureClass::ValidatorError => {
                    let mut invalid = 0;
                    if report.status_r1 != 0 {
                        invalid += report.validated_reads_r1;
                    }
                    if report.status_r2 != 0 {
                        invalid += report.validated_reads_r2.unwrap_or(0);
                    }
                    invalid
                }
            }
            .min(reads_total);
            FastqValidateMetricsV1 {
                reads_in,
                reads_out: reads_in,
                bases_in,
                bases_out: bases_in,
                pairs_in,
                pairs_out,
                reads_total,
                reads_valid: reads_total.saturating_sub(reads_invalid),
                reads_invalid,
                mean_q,
                validated_inputs: Some(report.validated_inputs),
                validated_pairs: report.validated_pairs,
                pair_sync_checked: Some(report.pair_sync_checked),
                pair_sync_pass: report.pair_sync_pass,
                pair_count_match: report.pair_count_match,
                strict_pass: Some(report.strict_pass),
                failure_class: serde_json::to_value(&report.failure_class)
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned)),
            }
        });
    Ok(serde_json::to_value(report_metrics.unwrap_or(FastqValidateMetricsV1 {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in,
        pairs_out,
        reads_total: reads_in,
        reads_valid: reads_in,
        reads_invalid: 0,
        mean_q,
        validated_inputs: None,
        validated_pairs: None,
        pair_sync_checked: None,
        pair_sync_pass: None,
        pair_count_match: None,
        strict_pass: None,
        failure_class: None,
    }))?)
}
