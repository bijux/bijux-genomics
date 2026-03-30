use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

use super::discovery;

pub(super) fn parse_quality_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
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
