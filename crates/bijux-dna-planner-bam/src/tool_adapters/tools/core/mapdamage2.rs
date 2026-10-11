use std::path::Path;

use bijux_dna_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn damage_args(
    bam: &Path,
    reference: &Path,
    out_dir: &Path,
    _params: &DamageEffectiveParams,
) -> Vec<String> {
    vec![
        "mapdamage2".to_string(),
        "analyze".to_string(),
        "--bam".to_string(),
        bam.display().to_string(),
        "--reference".to_string(),
        reference.display().to_string(),
        "--out-dir".to_string(),
        out_dir.display().to_string(),
    ]
}
