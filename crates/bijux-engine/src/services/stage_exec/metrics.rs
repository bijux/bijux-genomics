#[allow(clippy::too_many_lines)]
fn stage_metrics_for_plan(
    stage_id: &str,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
    params: &serde_json::Value,
    effective_params: &serde_json::Value,
) -> Result<serde_json::Value> {
    let mut metrics = match stage_id {
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
            let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
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
                pairs_in,
                pairs_out,
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
        stage_id if stage_id.starts_with("bam.") => {
            let out_dir = outputs
                .first()
                .and_then(|path| path.parent())
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let mut metrics = bam_metrics_from_dir(&out_dir);
            let thresholds = bijux_domain_bam::metrics::BamInvariantThresholds::default();
            let evaluation = bijux_domain_bam::metrics::evaluate_bam_invariants(stage_id, &metrics, &thresholds);
            metrics.stage_verdict = Some(evaluation.verdict.into());
            serde_json::to_value(metrics)?
        }
        _ => serde_json::json!({}),
    };
    if stage_id.starts_with("fastq.") {
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

#[allow(clippy::too_many_lines)]
fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();

    let flagstat_path = first_existing(
        out_dir,
        &["filter.flagstat.txt", "markdup.flagstat.txt", "flagstat.txt"],
    );
    if let Some(path) = flagstat_path {
        if let Ok(counts) = parse_samtools_flagstat(&path) {
            metrics.alignment = counts;
        }
    }

    let stats_path = first_existing(out_dir, &["samtools_stats.txt"]);
    if let Some(path) = stats_path {
        if let Ok((fragment, mapq)) = parse_samtools_stats(&path) {
            metrics.fragment_length = fragment;
            metrics.mapq = mapq;
        }
    }
    let idxstats_path = first_existing(out_dir, &["idxstats.txt"]);
    if let Some(path) = idxstats_path {
        if let Ok(idxstats) = crate::services::observer::parse_samtools_idxstats(&path) {
            metrics.idxstats = idxstats;
        }
    }

    let mosdepth_path =
        first_existing(out_dir, &["coverage.mosdepth.summary.txt", "mosdepth.summary.txt"]);
    if let Some(path) = mosdepth_path {
        if let Ok(coverage) = parse_mosdepth_summary(&path) {
            metrics.coverage = coverage;
        }
    } else {
        let depth_path = first_existing(out_dir, &["coverage.depth.txt", "depth.txt"]);
        if let Some(path) = depth_path {
            if let Ok(coverage) = bijux_domain_bam::metrics::parse_samtools_depth(&path) {
                metrics.coverage = coverage;
            }
        }
    }

    let preseq_path = first_existing(out_dir, &["preseq.txt"]);
    if let Some(path) = preseq_path {
        if let Ok(complexity) = parse_preseq_estimates(&path) {
            metrics.complexity = complexity;
        }
    }

    let mut damage_sources: Vec<(String, bijux_domain_bam::metrics::DamageMetricsV1)> = Vec::new();
    let pydamage_path = first_existing(out_dir, &["damage.pydamage.json", "pydamage.json"]);
    if let Some(path) = pydamage_path {
        if let Ok(damage) = parse_pydamage_json(&path) {
            metrics.damage = damage.clone();
            damage_sources.push(("pydamage".to_string(), damage));
        }
    }
    let mapdamage2_path = first_existing(out_dir, &["damage.mapdamage2.txt", "mapdamage2.txt"]);
    if let Some(path) = mapdamage2_path {
        if let Ok(damage) = bijux_domain_bam::parse_mapdamage2_misincorporation(&path) {
            if damage_sources.is_empty() {
                metrics.damage = damage.clone();
            }
            damage_sources.push(("mapdamage2".to_string(), damage));
        }
    }
    let damageprofiler_path =
        first_existing(out_dir, &["damage.profiler.json", "damageprofiler.json"]);
    if let Some(path) = damageprofiler_path {
        if let Ok(damage) = parse_damageprofiler_json(&path) {
            if damage_sources.is_empty() {
                metrics.damage = damage.clone();
            }
            damage_sources.push(("damageprofiler".to_string(), damage));
        }
    }
    if damage_sources.len() >= 2 {
        let threshold = 0.05;
        let (tool_a, metrics_a) = &damage_sources[0];
        let (tool_b, metrics_b) = &damage_sources[1];
        metrics.damage_comparison = Some(bijux_domain_bam::metrics::compare_damage_metrics(
            tool_a,
            metrics_a,
            tool_b,
            metrics_b,
            threshold,
        ));
    }

    let contamination_path = first_existing(out_dir, &["contamination.json"]);
    if let Some(path) = contamination_path {
        if let Ok(contamination) = parse_contamination_json(&path) {
            metrics.contamination = contamination;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                metrics.contamination_reconciliation.mt_fraction = value
                    .get("mt_estimate")
                    .and_then(serde_json::Value::as_f64);
                metrics.contamination_reconciliation.nuclear_fraction = value
                    .get("nuclear_estimate")
                    .and_then(serde_json::Value::as_f64);
            }
        }
    }

    let sex_path = first_existing(out_dir, &["sex.json"]);
    if let Some(path) = sex_path {
        if let Ok(sex) = parse_sex_json(&path) {
            metrics.sex = sex;
        }
    }

    if metrics.coverage.mean > 0.0 {
        metrics.effective_coverage.raw = metrics.coverage.mean;
        let dup_fraction = if metrics.alignment.total > 0 {
            u64_to_f64(metrics.alignment.duplicates) / u64_to_f64(metrics.alignment.total)
        } else {
            0.0
        };
        metrics.effective_coverage.dedup = metrics.coverage.mean * (1.0 - dup_fraction);
        let damage = metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p);
        let pmd_retention = if damage >= 0.10 { 0.8 } else { 0.5 };
        metrics.effective_coverage.pmd_filtered = metrics.coverage.mean * pmd_retention;
        metrics.coverage_uniformity.dropout_fraction =
            (1.0 - metrics.coverage.breadth_1x).clamp(0.0, 1.0);
        metrics.coverage_uniformity.coefficient_of_variation =
            (1.0 - metrics.coverage.breadth_1x).max(0.0);
        let sufficient = metrics.coverage.mean >= 1.0 || metrics.coverage.breadth_1x >= 0.1;
        let reason = if sufficient {
            "coverage meets minimum thresholds"
        } else {
            "coverage below minimum thresholds"
        };
        metrics.coverage_sufficiency.sufficient = sufficient;
        metrics.coverage_sufficiency.mean_coverage = metrics.coverage.mean;
        metrics.coverage_sufficiency.breadth_1x = metrics.coverage.breadth_1x;
        metrics.coverage_sufficiency.reason = reason.to_string();
    }

    if metrics.coverage_sufficiency.sufficient {
        metrics.sex_sufficiency.sufficient = metrics.sex.sufficient_data;
        metrics.sex_sufficiency.confidence = metrics.sex.confidence;
        metrics.sex_sufficiency.reason = if metrics.sex.sufficient_data {
            "sex inference meets thresholds".to_string()
        } else {
            "sex inference confidence below threshold".to_string()
        };
        metrics.contamination_sufficiency.sufficient = metrics.contamination.estimate > 0.0;
        metrics.contamination_sufficiency.estimate = metrics.contamination.estimate;
        metrics.contamination_sufficiency.reason = if metrics.contamination.estimate > 0.0 {
            "contamination estimate available".to_string()
        } else {
            "contamination estimate missing".to_string()
        };
    } else {
        let reason = metrics.coverage_sufficiency.reason.clone();
        metrics.sex_sufficiency.sufficient = false;
        metrics.sex_sufficiency.confidence = metrics.sex.confidence;
        metrics.sex_sufficiency.reason.clone_from(&reason);
        metrics.contamination_sufficiency.sufficient = false;
        metrics.contamination_sufficiency.estimate = metrics.contamination.estimate;
        metrics.contamination_sufficiency
            .reason
            .clone_from(&reason);
        metrics.haplogroup_sufficiency.sufficient = false;
        metrics.haplogroup_sufficiency.min_coverage = metrics.coverage.mean;
        metrics.haplogroup_sufficiency
            .reason
            .clone_from(&reason);
        metrics.kinship_sufficiency.sufficient = false;
        metrics.kinship_sufficiency.reason = reason;
        metrics.sex.classification = bijux_domain_bam::metrics::SexConfidenceClass::Insufficient;
        metrics.sex.sufficient_data = false;
    }

    let authenticity = bijux_domain_bam::metrics::authenticity_score(&metrics);
    metrics.authenticity = authenticity;
    metrics.contamination_reconciliation.assessment =
        bijux_domain_bam::metrics::contamination_cross_check(
            metrics.damage.c_to_t_5p.max(metrics.damage.g_to_a_3p),
            metrics.contamination.estimate,
        );
    if let (Some(mt), Some(nuclear)) = (
        metrics.contamination_reconciliation.mt_fraction,
        metrics.contamination_reconciliation.nuclear_fraction,
    ) {
        if (mt - nuclear).abs() >= 0.1 {
            metrics.contamination_reconciliation.assessment =
                "mtDNA vs nuclear contamination estimates diverge".to_string();
        }
    }
    metrics.sex_sufficiency.sufficient = metrics.sex.sufficient_data;
    metrics.sex_sufficiency.confidence = metrics.sex.confidence;
    metrics.sex_sufficiency.reason = if metrics.sex.sufficient_data {
        "sex inference sufficient"
    } else {
        "insufficient sex data"
    }
    .to_string();
    metrics.contamination_sufficiency.sufficient = metrics.contamination.estimate > 0.0;
    metrics.contamination_sufficiency.estimate = metrics.contamination.estimate;
    metrics.contamination_sufficiency.reason = if metrics.contamination.estimate > 0.0 {
        "contamination estimate available"
    } else {
        "contamination estimate unavailable"
    }
    .to_string();
    if metrics.coverage.mean >= 1.0 {
        metrics.haplogroup_sufficiency.sufficient = true;
        metrics.haplogroup_sufficiency.min_coverage = metrics.coverage.mean;
        metrics.haplogroup_sufficiency.reason = "coverage meets minimum threshold".to_string();
        metrics.kinship_sufficiency.sufficient = true;
        metrics.kinship_sufficiency.overlap_snps = 1000;
        metrics.kinship_sufficiency.reason = "coverage likely sufficient for kinship".to_string();
    } else {
        metrics.haplogroup_sufficiency.sufficient = false;
        metrics.haplogroup_sufficiency.min_coverage = metrics.coverage.mean;
        metrics.haplogroup_sufficiency.reason =
            "coverage below threshold for haplogroup assignment".to_string();
        metrics.kinship_sufficiency.sufficient = false;
        metrics.kinship_sufficiency.overlap_snps = 0;
        metrics.kinship_sufficiency.reason =
            "coverage below threshold for kinship inference".to_string();
    }

    metrics
}

fn first_existing(out_dir: &Path, names: &[&str]) -> Option<PathBuf> {
    for name in names {
        let candidate = out_dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
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
