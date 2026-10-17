use std::path::{Path, PathBuf};

use bijux_dna_domain_bam::metrics::BamMetricsV1;

mod alignment;
mod contamination;
mod coverage;
mod damage;
mod discovery;
mod quality;

#[must_use]
pub fn bam_metrics_from_dir(out_dir: &Path) -> BamMetricsV1 {
    collect_metrics(&discovery::MetricInputs::Directory(out_dir))
}

#[must_use]
pub(crate) fn bam_metrics_from_files(paths: &[PathBuf]) -> BamMetricsV1 {
    collect_metrics(&discovery::MetricInputs::Declared(paths))
}

fn collect_metrics(inputs: &discovery::MetricInputs<'_>) -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();
    alignment::parse_alignment_metrics(inputs, &mut metrics);
    coverage::parse_coverage_metrics(inputs, &mut metrics);
    quality::parse_quality_metrics(inputs, &mut metrics);
    damage::parse_damage_metrics(inputs, &mut metrics);
    contamination::parse_contamination_and_sex(inputs, &mut metrics);
    metrics
}
