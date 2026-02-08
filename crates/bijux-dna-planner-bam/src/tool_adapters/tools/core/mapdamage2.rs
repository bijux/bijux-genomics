use std::path::Path;

use bijux_dna_domain_bam::params::DamageEffectiveParams;

#[must_use]
pub fn damage_args(bam: &Path, out_dir: &Path, _params: &DamageEffectiveParams) -> Vec<String> {
    let out_file = out_dir.join("damage.mapdamage2.txt");
    let command = format!(
        "mapDamage --bam {bam} --folder {out} && \
if [ -f {out}/misincorporation.txt ]; then \
  cp {out}/misincorporation.txt {out_file}; \
elif [ -f {out}/5pCtoT.txt ]; then \
  cp {out}/5pCtoT.txt {out_file}; \
else \
  : > {out_file}; \
fi",
        bam = bam.display(),
        out = out_dir.display(),
        out_file = out_file.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
