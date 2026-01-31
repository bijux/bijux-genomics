use std::collections::BTreeMap;

use anyhow::Result;

use crate::aggregate::{
    BenchmarkRecord, FastqCorrectMetrics, FastqFilterMetrics, FastqMergeMetrics, FastqTrimMetrics,
    FastqUmiMetrics, FastqValidateMetrics,
};
use crate::decision::score::{build_rankings, RankInput, RankingEntry};

use super::math::ratio_u64;

/// Rank trim tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_trim_tools(
    records: &[BenchmarkRecord<FastqTrimMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.delta_metrics.read_retention),
            base_retention: Some(record.metrics.metrics.delta_metrics.base_retention),
            error_reduction_proxy: None,
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank validate tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_validate_tools(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| {
            let retention = ratio_u64(
                record.metrics.metrics.reads_valid,
                record.metrics.metrics.reads_total,
            );
            RankInput {
                tool: record.context.tool.clone(),
                runtime_s: record.execution.runtime_s,
                memory_mb: record.execution.memory_mb,
                read_retention: Some(retention),
                base_retention: None,
                error_reduction_proxy: None,
            }
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank filter tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_filter_tools(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(record.metrics.metrics.delta_metrics.read_retention),
            base_retention: Some(record.metrics.metrics.delta_metrics.base_retention),
            error_reduction_proxy: None,
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank merge tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_merge_tools(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: Some(record.metrics.metrics.merge_rate),
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank correct tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_correct_tools(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: None,
            base_retention: None,
            error_reduction_proxy: Some(record.metrics.metrics.kmer_fix_rate),
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}

/// Rank UMI tools by metrics and execution stats.
///
/// # Errors
/// Returns an error if ranking computation fails.
pub fn rank_umi_tools(
    records: &[BenchmarkRecord<FastqUmiMetrics>],
) -> Result<BTreeMap<String, Vec<RankingEntry>>> {
    let inputs = records
        .iter()
        .map(|record| RankInput {
            tool: record.context.tool.clone(),
            runtime_s: record.execution.runtime_s,
            memory_mb: record.execution.memory_mb,
            read_retention: Some(ratio_u64(
                record.metrics.metrics.reads_out,
                record.metrics.metrics.reads_in,
            )),
            base_retention: None,
            error_reduction_proxy: None,
        })
        .collect::<Vec<_>>();
    build_rankings(&inputs)
}
