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
    let chromosome_system = params.chromosome_system.clone().unwrap_or_else(|| "xy".to_string());
    let minimum_y_sites = params.minimum_y_sites.unwrap_or(1);
    let output_prefix = report.with_extension("");
    let command = format!(
        "yleaf -bam {bam} -o {output_prefix} --reference_genome hg38 && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"method\": \"yleaf\", \"backend\": \"yleaf\", \"chromosome_system\": \"{chromosome_system}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\npayload = {{\"method\": \"{method}\", \"backend\": \"yleaf\", \"minimum_y_sites\": {minimum_y_sites}, \"x_to_y_ratio\": 0.0, \"confidence\": 0.0}}\nprint(json.dumps(payload, indent=2))\nPY",
        bam = bam.display(),
        output_prefix = output_prefix.display(),
        report = report.display(),
        summary = summary.display(),
        method = method,
        chromosome_system = chromosome_system,
        minimum_y_sites = minimum_y_sites,
    );
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
