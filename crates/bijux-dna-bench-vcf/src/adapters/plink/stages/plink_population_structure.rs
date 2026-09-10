//! Owner: bijux-dna-bench-vcf
//! plink population structure command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.population_structure/plink.txt"
                .to_string(),
            normalized_metrics_artifact_id: "population_structure_report".to_string(),
            normalized_metrics_path: json_output_path("population_structure_report"),
            raw_output_ids: vec!["population_mds".to_string(), "plink_log".to_string()],
            parser_output_ids: vec!["population_structure_report".to_string()],
            declared_outputs: vec![
                artifact("population_mds", "report_tsv", &raw_path("mds")),
                artifact("plink_log", "log", &raw_path("log")),
                artifact(
                    "population_structure_report",
                    "report_json",
                    &json_output_path("population_structure_report"),
                ),
            ],
            command_steps: vec![step(
                "population_structure",
                "multidimensional_scaling",
                vec![
                    "plink",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--cluster",
                    "--mds-plot",
                    "2",
                    "--out",
                    output_prefix,
                ],
                &["population_mds", "plink_log"],
            )],
        }
    }
}
