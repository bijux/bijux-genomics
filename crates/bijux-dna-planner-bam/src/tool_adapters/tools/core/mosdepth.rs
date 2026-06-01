use std::path::Path;

use bijux_dna_domain_bam::params::CoverageEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_prefix: &Path, params: &CoverageEffectiveParams) -> Vec<String> {
    let depth_txt = out_prefix.with_extension("depth.txt");
    let mosdepth_summary = out_prefix.with_extension("mosdepth.summary.txt");
    let by_arg = params
        .regions
        .as_ref()
        .map_or_else(String::new, |regions| format!(" --by {}", regions.as_path().display()));
    let depth_regions_arg = params
        .regions
        .as_ref()
        .map_or_else(String::new, |regions| format!(" -b {}", regions.as_path().display()));
    let command = format!(
        "mosdepth -n{by_arg} {prefix} {bam} && \
samtools depth -a{depth_regions_arg} {bam} > {depth} && \
if [ -f {summary} ]; then :; else : > {summary}; fi",
        by_arg = by_arg,
        prefix = out_prefix.display(),
        bam = bam.display(),
        depth_regions_arg = depth_regions_arg,
        depth = depth_txt.display(),
        summary = mosdepth_summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
