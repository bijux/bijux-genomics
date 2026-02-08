use std::path::Path;

use bijux_dna_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_json: &Path, params: &DamageEffectiveParams) -> Vec<String> {
    vec![
        "pydamage".to_string(),
        "analyze".to_string(),
        "--input".to_string(),
        bam.display().to_string(),
        "--output".to_string(),
        out_json.display().to_string(),
        "--min-mapq".to_string(),
        params.pmd_threshold_5p.to_string(),
    ]
}
