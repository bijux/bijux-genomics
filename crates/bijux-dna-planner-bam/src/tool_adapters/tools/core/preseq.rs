use std::path::Path;

use bijux_dna_domain_bam::params::ComplexityEffectiveParams;

#[must_use]
pub fn args(bam: &Path, out_path: &Path, _params: &ComplexityEffectiveParams) -> Vec<String> {
    vec![
        "preseq".to_string(),
        "lc_extrap".to_string(),
        "-o".to_string(),
        out_path.display().to_string(),
        bam.display().to_string(),
    ]
}

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    complexity_curve_tsv: &Path,
    complexity_json: &Path,
    summary_json: &Path,
    _params: &ComplexityEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "preseq lc_extrap -o {complexity_curve} {bam} && \
python - <<'PY' > {complexity}\nimport json\nprint(json.dumps({{\"source\": \"preseq\", \"complexity_curve\": \"{complexity_curve}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"stage\": \"bam.complexity\", \"complexity_curve\": \"{complexity_curve}\"}}, indent=2))\nPY",
        bam = bam.display(),
        complexity_curve = complexity_curve_tsv.display(),
        complexity = complexity_json.display(),
        summary = summary_json.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
