//! Owner: bijux-dna-bench-vcf
//! angsd call gl command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfAngsdAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfAngsdAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let vcf_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let arg_output_path = |prefix: &str| format!("{prefix}.arg");
        let prefix = format!("{output_root}/gl_sites.native");
        let native_vcf = format!("{prefix}.vcf.gz");
        let output = vcf_output_path("gl_sites");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.call_gl/angsd.txt".to_string(),
            reference_path: Some(input_by_id(inputs, "reference_fasta")?.to_string()),
            sites_path: Some(input_by_id(inputs, "sites_vcf")?.to_string()),
            likelihood_model: Some("GL2_doGlf2_doMajorMinor1_doMaf1".to_string()),
            output_prefix: prefix.clone(),
            stage_output_ids: vec!["gl_sites_vcf".to_string()],
            raw_output_ids: vec![
                "angsd_native_vcf".to_string(),
                "gl_sites_vcf".to_string(),
                "gl_sites_vcf_tbi".to_string(),
                "angsd_arg".to_string(),
            ],
            parser_output_ids: vec!["gl_provenance".to_string()],
            declared_outputs: vec![
                artifact("angsd_native_vcf", "raw_variant", &native_vcf),
                artifact("gl_sites_vcf", "variant", &output),
                artifact("gl_sites_vcf_tbi", "index", &format!("{output}.tbi")),
                artifact("angsd_arg", "report", &arg_output_path(&prefix)),
                artifact("gl_provenance", "report_json", &json_output_path("gl_provenance")),
            ],
            command_steps: vec![
                step(
                    "call_gl",
                    "call",
                    vec![
                        "angsd",
                        "-b",
                        input_by_id(inputs, "bam_list")?,
                        "-ref",
                        input_by_id(inputs, "reference_fasta")?,
                        "-sites",
                        input_by_id(inputs, "sites_vcf")?,
                        "-GL",
                        "2",
                        "-doGlf",
                        "2",
                        "-doMajorMinor",
                        "1",
                        "-doMaf",
                        "1",
                        "-minMapQ",
                        "30",
                        "-minQ",
                        "20",
                        "-setMinDepth",
                        "3",
                        "-setMaxDepth",
                        "100",
                        "-doVcf",
                        "1",
                        "-out",
                        &prefix,
                    ],
                    &["angsd_native_vcf", "angsd_arg"],
                ),
                step(
                    "normalize_gl_sites_vcf",
                    "result_normalizer",
                    vec!["bcftools", "view", "-Oz", "-o", &output, &native_vcf],
                    &["gl_sites_vcf"],
                ),
                step(
                    "index_gl_sites_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    &["gl_sites_vcf_tbi"],
                ),
            ],
        }
    })
}
