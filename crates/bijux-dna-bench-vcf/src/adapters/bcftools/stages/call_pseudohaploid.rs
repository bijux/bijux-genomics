//! Owner: bijux-dna-bench-vcf
//! bcftools call pseudohaploid command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("pseudohaploid_vcf");
        StageContract {
            raw_output_ids: vec![
                "pseudohaploid_vcf".to_string(),
                "pseudohaploid_vcf_tbi".to_string(),
            ],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("pseudohaploid_vcf", "variant", &output),
                artifact("pseudohaploid_vcf_tbi", "index", &format!("{output}.tbi")),
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
                    "call_pseudohaploid",
                    "pipeline_sink",
                    vec!["bcftools", "call", "--ploidy", "1", "-mv", "-Oz", "-o", &output],
                    true,
                    &["pseudohaploid_vcf"],
                ),
                step(
                    "index_pseudohaploid_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["pseudohaploid_vcf_tbi"],
                ),
            ],
        }
    })
}
