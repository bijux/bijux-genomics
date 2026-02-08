use std::path::Path;

use bijux_domain_bam::params::ContaminationEffectiveParams;
use serde_json;

#[must_use]
pub fn args_with_outputs(
    bam: &Path,
    report: &Path,
    summary: &Path,
    params: &ContaminationEffectiveParams,
) -> Vec<String> {
    let assumptions = params.assumptions.as_ref().map_or_else(
        || "[]".to_string(),
        |value| serde_json::to_string(&vec![value]).unwrap_or_else(|_| "[]".to_string()),
    );
    let command = format!(
        "contamination_tool --input {bam} > {report} && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"method\": \"authenticct\", \"estimate\": 0.0, \"ci_low\": 0.0, \"ci_high\": 0.0, \"assumptions\": {assumptions}}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        report = report.display(),
        summary = summary.display(),
        assumptions = assumptions
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
