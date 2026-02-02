#[allow(clippy::too_many_lines)]
fn stage_metrics_for_plan(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let metrics = match stage_id {
        "fastq.trim" => {
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
                stage_boundary: stage_id.to_string(),
                conditions: retention_conditions_from_effective(stage_id, effective_params, params),
            };
            serde_json::to_value(FastqTrimMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q_before: input.mean_q,
                mean_q_after: output.mean_q,
                delta_metrics: delta,
                retention,
            })?
        }
        "fastq.filter" => filter_metrics_with_removals(
            stage_id,
            inputs,
            outputs,
            params,
            effective_params,
            &FilterRemovalCounts::default(),
        )?,
        "fastq.merge" => {
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
            let reads_unmerged = unmerged_r1.reads.min(unmerged_r2.reads);
            let min_reads = r1.reads.min(r2.reads);
            let merge_rate = if min_reads > 0 {
                f64_from_u64(merged.reads) / f64_from_u64(min_reads)
            } else {
                0.0
            };
            let bases_in = r1.bases.min(r2.bases);
            serde_json::to_value(FastqMergeMetricsV1 {
                reads_in: min_reads,
                reads_out: merged.reads,
                bases_in,
                bases_out: merged.bases,
                pairs_in: min_reads,
                pairs_out: merged.reads,
                reads_r1: r1.reads,
                reads_r2: r2.reads,
                reads_merged: merged.reads,
                reads_unmerged,
                merge_rate,
            })?
        }
        "fastq.validate_pre" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqValidateMetricsV1 {
                reads_in: input.reads,
                reads_out: input.reads,
                bases_in: input.bases,
                bases_out: input.bases,
                pairs_in,
                pairs_out,
                reads_total: input.reads,
                reads_valid: input.reads,
                reads_invalid: 0,
                mean_q: input.mean_q,
            })?
        }
        "fastq.detect_adapters" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let out_dir = path_from_params(params, "out_dir")
                .unwrap_or_else(|| inputs.first().cloned().unwrap_or_default());
            let metrics = fastqc_metrics_v2_from_dir(&out_dir).or_else(|| {
                let subdir = out_dir.join("fastqc");
                fastqc_metrics_v2_from_dir(&subdir)
            });
            let adapter_content_max = metrics
                .as_ref()
                .and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent));
            let adapter_content_mean = metrics
                .as_ref()
                .and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent));
            let duplication_rate = metrics
                .as_ref()
                .and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate));
            let n_rate = metrics
                .as_ref()
                .and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0));
            let kmer_warning_count = metrics
                .as_ref()
                .and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count));
            serde_json::to_value(FastqDetectAdaptersMetricsV1 {
                reads_in: input.reads,
                reads_out: input.reads,
                bases_in: input.bases,
                bases_out: input.bases,
                mean_q: input.mean_q,
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
            })?
        }
        "fastq.correct" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqCorrectMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.umi" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqUmiMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.preprocess" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::to_value(FastqPreprocessMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
            })?
        }
        "fastq.qc_post" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(params, "out_dir")
                .unwrap_or_else(|| outputs.first().cloned().unwrap_or_default());
            let raw_dir = out_dir.join("fastqc_raw");
            let trimmed_dir = out_dir.join("fastqc_trimmed");
            let multiqc_report = out_dir.join("multiqc_report.html");
            let multiqc_data = out_dir.join("multiqc_data");
            let raw_metrics = fastqc_metrics_v2_from_dir(&raw_dir);
            let trimmed_metrics = fastqc_metrics_v2_from_dir(&trimmed_dir);
            let metrics_source = trimmed_metrics.as_ref().or(raw_metrics.as_ref());
            let adapter_content_max =
                metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent));
            let adapter_content_mean =
                metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent));
            let duplication_rate =
                metrics_source.and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate));
            let n_rate =
                metrics_source.and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0));
            let kmer_warning_count =
                metrics_source.and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count));
            serde_json::to_value(FastqQcPostMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q: input.mean_q,
                contamination_rate: 0.0,
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
                raw_fastqc_dir: raw_dir.exists().then_some(raw_dir.display().to_string()),
                trimmed_fastqc_dir: trimmed_dir
                    .exists()
                    .then_some(trimmed_dir.display().to_string()),
                multiqc_report: multiqc_report
                    .exists()
                    .then_some(multiqc_report.display().to_string()),
                multiqc_data: multiqc_data
                    .exists()
                    .then_some(multiqc_data.display().to_string()),
            })?
        }
        "fastq.screen" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let report_path = path_from_params(params, "report")
                .or_else(|| outputs.first().cloned())
                .unwrap_or_else(|| PathBuf::from("screen_report.tsv"));
            let (contamination_rate, contamination_summary) = parse_screen_report(&report_path)?;
            serde_json::json!({
                "reads_in": input.reads,
                "reads_out": output.reads,
                "bases_in": input.bases,
                "bases_out": output.bases,
                "pairs_in": pairs_in,
                "pairs_out": pairs_out,
                "contamination_rate": contamination_rate,
                "contamination_summary": contamination_summary,
            })
        }
        "fastq.stats_neutral" => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            serde_json::json!({
                "reads_in": input.reads,
                "reads_out": output.reads,
                "bases_in": input.bases,
                "bases_out": output.bases,
                "pairs_in": pairs_in,
                "pairs_out": pairs_out,
            })
        }
        _ => serde_json::json!({}),
    };
    Ok(metrics)
}

fn filter_metrics_with_removals(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
    removals: &FilterRemovalCounts,
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
        stage_boundary: stage_id.to_string(),
        conditions: retention_conditions_from_effective(stage_id, effective_params, params),
    };
    Ok(serde_json::to_value(FastqFilterMetricsV1 {
        reads_in: input.reads,
        reads_out: output.reads,
        reads_dropped: input.reads.saturating_sub(output.reads),
        reads_removed_by_n: removals.by_n,
        reads_removed_by_entropy: removals.by_entropy,
        reads_removed_low_complexity: removals.by_low_complexity,
        reads_removed_by_kmer: removals.by_kmer,
        reads_removed_contaminant_kmer: removals.by_contaminant_kmer,
        reads_removed_by_length: removals.by_length,
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        delta_metrics: delta,
        retention,
    })?)
}

fn retention_counts_for_plan(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<Option<RetentionCounts>> {
    let counts = match stage_id {
        "fastq.trim" | "fastq.filter" | "fastq.correct" | "fastq.umi" | "fastq.preprocess" => {
            let input = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let output = if outputs.is_empty() {
                input
            } else {
                stats_or_zero(outputs.first().map(PathBuf::as_path))?
            };
            RetentionCounts {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
            }
        }
        "fastq.merge" => {
            let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
            let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
            let merged = stats_or_zero(outputs.first().map(PathBuf::as_path))?;
            RetentionCounts {
                reads_in: r1.reads.min(r2.reads),
                reads_out: merged.reads,
                bases_in: r1.bases.min(r2.bases),
                bases_out: merged.bases,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some(counts))
}
