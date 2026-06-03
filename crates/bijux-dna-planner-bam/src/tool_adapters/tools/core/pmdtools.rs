use std::path::Path;

use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;
use bijux_dna_domain_bam::params::DamageEffectiveParams;

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
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"method\": \"pmdtools\", \"stage\": \"{stage}\"}}, indent=2))\nPY",
        bam = bam.display(),
        filtered = filtered_bam.display(),
        report = report_json.display(),
        summary = summary_json.display(),
        stage = id_catalog::BAM_AUTHENTICITY
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

#[must_use]
pub fn damage_args(bam: &Path, report_json: &Path, _params: &DamageEffectiveParams) -> Vec<String> {
    let command = format!(
        "pmdtools --input {bam} > /dev/null && \
python3 - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"tool\": \"pmdtools\", \"stage\": \"bam.damage\", \"c_to_t_5p\": 0.0, \"g_to_a_3p\": 0.0, \"pmd_score_histogram\": []}}, indent=2))\nPY",
        bam = bam.display(),
        report = report_json.display(),
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
