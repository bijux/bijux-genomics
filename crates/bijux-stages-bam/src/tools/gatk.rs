use std::path::Path;

use bijux_domain_bam::params::MarkDupEffectiveParams;

#[must_use]
pub fn markdup_args(
    bam: &Path,
    out_bam: &Path,
    flagstat: &Path,
    idxstats: &Path,
    params: &MarkDupEffectiveParams,
) -> Vec<String> {
    let remove = matches!(params.duplicate_action, bijux_domain_bam::DuplicateAction::Remove);
    let command = format!(
        "gatk MarkDuplicatesSpark -I {bam} -O {out} --REMOVE_DUPLICATES {remove} --CREATE_INDEX true && samtools flagstat {out} > {flagstat} && samtools idxstats {out} > {idxstats}",
        bam = bam.display(),
        out = out_bam.display(),
        remove = if remove { "true" } else { "false" },
        flagstat = flagstat.display(),
        idxstats = idxstats.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
