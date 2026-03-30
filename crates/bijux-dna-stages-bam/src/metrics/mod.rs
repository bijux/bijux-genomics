use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

mod alignment;
mod coverage;
mod discovery;

#[must_use]
pub fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();
    alignment::parse_alignment_metrics(out_dir, &mut metrics);
    coverage::parse_coverage_metrics(out_dir, &mut metrics);
    parse_quality_metrics(out_dir, &mut metrics);
    parse_damage_metrics(out_dir, &mut metrics);
    parse_contamination_and_sex(out_dir, &mut metrics);
    metrics
}

fn parse_quality_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let preseq_path = discovery::first_existing(out_dir, &["preseq.txt"]);
    if let Some(path) = preseq_path {
        if let Ok(complexity) = bijux_dna_domain_bam::metrics::parse_preseq_estimates(&path) {
            metrics.complexity = complexity;
        }
    }

    let insert_size_path = discovery::first_existing(out_dir, &["insert_size.metrics.txt"]);
    if let Some(path) = insert_size_path {
        if let Ok(insert_size) =
            bijux_dna_domain_bam::metrics::parse_picard_insert_size_metrics(&path)
        {
            metrics.insert_size = insert_size;
        }
    }

    let gc_bias_path = discovery::first_existing(out_dir, &["gc_bias.metrics.txt"]);
    if let Some(path) = gc_bias_path {
        if let Ok(gc_bias) = bijux_dna_domain_bam::metrics::parse_picard_gc_bias_metrics(&path) {
            metrics.gc_bias = gc_bias;
        }
    }
}

fn parse_damage_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let mut damage_sources: Vec<(String, bijux_dna_domain_bam::metrics::DamageMetricsV1)> =
        Vec::new();
    let pydamage_path =
        discovery::first_existing(out_dir, &["damage.pydamage.json", "pydamage.json"]);
    if let Some(path) = pydamage_path {
        if let Ok(damage) = bijux_dna_domain_bam::metrics::parse_pydamage_json(&path) {
            metrics.damage = damage.clone();
            damage_sources.push(("pydamage".to_string(), damage));
        }
    }
    let mapdamage2_path =
        discovery::first_existing(out_dir, &["damage.mapdamage2.txt", "mapdamage2.txt"]);
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
        discovery::first_existing(out_dir, &["damage.profiler.json", "damageprofiler.json"]);
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
}

fn parse_contamination_and_sex(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let contamination_path = discovery::first_existing(out_dir, &["contamination.json"]);
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

    let sex_path = discovery::first_existing(out_dir, &["sex.json"]);
    if let Some(path) = sex_path {
        if let Ok(sex) = bijux_dna_domain_bam::metrics::parse_sex_json(&path) {
            metrics.sex = sex;
        }
    }
}
