use std::path::Path;

use bijux_dna_domain_bam::params::ComplexityEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_path: &Path, _params: &ComplexityEffectiveParams) -> Vec<String> {
    vec![
        "preseq".to_string(),
        "lc_extrap".to_string(),
        "-o".to_string(),
        out_path.display().to_string(),
        bam.display().to_string(),
    ]
}
