//! Owner: bijux-dna-bench-vcf
//! plink admixture command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.admixture/plink2.txt".to_string(),
            normalized_metrics_artifact_id: "admixture_report".to_string(),
            normalized_metrics_path: json_output_path("admixture_report"),
            raw_output_ids: vec![
                "admixture_proxy_eigenvec".to_string(),
                "admixture_proxy_eigenval".to_string(),
                "plink2_log".to_string(),
            ],
            parser_output_ids: vec!["admixture_report".to_string()],
            declared_outputs: vec![
                artifact("admixture_proxy_eigenvec", "report_tsv", &raw_path("eigenvec")),
                artifact("admixture_proxy_eigenval", "report_tsv", &raw_path("eigenval")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("admixture_report", "report_json", &json_output_path("admixture_report")),
            ],
            command_steps: vec![step(
                "admixture_proxy",
                "population_proxy",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--pca",
                    "2",
                    "--out",
                    output_prefix,
                ],
                &["admixture_proxy_eigenvec", "admixture_proxy_eigenval", "plink2_log"],
            )],
        }
    }
}
