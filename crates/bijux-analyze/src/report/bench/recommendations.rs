use bijux_core::primitives::RawFailure;
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

pub fn sanity_flags_trim(records: &[BenchmarkRecord<FastqTrimMetrics>]) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| record.metrics.metrics.delta_metrics.read_retention)
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "Trim retains less than 85% of reads; check adapter or quality thresholds.",
        }));
    }
    flags
}

pub fn sanity_flags_filter(
    records: &[BenchmarkRecord<FastqFilterMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| record.metrics.metrics.delta_metrics.read_retention)
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "Filter retains less than 85% of reads; check filtering thresholds.",
        }));
    }
    flags
}

pub fn sanity_flags_correct(
    records: &[BenchmarkRecord<FastqCorrectMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let rates = records
        .iter()
        .map(|record| record.metrics.metrics.kmer_fix_rate)
        .collect::<Vec<_>>();
    let median_rate = median(rates);
    if median_rate < 0.2 {
        flags.push(serde_json::json!({
            "flag": "low_kmer_fix_rate",
            "severity": "warning",
            "message": "Correct fixes fewer than 20% of k-mer errors; check k-mer parameters.",
        }));
    }
    flags
}

pub fn sanity_flags_umi(records: &[BenchmarkRecord<FastqUmiMetrics>]) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| {
            ratio_u64(
                record.metrics.metrics.reads_out,
                record.metrics.metrics.reads_in,
            )
        })
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.85 {
        flags.push(serde_json::json!({
            "flag": "low_read_retention",
            "severity": "warning",
            "message": "UMI processing retains less than 85% of reads; verify UMI parameters.",
        }));
    }
    flags
}

pub fn sanity_flags_merge(
    records: &[BenchmarkRecord<FastqMergeMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let merge_rates = records
        .iter()
        .map(|record| record.metrics.metrics.merge_rate)
        .collect::<Vec<_>>();
    let median_merge = median(merge_rates);
    if median_merge < 0.2 {
        flags.push(serde_json::json!({
            "flag": "low_merge_rate",
            "severity": "warning",
            "message": "Merge rate below 20%; check insert size vs read length.",
        }));
    }
    flags
}

pub fn sanity_flags_stats(
    records: &[BenchmarkRecord<FastqStatsMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let gc = records
        .iter()
        .map(|record| record.metrics.metrics.gc_percent)
        .collect::<Vec<_>>();
    let median_gc = median(gc);
    if !(20.0..=80.0).contains(&median_gc) {
        flags.push(serde_json::json!({
            "flag": "gc_out_of_range",
            "severity": "warning",
            "message": "Median GC% is outside expected range; check sample composition.",
        }));
    }
    flags
}

pub fn sanity_flags_validate(
    records: &[BenchmarkRecord<FastqValidateMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let retention = records
        .iter()
        .map(|record| {
            ratio_u64(
                record.metrics.metrics.reads_valid,
                record.metrics.metrics.reads_total,
            )
        })
        .collect::<Vec<_>>();
    let median_retention = median(retention);
    if median_retention < 0.9 {
        flags.push(serde_json::json!({
            "flag": "low_validation_pass_rate",
            "severity": "warning",
            "message": "More than 10% of reads are invalid; check read integrity.",
        }));
    }
    flags
}

pub fn sanity_flags_qc_post(
    records: &[BenchmarkRecord<FastqQcPostMetrics>],
) -> Vec<serde_json::Value> {
    let mut flags = Vec::new();
    let contamination = records
        .iter()
        .map(|record| record.metrics.metrics.contamination_rate)
        .collect::<Vec<_>>();
    let median_contamination = median(contamination);
    if median_contamination > 0.05 {
        flags.push(serde_json::json!({
            "flag": "high_contamination",
            "severity": "warning",
            "message": "Contamination rate > 5%; check contaminant panel or sample prep.",
        }));
    }
    flags
}

pub fn derived_trim_metrics(record: &BenchmarkRecord<FastqTrimMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    serde_json::json!({
        "read_retention": delta.read_retention,
        "base_retention": delta.base_retention,
        "mean_q_delta": delta.mean_q_delta,
        "gc_delta": delta.gc_delta,
    })
}

pub fn derived_filter_metrics(record: &BenchmarkRecord<FastqFilterMetrics>) -> serde_json::Value {
    let delta = &record.metrics.metrics.delta_metrics;
    serde_json::json!({
        "read_retention": delta.read_retention,
        "base_retention": delta.base_retention,
        "mean_q_delta": delta.mean_q_delta,
        "gc_delta": delta.gc_delta,
    })
}

pub fn derived_merge_metrics(record: &BenchmarkRecord<FastqMergeMetrics>) -> serde_json::Value {
    serde_json::json!({
        "merge_rate": record.metrics.metrics.merge_rate,
        "reads_merged": record.metrics.metrics.reads_merged,
        "reads_unmerged": record.metrics.metrics.reads_unmerged,
    })
}

pub fn derived_correct_metrics(record: &BenchmarkRecord<FastqCorrectMetrics>) -> serde_json::Value {
    serde_json::json!({
        "kmer_fix_rate": record.metrics.metrics.kmer_fix_rate,
    })
}

pub fn derived_umi_metrics(record: &BenchmarkRecord<FastqUmiMetrics>) -> serde_json::Value {
    serde_json::json!({
        "read_retention": ratio_u64(
            record.metrics.metrics.reads_out,
            record.metrics.metrics.reads_in,
        ),
    })
}

#[must_use]
pub fn derived_metrics_for_stage_json(stage: &str) -> Vec<serde_json::Value> {
    let mut derived = Vec::new();
    for metric in derived_metrics_for_stage(stage) {
        let spec = derived_metric_spec(metric.id);
        let derived_metric = match metric.id {
            DerivedMetricId::ReadRetention => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::BaseRetention => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::MergeEfficiency => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
            DerivedMetricId::ErrorReductionProxy => serde_json::json!({
                "id": spec.name,
                "meaning": spec.meaning,
                "direction": format!("{:?}", spec.direction),
                "range": spec.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max,
                })),
            }),
        };
        derived.push(derived_metric);
    }
    derived
}

/// Print the benchmark schema for a stage.
///
/// # Errors
/// Returns an error if the schema cannot be rendered.
pub fn print_bench_schema(stage: &str) -> Result<()> {
    let json = bench_schema_json(stage)?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

/// Build the benchmark schema as JSON for a stage.
///
/// # Errors
/// Returns an error if the stage is unknown or serialization fails.
pub fn bench_schema_json(stage: &str) -> Result<serde_json::Value> {
    let kind = metric_kind_for_stage(stage).ok_or_else(|| anyhow!("unknown stage {stage}"))?;
    let spec = stage_metric_spec(kind);
    let metrics: Vec<_> = spec
        .metrics
        .iter()
        .map(|metric_id| {
            let metric = metric_spec(*metric_id);
            serde_json::json!({
                "name": metric.name,
                "meaning": metric.meaning,
                "direction": format!("{:?}", metric.direction),
                "range": metric.range.map(|range| serde_json::json!({
                    "min": range.min,
                    "max": range.max
                })),
                "measured": metric.measured,
                "derived": metric.derived,
            })
        })
        .collect();
    let derived: Vec<_> = spec
        .metrics
        .iter()
        .filter_map(|metric_id| {
            let metric = metric_spec(*metric_id);
            if metric.derived {
                Some(metric.name.to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(serde_json::json!({
        "stage": stage,
        "schema_version": format!("{}_v{}", stage.replace('.', "_"), spec.version),
        "metrics": metrics,
        "derived": derived,
        "invariants": spec.invariants,
    }))
}

/// Write the trim benchmark report.
///
/// # Errors
/// Returns an error if report serialization or file writes fail.
pub fn write_trim_report(
    base_dir: &Path,
    records: &[BenchmarkRecord<FastqTrimMetrics>],
    failures: &[RawFailure],
    explain: bool,
) -> Result<()> {
    let path = base_dir.join("report.json");
    let mut report = BTreeMap::new();
    report.insert("records", serde_json::to_value(records)?);
    let classified: Vec<BenchmarkFailure> = failures.iter().map(classify_raw_failure).collect();
    report.insert("failures", serde_json::to_value(&classified)?);
    report.insert("gate", gate_payload(&classified));
    report.insert(
        "sanity_flags",
        serde_json::to_value(sanity_flags_trim(records))?,
    );
    let derived: Vec<_> = records.iter().map(derived_trim_metrics).collect();
    report.insert("derived_metrics", serde_json::to_value(&derived)?);
    let semantic: Vec<_> = records
        .iter()
        .map(|record| semantic_trim(&record.metrics.metrics))
        .collect();
    report.insert("semantic_metrics", serde_json::to_value(&semantic)?);
    let rankings = rank_trim_tools(records)?;
    report.insert("rankings", serde_json::to_value(&rankings)?);
    let json = serde_json::to_string_pretty(&report)?;
    atomic_write_bytes(&path, json.as_bytes())
        .map_err(anyhow::Error::from)
        .context("write report.json")?;
    if explain {
        crate::decision::score::print_rank_explain("fastq.trim", &rankings);
    }
    Ok(())
}
