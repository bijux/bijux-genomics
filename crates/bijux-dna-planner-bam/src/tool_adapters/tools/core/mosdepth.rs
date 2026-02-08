use std::path::Path;

use bijux_dna_domain_bam::params::CoverageEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_prefix: &Path, _params: &CoverageEffectiveParams) -> Vec<String> {
    vec![
        "mosdepth".to_string(),
        "-n".to_string(),
        out_prefix.display().to_string(),
        bam.display().to_string(),
    ]
}
