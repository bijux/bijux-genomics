//! Owner: bijux-dna-bench-vcf
//! bcftools call gl command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("gl_sites_vcf");
        StageContract {
            raw_output_ids: vec!["gl_sites_vcf".to_string(), "gl_sites_vcf_tbi".to_string()],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("gl_sites_vcf", "variant", &output),
                artifact("gl_sites_vcf_tbi", "index", &format!("{output}.tbi")),
            ],
            command_steps: vec![
                step(
                    "mpileup",
                    "pipeline_source",
                    vec![
                        "bcftools",
                        "mpileup",
                        "-Ou",
                        "-f",
                        input_by_id(inputs, "reference_fasta")?,
                        input_by_id(inputs, "input_bam")?,
                    ],
                    false,
                    &[],
                ),
                step(
                    "call_gl",
                    "pipeline_sink",
                    vec!["bcftools", "call", "-Aim", "-Oz", "-o", &output],
                    true,
                    &["gl_sites_vcf"],
                ),
                step(
                    "index_gl_sites_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["gl_sites_vcf_tbi"],
                ),
            ],
        }
    })
}
