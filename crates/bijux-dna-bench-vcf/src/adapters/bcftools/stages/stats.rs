//! Owner: bijux-dna-bench-vcf
//! bcftools stats command and output contract.
use super::super::commands::{artifact, input_by_id, step, StageContract};
use super::super::{Result, VcfBcftoolsAdapterArtifact};

pub(crate) fn build(
    output_root: &str,
    inputs: &[VcfBcftoolsAdapterArtifact],
) -> Result<StageContract> {
    Ok({
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let text_output_path = |name: &str| format!("{output_root}/{name}.txt");
        let output = text_output_path("bcftools_stats");
        StageContract {
            raw_output_ids: vec!["bcftools_stats_txt".to_string()],
            parser_output_ids: vec!["stats_json".to_string()],
            declared_outputs: vec![
                artifact("bcftools_stats_txt", "report", &output),
                artifact("stats_json", "metrics_json", &json_output_path("stats_json")),
            ],
            command_steps: vec![step(
                "render_stats",
                "report",
                vec!["bcftools", "stats", "-s", "-", "-o", &output, input_by_id(inputs, "vcf")?],
                false,
                &["bcftools_stats_txt"],
            )],
        }
    })
}
