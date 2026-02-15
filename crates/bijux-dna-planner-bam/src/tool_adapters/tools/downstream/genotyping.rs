use std::path::Path;

use bijux_dna_domain_bam::params::GenotypingEffectiveParams;

#[must_use]
pub fn args_with_outputs(
    tool_id: &str,
    bam: &Path,
    report: &Path,
    summary: &Path,
    vcf_gz: &Path,
    tbi: &Path,
    gl_json: &Path,
    params: &GenotypingEffectiveParams,
) -> Vec<String> {
    let min_posterior = params.min_posterior.unwrap_or(0.0);
    let min_call_rate = params.min_call_rate.unwrap_or(0.0);
    let command = match tool_id {
        "angsd" => format!(
            "angsd -i {bam} -doGlf 2 -doMajorMinor 1 -doMaf 1 -out {prefix} && \
bcftools view {prefix}.bcf -Oz -o {vcf} && tabix -f -p vcf {vcf} && \
python - <<'PY' > {gl}\nimport json\nprint(json.dumps({{\"producer\": \"angsd\", \"gl_source\": \"{prefix}.glf.gz\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"caller\": \"angsd\", \"vcf\": \"{vcf}\", \"tbi\": \"{tbi}\", \"min_posterior\": {min_posterior}, \"min_call_rate\": {min_call_rate}}}, indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"status\": \"ok\", \"producer\": \"bam.genotyping\"}}, indent=2))\nPY",
            bam = bam.display(),
            prefix = report.with_extension("").display(),
            vcf = vcf_gz.display(),
            tbi = tbi.display(),
            gl = gl_json.display(),
            summary = summary.display(),
            report = report.display(),
            min_posterior = min_posterior,
            min_call_rate = min_call_rate
        ),
        _ => format!(
            "gatk HaplotypeCaller -I {bam} -O {vcf} -ERC GVCF && \
tabix -f -p vcf {vcf} && \
python - <<'PY' > {gl}\nimport json\nprint(json.dumps({{\"producer\": \"gatk\", \"mode\": \"gvcf\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"caller\": \"gatk\", \"vcf\": \"{vcf}\", \"tbi\": \"{tbi}\", \"min_posterior\": {min_posterior}, \"min_call_rate\": {min_call_rate}}}, indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"status\": \"ok\", \"producer\": \"bam.genotyping\"}}, indent=2))\nPY",
            bam = bam.display(),
            vcf = vcf_gz.display(),
            tbi = tbi.display(),
            gl = gl_json.display(),
            summary = summary.display(),
            report = report.display(),
            min_posterior = min_posterior,
            min_call_rate = min_call_rate
        ),
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
