use std::path::Path;

use bijux_domain_bam::params::SexEffectiveParams;

#[must_use]
pub fn args(bam: &Path, _params: &SexEffectiveParams) -> Vec<String> {
    vec![
        "rxy".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
    ]
}
