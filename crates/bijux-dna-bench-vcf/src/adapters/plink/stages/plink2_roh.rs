//! Owner: bijux-dna-bench-vcf
//! plink roh command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.roh/plink2.txt".to_string(),
            normalized_metrics_artifact_id: "roh_report".to_string(),
            normalized_metrics_path: json_output_path("roh_report"),
            raw_output_ids: vec!["roh_hom".to_string(), "plink2_log".to_string()],
            parser_output_ids: vec!["roh_report".to_string()],
            declared_outputs: vec![
                artifact("roh_hom", "report_tsv", &raw_path("hom")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("roh_report", "report_json", &json_output_path("roh_report")),
            ],
            command_steps: vec![step(
                "roh",
                "runs_of_homozygosity",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--homozyg",
                    "--out",
                    output_prefix,
                ],
                &["roh_hom", "plink2_log"],
            )],
        }
    }
}
