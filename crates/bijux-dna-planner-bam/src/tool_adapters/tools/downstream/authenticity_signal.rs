use std::path::Path;

use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    params: &AuthenticityEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "authenticity_tool --input {bam} --mode {mode} > {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"authenticity_tool\", \"mode\": \"{mode}\", \"status\": \"ok\"}}, indent=2))\nPY",
        bam = bam.display(),
        mode = params.mode,
        report = report.display(),
        summary = summary.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
