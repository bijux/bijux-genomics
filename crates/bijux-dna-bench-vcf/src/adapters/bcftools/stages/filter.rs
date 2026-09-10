//! Owner: bijux-dna-bench-vcf
//! bcftools filter command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("filtered_vcf");
        StageContract {
            raw_output_ids: vec!["filtered_vcf".to_string(), "filtered_vcf_tbi".to_string()],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("filtered_vcf", "variant", &output),
                artifact("filtered_vcf_tbi", "index", &format!("{output}.tbi")),
            ],
            command_steps: vec![
                step(
                    "filter_sites",
                    "transform",
                    vec![
                        "bcftools",
                        "filter",
                        "-s",
                        "LOWQUAL",
                        "-e",
                        "QUAL<30",
                        input_by_id(inputs, "vcf")?,
                        "-Oz",
                        "-o",
                        &output,
                    ],
                    false,
                    &["filtered_vcf"],
                ),
                step(
                    "index_filtered_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["filtered_vcf_tbi"],
                ),
            ],
        }
    })
}
