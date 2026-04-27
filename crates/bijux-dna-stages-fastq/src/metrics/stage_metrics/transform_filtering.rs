use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_fastq::metrics::{
    FastqDeltaMetricsV1, FastqFilterMetricsV1, FastqTrimMetricsV1, RetentionReportMetricV1,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    f64_from_u64, filter_removals_for_plan, pair_counts_from_paths, path_from_params,
    retention_conditions_from_effective, stats_for_paths, zero_seqkit_metrics,
};
use crate::metrics::filters::{filter_metrics_with_removals, FilterRemovalCounts};

pub(super) fn trim_metrics(
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
    let read_retention =
        if input.reads > 0 { f64_from_u64(output.reads) / f64_from_u64(input.reads) } else { 0.0 };
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

pub(super) fn filter_metrics(
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

pub(super) fn low_complexity_metrics(
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
        removals.low_complexity =
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
