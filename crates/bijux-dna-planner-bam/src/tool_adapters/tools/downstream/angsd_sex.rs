use std::path::Path;

use bijux_dna_domain_bam::params::SexEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    params: &SexEffectiveParams,
) -> Vec<String> {
    let method = params.method.clone();
    let command = format!(
        "angsd -i {bam} -doCounts 1 -dumpCounts 2 -out {prefix} && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"method\": \"angsd\", \"counts_prefix\": \"{prefix}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"method\": \"{method}\", \"backend\": \"angsd\", \"x_to_y_ratio\": 0.0, \"confidence\": 0.0}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        report = report.display(),
        summary = summary.display(),
        prefix = report
            .parent()
            .map(|p| p.join("sex.angsd"))
            .unwrap_or_else(|| Path::new("sex.angsd").to_path_buf())
            .display(),
        method = method
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
