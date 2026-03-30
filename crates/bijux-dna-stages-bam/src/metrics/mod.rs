use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

mod alignment;
mod contamination;
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
    contamination::parse_contamination_and_sex(out_dir, &mut metrics);
    metrics
}
