use std::path::Path;

use bijux_dna_domain_bam::params::GenotypingEffectiveParams;

#[derive(Debug, Clone, Copy, Default)]
pub struct GenotypingPlanningContext<'a> {
    pub bam_index: Option<&'a Path>,
    pub reference: Option<&'a Path>,
    pub sites: Option<&'a Path>,
    pub regions: Option<&'a Path>,
}

pub struct GenotypingOutputs<'a> {
    pub report: &'a Path,
    pub summary: &'a Path,
    pub bcf: Option<&'a Path>,
    pub vcf_gz: &'a Path,
    pub tbi: &'a Path,
    pub gl_json: &'a Path,
}

#[must_use]
pub fn args_with_outputs(
    tool_id: &str,
    bam: &Path,
    context: GenotypingPlanningContext<'_>,
    outputs: GenotypingOutputs<'_>,
    params: &GenotypingEffectiveParams,
) -> Vec<String> {
    let min_posterior = params.min_posterior.unwrap_or(0.0);
    let min_call_rate = params.min_call_rate.unwrap_or(0.0);
    let bam_index_check =
        context.bam_index.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let reference_check =
        context.reference.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let sites_check =
        context.sites.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let regions_check =
        context.regions.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let sites_arg = context
        .sites
        .map_or_else(String::new, |path| format!(" -sites {}", path.display()));
    let regions_arg = context
        .regions
        .map_or_else(String::new, |path| format!(" -rf {}", path.display()));
    let reference_json =
        context.reference.map(|path| path.display().to_string()).unwrap_or_default();
    let sites_json = context.sites.map(|path| path.display().to_string()).unwrap_or_default();
    let regions_json =
        context.regions.map(|path| path.display().to_string()).unwrap_or_default();
    let bcf_path = outputs
        .bcf
        .map_or_else(|| outputs.report.with_extension("bcf"), std::path::Path::to_path_buf);
    let command = match tool_id {
        "angsd" => format!(
            "{bam_index_check}{reference_check}{sites_check}{regions_check}angsd -i {bam}{sites_arg}{regions_arg} -doGlf 2 -doMajorMinor 1 -doMaf 1 -out {prefix} && \
bcftools view {bcf} -Oz -o {vcf} && tabix -f -p vcf {vcf} && \
python - <<'PY' > {gl}\nimport json\nprint(json.dumps({{\"producer\": \"angsd\", \"gl_source\": \"{prefix}.glf.gz\", \"bcf_source\": \"{bcf}\"}}, indent=2))\nPY && \
python - <<'PY' > {summary}\nimport json\nprint(json.dumps({{\"caller\": \"angsd\", \"reference\": \"{reference}\", \"sites\": \"{sites}\", \"regions\": \"{regions}\", \"bcf\": \"{bcf}\", \"vcf\": \"{vcf}\", \"tbi\": \"{tbi}\", \"min_posterior\": {min_posterior}, \"min_call_rate\": {min_call_rate}}}, indent=2))\nPY && \
python - <<'PY' > {report}\nimport json\nprint(json.dumps({{\"status\": \"ok\", \"producer\": \"bam.genotyping\", \"reference\": \"{reference}\", \"sites\": \"{sites}\", \"regions\": \"{regions}\", \"output_bcf\": \"{bcf}\", \"output_vcf\": \"{vcf}\"}}, indent=2))\nPY",
            bam_index_check = bam_index_check,
            reference_check = reference_check,
            sites_check = sites_check,
            regions_check = regions_check,
            bam = bam.display(),
            sites_arg = sites_arg,
            regions_arg = regions_arg,
            prefix = outputs.report.with_extension("").display(),
            reference = reference_json,
            sites = sites_json,
            regions = regions_json,
            bcf = bcf_path.display(),
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
