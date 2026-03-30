use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_domain_fastq::metrics::*;
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    f64_from_u64, filter_removals_for_plan, pair_counts_from_paths, path_from_params,
    retention_conditions_from_effective, stats_for_paths, zero_seqkit_metrics,
};
use crate::metrics::filters::{filter_metrics_with_removals, FilterRemovalCounts};

pub(super) fn stage_metrics_for_stage(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Option<Result<serde_json::Value>> {
    match plan.stage_id.as_str() {
        id_catalog::FASTQ_TRIM => Some(trim_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_FILTER => Some(filter_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_DEDUPLICATE => Some(deduplicate_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_LOW_COMPLEXITY => Some(low_complexity_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_MERGE => Some(merge_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_VALIDATE_PRE => Some(validate_metrics(plan, inputs)),
        _ => None,
    }
}

fn trim_metrics(
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
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let read_retention = if input.reads > 0 {
        f64_from_u64(output.reads) / f64_from_u64(input.reads)
    } else {
        0.0
    };
    let base_retention = if input.bases > 0 {
        f64_from_u64(output.bases) / f64_from_u64(input.bases)
    } else {
        0.0
    };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: output.mean_q - input.mean_q,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: output.reads,
        denominator_reads: input.reads,
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
    Ok(serde_json::to_value(FastqTrimMetricsV1 {
        reads_in: input.reads,
        reads_out: output.reads,
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        delta_metrics: delta,
        paired_mode: None,
        adapter_policy: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
        raw_backend_report_format: None,
        retention,
    })?)
}

fn filter_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("filter_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_filter_reads_report(&raw).ok());
    if let Some(report) = governed_report {
        let read_retention = if report.reads_in > 0 {
            f64_from_u64(report.reads_out) / f64_from_u64(report.reads_in)
        } else {
            0.0
        };
        let base_retention = if report.bases_in > 0 {
            f64_from_u64(report.bases_out) / f64_from_u64(report.bases_in)
        } else {
            0.0
        };
        let delta = FastqDeltaMetricsV1 {
            read_retention,
            base_retention,
            mean_q_delta: report.mean_q_after - report.mean_q_before,
            gc_delta: 0.0,
        };
        let retention = RetentionReportMetricV1 {
            value: read_retention,
            numerator_reads: report.reads_out,
            denominator_reads: report.reads_in,
            numerator_bases: report.bases_out,
            denominator_bases: report.bases_in,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: plan.stage_id.to_string(),
            conditions: retention_conditions_from_effective(
                &plan.stage_id,
                &plan.effective_params,
                &plan.params,
            ),
        };
        Ok(serde_json::to_value(FastqFilterMetricsV1 {
            reads_in: report.reads_in,
            reads_out: report.reads_out,
            reads_dropped: report.reads_dropped,
            reads_removed_by_n: report.reads_removed_by_n,
            reads_removed_by_entropy: report.reads_removed_by_entropy,
            reads_removed_low_complexity: report.reads_removed_low_complexity,
            reads_removed_by_kmer: report.reads_removed_by_kmer,
            reads_removed_contaminant_kmer: report.reads_removed_contaminant_kmer,
            reads_removed_by_length: report.reads_removed_by_length,
            bases_in: report.bases_in,
            bases_out: report.bases_out,
            pairs_in: report.pairs_in,
            pairs_out: report.pairs_out,
            mean_q_before: report.mean_q_before,
            mean_q_after: report.mean_q_after,
            delta_metrics: delta,
            retention,
        })?)
    } else {
        let removals = filter_removals_for_plan(plan.tool_id.as_str(), &plan.out_dir, &plan.params);
        filter_metrics_with_removals(
            &plan.stage_id,
            inputs,
            outputs,
            &plan.params,
            &plan.effective_params,
            &removals,
        )
    }
}

fn deduplicate_metrics(
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
    let reads_in = parsed_report
        .as_ref()
        .map(|report| report.reads_in)
        .unwrap_or(input.reads);
    let reads_out = parsed_report
        .as_ref()
        .map(|report| report.reads_out)
        .unwrap_or(output.reads);
    let (pairs_in, pairs_out) = (
        parsed_report.as_ref().and_then(|report| report.pairs_in),
        parsed_report.as_ref().and_then(|report| report.pairs_out),
    );
    let read_retention = if reads_in > 0 {
        f64_from_u64(reads_out) / f64_from_u64(reads_in)
    } else {
        0.0
    };
    let base_retention = if input.bases > 0 {
        f64_from_u64(output.bases) / f64_from_u64(input.bases)
    } else {
        0.0
    };
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
        dedup_mode: parsed_report
            .as_ref()
            .map(|report| match &report.dedup_mode {
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
        pair_count_match: parsed_report
            .as_ref()
            .and_then(|report| report.pair_count_match),
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

fn low_complexity_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("low_complexity_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_filter_low_complexity_report(&raw).ok());
    if let Some(report) = parsed_report {
        let read_retention = if report.reads_in > 0 {
            f64_from_u64(report.reads_out) / f64_from_u64(report.reads_in)
        } else {
            0.0
        };
        let base_retention = if report.bases_in > 0 {
            f64_from_u64(report.bases_out) / f64_from_u64(report.bases_in)
        } else {
            0.0
        };
        let delta = FastqDeltaMetricsV1 {
            read_retention,
            base_retention,
            mean_q_delta: report.mean_q_after - report.mean_q_before,
            gc_delta: 0.0,
        };
        let retention = RetentionReportMetricV1 {
            value: read_retention,
            numerator_reads: report.reads_out,
            denominator_reads: report.reads_in,
            numerator_bases: report.bases_out,
            denominator_bases: report.bases_in,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: plan.stage_id.to_string(),
            conditions: retention_conditions_from_effective(
                &plan.stage_id,
                &plan.effective_params,
                &plan.params,
            ),
        };
        Ok(serde_json::to_value(FastqFilterMetricsV1 {
            reads_in: report.reads_in,
            reads_out: report.reads_out,
            reads_dropped: report.reads_removed_low_complexity,
            reads_removed_by_n: 0,
            reads_removed_by_entropy: 0,
            reads_removed_low_complexity: report.reads_removed_low_complexity,
            reads_removed_by_kmer: 0,
            reads_removed_contaminant_kmer: 0,
            reads_removed_by_length: 0,
            bases_in: report.bases_in,
            bases_out: report.bases_out,
            pairs_in: report.pairs_in,
            pairs_out: report.pairs_out,
            mean_q_before: report.mean_q_before,
            mean_q_after: report.mean_q_after,
            delta_metrics: delta,
            retention,
        })?)
    } else {
        let mut removals = FilterRemovalCounts::default();
        let stats = stats_for_paths(&[
            inputs.first().map(PathBuf::as_path),
            outputs.first().map(PathBuf::as_path),
        ])?;
        let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
        let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
        removals.by_low_complexity =
            std::fs::read_to_string(plan.out_dir.join("low_complexity_report.json"))
                .ok()
                .and_then(|raw| crate::observer::parse_low_complexity_report(&raw).ok())
                .unwrap_or_else(|| input.reads.saturating_sub(output.reads));
        filter_metrics_with_removals(
            &plan.stage_id,
            inputs,
            outputs,
            &plan.params,
            &plan.effective_params,
            &removals,
        )
    }
}

fn merge_metrics(
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
    let reads_r1 = parsed_report
        .as_ref()
        .map(|report| report.reads_r1)
        .unwrap_or(r1.reads);
    let reads_r2 = parsed_report
        .as_ref()
        .map(|report| report.reads_r2)
        .unwrap_or(r2.reads);
    let reads_merged = parsed_report
        .as_ref()
        .map(|report| report.reads_merged)
        .unwrap_or(merged.reads);
    let reads_unmerged = parsed_report
        .as_ref()
        .map(|report| report.reads_unmerged)
        .unwrap_or_else(|| unmerged_r1.reads.min(unmerged_r2.reads));
    let min_reads = reads_r1.min(reads_r2);
    let merge_rate = parsed_report
        .as_ref()
        .map(|report| report.merge_rate)
        .unwrap_or_else(|| {
            if min_reads > 0 {
                f64_from_u64(reads_merged) / f64_from_u64(min_reads)
            } else {
                0.0
            }
        });
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

fn validate_metrics(plan: &StagePlanV1, inputs: &[PathBuf]) -> Result<serde_json::Value> {
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
                bijux_dna_domain_fastq::ValidateFailureClass::None => 0,
                bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => report
                    .validated_reads_r1
                    .abs_diff(report.validated_reads_r2.unwrap_or(0)),
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
                bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => 0,
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
    Ok(serde_json::to_value(report_metrics.unwrap_or(
        FastqValidateMetricsV1 {
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
        },
    ))?)
}
