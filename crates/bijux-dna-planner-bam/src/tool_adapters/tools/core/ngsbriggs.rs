use std::path::Path;

use bijux_dna_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn args(
    bam: &Path,
    reference: &Path,
    out_json: &Path,
    _params: &DamageEffectiveParams,
) -> Vec<String> {
    vec![
        "ngsbriggs".to_string(),
        "analyze".to_string(),
        "--bam".to_string(),
        bam.display().to_string(),
        "--reference".to_string(),
        reference.display().to_string(),
        "--json-out".to_string(),
        out_json.display().to_string(),
        "--model".to_string(),
        "nb".to_string(),
    ]
}
