use std::path::Path;

use bijux_dna_domain_bam::params::SexEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    _params: &SexEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "rxy --input {bam} > {report} && cp {report} {summary}",
        bam = bam.display(),
        report = report.display(),
        summary = summary.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
