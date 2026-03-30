use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

use super::discovery;

pub(super) fn parse_coverage_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let stats_path = discovery::first_existing(out_dir, &["samtools_stats.txt"]);
    if let Some(path) = stats_path {
        if let Ok((fragment, mapq)) = bijux_dna_domain_bam::metrics::parse_samtools_stats(&path) {
            metrics.fragment_length = fragment;
            metrics.mapq = mapq;
        }
    }
    let mosdepth_path = discovery::first_existing(
        out_dir,
        &["coverage.mosdepth.summary.txt", "mosdepth.summary.txt"],
    );
    if let Some(path) = mosdepth_path {
        if let Ok(coverage) = bijux_dna_domain_bam::metrics::parse_mosdepth_summary(&path) {
            metrics.coverage = coverage;
        }
    } else {
        let depth_path = discovery::first_existing(out_dir, &["coverage.depth.txt", "depth.txt"]);
        if let Some(path) = depth_path {
            if let Ok((coverage, uniformity)) =
                bijux_dna_domain_bam::metrics::parse_samtools_depth_with_uniformity(&path)
            {
                metrics.coverage = coverage;
                metrics.coverage_uniformity = uniformity;
            }
        }
    }
}
