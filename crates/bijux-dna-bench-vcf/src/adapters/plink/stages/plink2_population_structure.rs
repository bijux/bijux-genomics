//! Owner: bijux-dna-bench-vcf
//! plink population structure command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let prune_prefix = format!("{output_prefix}.prune");
        let pca_prefix = format!("{output_prefix}.pca");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.population_structure/plink2.txt"
                .to_string(),
            normalized_metrics_artifact_id: "population_structure_report".to_string(),
            normalized_metrics_path: json_output_path("population_structure_report"),
            raw_output_ids: vec![
                "ld_prune_in".to_string(),
                "ld_prune_out".to_string(),
                "population_pca_eigenvec".to_string(),
                "population_pca_eigenval".to_string(),
                "prune_log".to_string(),
                "pca_log".to_string(),
            ],
            parser_output_ids: vec!["population_structure_report".to_string()],
            declared_outputs: vec![
                artifact("ld_prune_in", "report_tsv", &format!("{prune_prefix}.prune.in")),
                artifact("ld_prune_out", "report_tsv", &format!("{prune_prefix}.prune.out")),
                artifact(
                    "population_pca_eigenvec",
                    "report_tsv",
                    &format!("{pca_prefix}.eigenvec"),
                ),
                artifact(
                    "population_pca_eigenval",
                    "report_tsv",
                    &format!("{pca_prefix}.eigenval"),
                ),
                artifact("prune_log", "log", &format!("{prune_prefix}.log")),
                artifact("pca_log", "log", &format!("{pca_prefix}.log")),
                artifact(
                    "population_structure_report",
                    "report_json",
                    &json_output_path("population_structure_report"),
                ),
            ],
            command_steps: vec![
                step(
                    "ld_prune",
                    "ld_pruning",
                    vec![
                        "plink2",
                        "--vcf",
                        GOVERNED_COHORT_VCF_PATH,
                        "--double-id",
                        "--allow-extra-chr",
                        "--indep-pairwise",
                        "50",
                        "5",
                        "0.2",
                        "--out",
                        &prune_prefix,
                    ],
                    &["ld_prune_in", "ld_prune_out", "prune_log"],
                ),
                step(
                    "population_pca",
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
                        &pca_prefix,
                    ],
                    &["population_pca_eigenvec", "population_pca_eigenval", "pca_log"],
                ),
            ],
        }
    }
}
