//! Owner: bijux-dna-bench-vcf
//! plink pca command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.pca/plink2.txt".to_string(),
            normalized_metrics_artifact_id: "pca_report".to_string(),
            normalized_metrics_path: json_output_path("pca_report"),
            raw_output_ids: vec![
                "pca_eigenvec".to_string(),
                "pca_eigenval".to_string(),
                "plink2_log".to_string(),
            ],
            parser_output_ids: vec!["pca_report".to_string()],
            declared_outputs: vec![
                artifact("pca_eigenvec", "report_tsv", &raw_path("eigenvec")),
                artifact("pca_eigenval", "report_tsv", &raw_path("eigenval")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("pca_report", "report_json", &json_output_path("pca_report")),
            ],
            command_steps: vec![step(
                "pca",
                "principal_components",
                vec![
                    "plink2",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--pca",
                    "10",
                    "--out",
                    output_prefix,
                ],
                &["pca_eigenvec", "pca_eigenval", "plink2_log"],
            )],
        }
    }
}
