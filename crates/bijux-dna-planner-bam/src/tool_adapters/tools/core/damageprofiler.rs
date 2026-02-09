use std::path::Path;

use bijux_dna_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_json: &Path, _params: &DamageEffectiveParams) -> Vec<String> {
    vec![
        "damageprofiler".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
    ]
}
