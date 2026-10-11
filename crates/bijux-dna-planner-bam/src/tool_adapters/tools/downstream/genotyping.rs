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
    pub stage_metrics: &'a Path,
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
    let bam_index_check = context
        .bam_index
        .map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let reference_check = context
        .reference
        .map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let sites_check =
        context.sites.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let regions_check =
        context.regions.map_or_else(String::new, |path| format!("test -f {} && ", path.display()));
    let sites_arg =
        context.sites.map_or_else(String::new, |path| format!(" -T {}", path.display()));
    let regions_arg =
        context.regions.map_or_else(String::new, |path| format!(" -R {}", path.display()));
    let reference_json =
        context.reference.map(|path| path.display().to_string()).unwrap_or_default();
    let sites_json = context.sites.map(|path| path.display().to_string()).unwrap_or_default();
    let regions_json = context.regions.map(|path| path.display().to_string()).unwrap_or_default();
    let bcf_path = outputs
        .bcf
        .map_or_else(|| outputs.report.with_extension("bcf"), std::path::Path::to_path_buf);
    let raw_bcf_path = bcf_path.with_extension("raw.bcf");
    let command = match tool_id {
        "bcftools" => format!(
            "{bam_index_check}{reference_check}{sites_check}{regions_check}\
test -n {reference_quoted} && \
bcftools mpileup -Ou -f {reference_quoted}{sites_arg}{regions_arg} {bam} | \
bcftools call -m -v -Ob -o {raw_bcf} && \
bcftools +tag2tag {raw_bcf} -Ob -o {bcf} -- --PL-to-GP && rm -f {raw_bcf} && \
bcftools view -Oz -o {vcf} {bcf} && bcftools index --force --tbi {vcf} && \
variant_count=$(bcftools view -H {vcf} | wc -l) && \
sample_count=$(bcftools query -l {vcf} | wc -l) && \
genotype_counts=$(bcftools query -f '[%GT\\t%GP\\n]' {vcf} | awk -v minimum_posterior=\"{min_posterior}\" 'BEGIN{{called=0; total=0}} {{total++; highest=0; n=split($2, probabilities, \",\"); for (i=1; i<=n; i++) if (probabilities[i]+0 > highest) highest=probabilities[i]+0; if ($1 != \"./.\" && $1 != \".|.\" && $1 != \".\" && $2 != \".\" && highest >= minimum_posterior) called++}} END{{printf \"%d %d %.12f\", called, total, total ? called/total : 0}}') && \
set -- $genotype_counts && called_genotypes=$1 && total_genotypes=$2 && call_rate=$3 && \
awk -v observed=\"$call_rate\" -v required=\"{min_call_rate}\" 'BEGIN{{exit !(observed >= required)}}' && \
printf '{{\"producer\":\"bcftools\",\"bcf_source\":\"{bcf}\",\"likelihood_field\":\"PL\",\"posterior_field\":\"GP\",\"posterior_derivation\":\"bcftools_tag2tag_PL_to_GP\"}}\\n' > {gl} && \
printf '{{\"caller\":\"bcftools\",\"reference\":\"{reference}\",\"sites\":\"{sites}\",\"regions\":\"{regions}\",\"bcf\":\"{bcf}\",\"vcf\":\"{vcf}\",\"tbi\":\"{tbi}\",\"minimum_posterior\":{min_posterior},\"minimum_call_rate\":{min_call_rate},\"posterior_derivation\":\"bcftools_tag2tag_PL_to_GP\"}}\\n' > {summary} && \
printf '{{\"variant_count\":%s,\"sample_count\":%s,\"confident_called_genotypes\":%s,\"total_genotypes\":%s,\"minimum_posterior\":{min_posterior},\"confident_call_rate\":%s}}\\n' \"$variant_count\" \"$sample_count\" \"$called_genotypes\" \"$total_genotypes\" \"$call_rate\" > {metrics} && \
printf '{{\"status\":\"ok\",\"producer\":\"bam.genotyping\",\"caller\":\"bcftools\",\"output_bcf\":\"{bcf}\",\"output_vcf\":\"{vcf}\"}}\\n' > {report}",
            bam_index_check = bam_index_check,
            reference_check = reference_check,
            sites_check = sites_check,
            regions_check = regions_check,
            bam = bam.display(),
            sites_arg = sites_arg,
            regions_arg = regions_arg,
            reference_quoted = shell_quote(&reference_json),
            reference = reference_json,
            sites = sites_json,
            regions = regions_json,
            bcf = bcf_path.display(),
            raw_bcf = raw_bcf_path.display(),
            vcf = outputs.vcf_gz.display(),
            tbi = outputs.tbi.display(),
            gl = outputs.gl_json.display(),
            summary = outputs.summary.display(),
            metrics = outputs.stage_metrics.display(),
            report = outputs.report.display(),
            min_posterior = min_posterior,
            min_call_rate = min_call_rate
        ),
        _ => format!("printf 'unsupported bam.genotyping tool: %s\\n' {tool} >&2; exit 2", tool = shell_quote(tool_id)),
    };
    vec!["/bin/sh".to_string(), "-c".to_string(), command]
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\\"'\\\"'"))
}
