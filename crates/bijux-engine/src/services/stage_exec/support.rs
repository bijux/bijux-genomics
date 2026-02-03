// Consolidated helpers to keep stage_exec dir within module-count guardrails.

// --- metrics_bam.rs ---

#[allow(clippy::too_many_lines)]
pub(super) fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();

    let flagstat_path = first_existing(
        out_dir,
        &[
            "flagstat.after.txt",
            "filter.flagstat.txt",
            "markdup.flagstat.txt",
            "flagstat.txt",
        ],
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
    let idxstats_path = first_existing(out_dir, &["idxstats.after.txt", "idxstats.txt"]);
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
            if let Ok((coverage, uniformity)) =
                bijux_domain_bam::metrics::parse_samtools_depth_with_uniformity(&path)
            {
                metrics.coverage = coverage;
                metrics.coverage_uniformity = uniformity;
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
        if let Ok(damage) = bijux_domain_bam::metrics::parse_mapdamage2_misincorporation(&path) {
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
        metrics.contamination_sufficiency.reason = if metrics.contamination.estimate > 0.0 {
            "contamination estimate available".to_string()
        } else {
            "contamination estimate unavailable".to_string()
        };
    }

    metrics
}
