use std::path::{Path, PathBuf};

use bijux_dna_domain_bam::metrics::BamMetricsV1;

#[allow(clippy::too_many_lines)]
pub fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
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
        if let Ok(counts) = bijux_dna_domain_bam::metrics::parse_samtools_flagstat(&path) {
            metrics.alignment = counts;
        }
    }

    let stats_path = first_existing(out_dir, &["samtools_stats.txt"]);
    if let Some(path) = stats_path {
        if let Ok((fragment, mapq)) = bijux_dna_domain_bam::metrics::parse_samtools_stats(&path) {
            metrics.fragment_length = fragment;
            metrics.mapq = mapq;
        }
    }
    let idxstats_path = first_existing(out_dir, &["idxstats.after.txt", "idxstats.txt"]);
    if let Some(path) = idxstats_path {
        if let Ok(idxstats) = bijux_dna_domain_bam::metrics::parse_samtools_idxstats(&path) {
            metrics.idxstats = idxstats;
        }
    }

    let mosdepth_path = first_existing(
        out_dir,
        &["coverage.mosdepth.summary.txt", "mosdepth.summary.txt"],
    );
    if let Some(path) = mosdepth_path {
        if let Ok(coverage) = bijux_dna_domain_bam::metrics::parse_mosdepth_summary(&path) {
            metrics.coverage = coverage;
        }
    } else {
        let depth_path = first_existing(out_dir, &["coverage.depth.txt", "depth.txt"]);
        if let Some(path) = depth_path {
            if let Ok((coverage, uniformity)) =
                bijux_dna_domain_bam::metrics::parse_samtools_depth_with_uniformity(&path)
            {
                metrics.coverage = coverage;
                metrics.coverage_uniformity = uniformity;
            }
        }
    }

    let preseq_path = first_existing(out_dir, &["preseq.txt"]);
    if let Some(path) = preseq_path {
        if let Ok(complexity) = bijux_dna_domain_bam::metrics::parse_preseq_estimates(&path) {
            metrics.complexity = complexity;
        }
    }

    let mut damage_sources: Vec<(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)> =
        Vec::new();
    let pydamage_path = first_existing(out_dir, &["damage.pydamage.json", "pydamage.json"]);
    if let Some(path) = pydamage_path {
        if let Ok(damage) = bijux_dna_domain_bam::metrics::parse_pydamage_json(&path) {
            metrics.damage = damage.clone();
            damage_sources.push(("pydamage".to_string(), damage));
        }
    }
    let mapdamage2_path = first_existing(out_dir, &["damage.mapdamage2.txt", "mapdamage2.txt"]);
    if let Some(path) = mapdamage2_path {
        if let Ok(damage) = bijux_dna_domain_bam::metrics::parse_mapdamage2_misincorporation(&path)
        {
            if damage_sources.is_empty() {
                metrics.damage = damage.clone();
            }
            damage_sources.push(("mapdamage2".to_string(), damage));
        }
    }
    let damageprofiler_path =
        first_existing(out_dir, &["damage.profiler.json", "damageprofiler.json"]);
    if let Some(path) = damageprofiler_path {
        if let Ok(damage) = bijux_dna_domain_bam::metrics::parse_damageprofiler_json(&path) {
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
        metrics.damage_comparison = Some(bijux_dna_domain_bam::metrics::compare_damage_metrics(
            tool_a, metrics_a, tool_b, metrics_b, threshold,
        ));
    }

    let contamination_path = first_existing(out_dir, &["contamination.json"]);
    if let Some(path) = contamination_path {
        if let Ok(contamination) = bijux_dna_domain_bam::metrics::parse_contamination_json(&path) {
            metrics.contamination = contamination;
        }
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                metrics.contamination_reconciliation.mt_fraction =
                    value.get("mt_estimate").and_then(serde_json::Value::as_f64);
                metrics.contamination_reconciliation.nuclear_fraction = value
                    .get("nuclear_estimate")
                    .and_then(serde_json::Value::as_f64);
            }
        }
    }

    let sex_path = first_existing(out_dir, &["sex.json"]);
    if let Some(path) = sex_path {
        if let Ok(sex) = bijux_dna_domain_bam::metrics::parse_sex_json(&path) {
            metrics.sex = sex;
        }
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
