use std::path::Path;

use bijux_dna_domain_bam::params::CoverageEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_prefix: &Path, _params: &CoverageEffectiveParams) -> Vec<String> {
    let depth_txt = out_prefix.with_extension("depth.txt");
    let mosdepth_summary = out_prefix.with_extension("mosdepth.summary.txt");
    let command = format!(
        "mosdepth -n {prefix} {bam} && \
samtools depth -a {bam} > {depth} && \
if [ -f {prefix}.mosdepth.summary.txt ]; then cp {prefix}.mosdepth.summary.txt {summary}; else : > {summary}; fi",
        prefix = out_prefix.display(),
        bam = bam.display(),
        depth = depth_txt.display(),
        summary = mosdepth_summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
