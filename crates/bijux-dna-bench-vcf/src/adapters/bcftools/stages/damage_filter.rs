//! Owner: bijux-dna-bench-vcf
//! bcftools damage filter command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let output = variant_output_path("damage_filtered_vcf");
        StageContract {
            raw_output_ids: vec![
                "damage_filtered_vcf".to_string(),
                "damage_filtered_vcf_tbi".to_string(),
            ],
            parser_output_ids: Vec::new(),
            declared_outputs: vec![
                artifact("damage_filtered_vcf", "variant", &output),
                artifact("damage_filtered_vcf_tbi", "index", &format!("{output}.tbi")),
            ],
            command_steps: vec![
                step(
                    "damage_filter",
                    "transform",
                    vec![
                        "bcftools",
                        "filter",
                        "-e",
                        "((REF=\"C\" && ALT=\"T\") || (REF=\"G\" && ALT=\"A\")) && INFO/PMD>3",
                        input_by_id(inputs, "vcf")?,
                        "-Oz",
                        "-o",
                        &output,
                    ],
                    false,
                    &["damage_filtered_vcf"],
                ),
                step(
                    "index_damage_filtered_vcf",
                    "index",
                    vec!["bcftools", "index", "-t", &output],
                    false,
                    &["damage_filtered_vcf_tbi"],
                ),
            ],
        }
    })
}
