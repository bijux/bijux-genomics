pub fn stage_metrics_for_plan(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let mut metrics = match plan.stage_id.as_str() {
        id_catalog::FASTQ_TRIM => {
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
        id_catalog::FASTQ_FILTER => {
            let removals =
                filter_removals_for_plan(plan.tool_id.as_str(), &plan.out_dir, &plan.params);
            filter_metrics_with_removals(
                &plan.stage_id,
                inputs,
                outputs,
                &plan.params,
                &plan.effective_params,
                &removals,
            )?
        }
        id_catalog::FASTQ_DEDUPLICATE => {
            let stats = stats_for_paths(&[
                inputs.first().map(PathBuf::as_path),
                outputs.first().map(PathBuf::as_path),
            ])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
            let parsed_counts =
                std::fs::read_to_string(plan.out_dir.join("deduplicate_report.json"))
                    .ok()
                    .and_then(|raw| crate::observer::parse_deduplicate_report(&raw).ok());
            let (reads_in, reads_out) = parsed_counts.unwrap_or((input.reads, output.reads));
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
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
            serde_json::to_value(FastqDeduplicateMetricsV1 {
                reads_in,
                reads_out,
                reads_removed_duplicates: reads_in.saturating_sub(reads_out),
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
        id_catalog::FASTQ_LOW_COMPLEXITY => {
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
            )?
        }
        id_catalog::FASTQ_MERGE => {
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
            let mean_q_in = (r1.mean_q + r2.mean_q) / 2.0;
            let merge_q_delta = merged.mean_q - mean_q_in;
            serde_json::to_value(FastqMergeMetricsV1 {
                reads_in: min_reads,
                reads_out: merged.reads,
                bases_in,
                bases_out: merged.bases,
                pairs_in: Some(min_reads),
                pairs_out: Some(merged.reads),
                reads_r1: r1.reads,
                reads_r2: r2.reads,
                reads_merged: merged.reads,
                reads_unmerged,
                reads_discarded: 0,
                merge_rate,
                merge_q_delta,
            })?
        }
        id_catalog::FASTQ_VALIDATE_PRE => {
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
        id_catalog::FASTQ_DETECT_ADAPTERS => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(&plan.params, "out_dir")
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
            let overrepresented_sequence_count = metrics
                .as_ref()
                .and_then(|m| m.overrepresented_sequences.as_ref().map(|o| o.count));
            serde_json::to_value(FastqDetectAdaptersMetricsV1 {
                reads_in: input.reads,
                reads_out: input.reads,
                bases_in: input.bases,
                bases_out: input.bases,
                pairs_in,
                pairs_out,
                mean_q: input.mean_q,
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
                overrepresented_sequence_count,
            })?
        }
        id_catalog::FASTQ_CORRECT => {
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
                reads_corrected: output.reads,
                reads_uncorrected: input.reads.saturating_sub(output.reads),
                bases_corrected: output.bases,
                bases_uncorrected: input.bases.saturating_sub(output.bases),
            })?
        }
        id_catalog::FASTQ_UMI => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
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
            serde_json::to_value(FastqUmiMetricsV1 {
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
        id_catalog::FASTQ_PREPROCESS => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
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
            serde_json::to_value(FastqPreprocessMetricsV1 {
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
        id_catalog::FASTQ_QC_POST => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let out_dir = path_from_params(&plan.params, "out_dir")
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
            let overrepresented_sequence_count =
                metrics_source.and_then(|m| m.overrepresented_sequences.as_ref().map(|o| o.count));
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
            serde_json::to_value(FastqQcPostMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                mean_q: Some(output.mean_q),
                mean_q_before: input.mean_q,
                mean_q_after: output.mean_q,
                delta_metrics: delta,
                retention,
                contamination_rate: Some(0.0),
                adapter_content_max,
                adapter_content_mean,
                duplication_rate,
                n_rate,
                kmer_warning_count,
                overrepresented_sequence_count,
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
        id_catalog::FASTQ_STATS_NEUTRAL => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let (read_length_distribution, gc_distribution) =
                distributions_for_path(inputs.first().map(PathBuf::as_path))?;
            serde_json::to_value(FastqStatsNeutralMetricsV1 {
                reads_in: input.reads,
                reads_out: output.reads,
                bases_in: input.bases,
                bases_out: output.bases,
                pairs_in,
                pairs_out,
                read_length_distribution,
                gc_distribution,
            })?
        }
        id_catalog::FASTQ_SCREEN => {
            let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
            let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
            let output = if outputs.is_empty() {
                input
            } else {
                let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
                stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
            };
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
            let report_path = path_from_params(&plan.params, "report")
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
        _ => serde_json::json!({}),
    };
    if plan.stage_id.0.starts_with(id_catalog::FASTQ_PREFIX) {
        if let Some(obj) = metrics.as_object_mut() {
            if !obj.contains_key("pairs_in") || !obj.contains_key("pairs_out") {
                let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
                if !obj.contains_key("pairs_in") {
                    obj.insert("pairs_in".to_string(), serde_json::to_value(pairs_in)?);
                }
                if !obj.contains_key("pairs_out") {
                    obj.insert("pairs_out".to_string(), serde_json::to_value(pairs_out)?);
                }
            }
        }
    }
    Ok(metrics)
}
