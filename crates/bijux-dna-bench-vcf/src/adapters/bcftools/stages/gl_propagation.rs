//! Owner: bijux-dna-bench-vcf
//! bcftools gl propagation command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("gl_propagated_vcf");
        StageContract {
            raw_output_ids: vec![
                "gl_propagated_vcf".to_string(),
                "gl_propagated_vcf_tbi".to_string(),
            ],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("gl_propagated_vcf", "variant", &output),
                artifact("gl_propagated_vcf_tbi", "index", &format!("{output}.tbi")),
            ],
            command_steps: vec![
                step(
                    "propagate_gl_fields",
                    "transform",
                    vec![
                        "bcftools",
                        "annotate",
                        "-x",
                        "INFO,^FORMAT/GL,^FORMAT/PL,^FORMAT/GP",
                        input_by_id(inputs, "vcf")?,
                        "-Oz",
                        "-o",
                        &output,
                    ],
                    false,
                    &["gl_propagated_vcf"],
                ),
                step(
                    "index_gl_propagated_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["gl_propagated_vcf_tbi"],
                ),
            ],
        }
    })
}
