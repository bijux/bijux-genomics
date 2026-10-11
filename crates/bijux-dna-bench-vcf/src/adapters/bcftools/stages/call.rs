//! Owner: bijux-dna-bench-vcf
//! bcftools call command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("called_vcf");
        StageContract {
            raw_output_ids: vec!["called_vcf".to_string(), "called_vcf_tbi".to_string()],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("called_vcf", "variant", &output),
                artifact("called_vcf_tbi", "index", &format!("{output}.tbi")),
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
                    "call",
                    "pipeline_sink",
                    vec!["bcftools", "call", "-c", "-Oz", "-o", &output],
                    true,
                    &["called_vcf"],
                ),
                step(
                    "index_called_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["called_vcf_tbi"],
                ),
            ],
        }
    })
}
