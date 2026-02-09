use std::path::Path;

use bijux_dna_domain_bam::params::ContaminationEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    _params: &ContaminationEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "contammix --bam {bam} > {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"contammix\", \"scope\": \"nuclear\"}}, indent=2))\nPY",
        bam = bam.display(),
        report = report.display(),
        summary = summary.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
