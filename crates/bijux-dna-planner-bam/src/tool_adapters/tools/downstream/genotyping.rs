use std::path::Path;

use bijux_dna_domain_bam::params::GenotypingEffectiveParams;

pub struct GenotypingOutputs<'a> {
    pub report: &'a Path,
    pub summary: &'a Path,
    pub vcf_gz: &'a Path,
    pub tbi: &'a Path,
    pub gl_json: &'a Path,
}

#[must_use]
pub fn args_with_outputs(
    tool_id: &str,
    bam: &Path,
    outputs: GenotypingOutputs<'_>,
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
            prefix = outputs.report.with_extension("").display(),
            vcf = outputs.vcf_gz.display(),
            tbi = outputs.tbi.display(),
            gl = outputs.gl_json.display(),
            summary = outputs.summary.display(),
            report = outputs.report.display(),
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
            vcf = outputs.vcf_gz.display(),
            tbi = outputs.tbi.display(),
            gl = outputs.gl_json.display(),
            summary = outputs.summary.display(),
            report = outputs.report.display(),
            min_posterior = min_posterior,
            min_call_rate = min_call_rate
        ),
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}
