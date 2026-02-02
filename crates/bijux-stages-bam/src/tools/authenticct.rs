use std::path::Path;

use bijux_domain_bam::params::ContaminationEffectiveParams;

#[must_use]
pub fn args(bam: &Path, _params: &ContaminationEffectiveParams) -> Vec<String> {
    vec![
        "contamination_tool".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
    ]
}
