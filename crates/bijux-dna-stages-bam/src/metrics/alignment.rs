use std::path::Path;

use bijux_dna_domain_bam::metrics::BamMetricsV1;

use super::discovery;

pub(super) fn parse_alignment_metrics(out_dir: &Path, metrics: &mut BamMetricsV1) {
    let flagstat_path = discovery::first_existing(
        out_dir,
        &["flagstat.after.txt", "filter.flagstat.txt", "markdup.flagstat.txt", "flagstat.txt"],
    );
    if let Some(path) = flagstat_path {
        if let Ok(counts) = bijux_dna_domain_bam::metrics::parse_samtools_flagstat(&path) {
            metrics.alignment = counts;
        }
    }
    let idxstats_path = discovery::first_existing(out_dir, &["idxstats.after.txt", "idxstats.txt"]);
    if let Some(path) = idxstats_path {
        if let Ok(idxstats) = bijux_dna_domain_bam::metrics::parse_samtools_idxstats(&path) {
            metrics.idxstats = idxstats;
        }
    }
}
