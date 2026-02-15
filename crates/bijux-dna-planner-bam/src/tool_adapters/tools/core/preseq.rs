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
    preseq_txt: &Path,
    complexity_json: &Path,
    summary_json: &Path,
    _params: &ComplexityEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "preseq lc_extrap -o {preseq} {bam} && \
python - <<'PY' > {complexity}\nimport json\nprint(json.dumps({{\"source\": \"preseq\", \"report\": \"{preseq}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"stage\": \"bam.complexity\", \"preseq\": \"{preseq}\"}}, indent=2))\nPY",
        bam = bam.display(),
        preseq = preseq_txt.display(),
        complexity = complexity_json.display(),
        summary = summary_json.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
