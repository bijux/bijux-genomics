use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

mod alignment;
mod coverage;
mod damage;
mod discovery;
mod quality;

#[must_use]
pub fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();
    alignment::parse_alignment_metrics(out_dir, &mut metrics);
    coverage::parse_coverage_metrics(out_dir, &mut metrics);
    quality::parse_quality_metrics(out_dir, &mut metrics);
    damage::parse_damage_metrics(out_dir, &mut metrics);
    parse_contamination_and_sex(out_dir, &mut metrics);
    metrics
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
