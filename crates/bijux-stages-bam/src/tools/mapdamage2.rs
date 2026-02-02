use std::path::Path;

use bijux_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn damage_args(bam: &Path, out_dir: &Path, _params: &DamageEffectiveParams) -> Vec<String> {
    let command = format!(
        "mapDamage --bam {bam} --folder {out}",
        bam = bam.display(),
        out = out_dir.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
