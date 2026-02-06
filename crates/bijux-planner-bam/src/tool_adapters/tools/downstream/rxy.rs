use std::path::Path;

use bijux_domain_bam::params::SexEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    params: &SexEffectiveParams,
) -> Vec<String> {
    let method = params.method.clone();
    let command = format!(
        "rxy --input {bam} > {report} && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"method\": \"{method}\", \"x_to_y_ratio\": 0.0, \"confidence\": 0.0}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        report = report.display(),
        summary = summary.display(),
        method = method
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
