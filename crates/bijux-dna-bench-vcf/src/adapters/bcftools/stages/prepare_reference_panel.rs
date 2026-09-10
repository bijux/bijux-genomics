//! Owner: bijux-dna-bench-vcf
//! bcftools prepare reference panel command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let variant_output_path = |name: &str| format!("{output_root}/{name}.vcf.gz");
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let panel_output = variant_output_path("prepared_panel");
        StageContract {
            raw_output_ids: vec!["prepared_panel".to_string(), "prepared_panel_tbi".to_string()],
            parser_output_ids: vec!["chunks_json".to_string()],
            declared_outputs: vec![
                artifact("prepared_panel", "variant", &panel_output),
                artifact("prepared_panel_tbi", "index", &format!("{panel_output}.tbi")),
                artifact("chunks_json", "report_json", &json_output_path("chunks")),
            ],
            command_steps: vec![
                step(
                    "normalize_panel",
                    "transform",
                    vec![
                        "bcftools",
                        "norm",
                        "-m-any",
                        input_by_id(inputs, "reference_panel_vcf")?,
                        "-Oz",
                        "-o",
                        &panel_output,
                    ],
                    false,
                    &["prepared_panel"],
                ),
                step(
                    "index_panel",
                    "index",
                    vec!["bcftools", "index", "-t", &panel_output],
                    false,
                    &["prepared_panel_tbi"],
                ),
            ],
        }
    })
}
