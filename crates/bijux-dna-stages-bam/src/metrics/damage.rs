use std::path::Path;

use bijux_dna_domain_bam::metrics::{BamMetricsV1, DamageMetricsV1};

use super::discovery;

pub(super) fn parse_damage_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let mut damage_sources: Vec<(String, DamageMetricsV1)> = Vec::new();
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
