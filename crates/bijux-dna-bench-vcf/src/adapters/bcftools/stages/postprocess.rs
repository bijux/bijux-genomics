//! Owner: bijux-dna-bench-vcf
//! bcftools postprocess command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("postprocess_vcf");
        StageContract {
            raw_output_ids: vec!["postprocess_vcf".to_string(), "postprocess_vcf_tbi".to_string()],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("postprocess_vcf", "variant", &output),
                artifact("postprocess_vcf_tbi", "index", &format!("{output}.tbi")),
            ],
            command_steps: vec![
                step(
                    "fill_tags",
                    "transform",
                    vec![
                        "bcftools",
                        "+fill-tags",
                        input_by_id(inputs, "vcf")?,
                        "-Oz",
                        "-o",
                        &output,
                        "--",
                        "-t",
                        "AC,AN,AF",
                    ],
                    false,
                    &["postprocess_vcf"],
                ),
                step(
                    "index_postprocess_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["postprocess_vcf_tbi"],
                ),
            ],
        }
    })
}
