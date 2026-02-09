use std::path::Path;

use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;

#[must_use]
pub fn filter_args(
    bam: &Path,
    filtered_bam: &Path,
    report_json: &Path,
    summary_json: &Path,
    _params: &AuthenticityEffectiveParams,
) -> Vec<String> {
    let command = format!(
        "pmdtools --input {bam} --output {filtered} > {report} && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"pmdtools\", \"stage\": \"bam.authenticity\"}}, indent=2))\nPY",
        bam = bam.display(),
        filtered = filtered_bam.display(),
        report = report_json.display(),
        summary = summary_json.display()
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
