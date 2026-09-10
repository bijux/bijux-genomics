//! Owner: bijux-dna-bench-vcf
//! plink admixture command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.admixture/plink.txt".to_string(),
            normalized_metrics_artifact_id: "admixture_report".to_string(),
            normalized_metrics_path: json_output_path("admixture_report"),
            raw_output_ids: vec![
                "bed_matrix".to_string(),
                "bim_index".to_string(),
                "fam_samples".to_string(),
                "allele_frequency_frq".to_string(),
                "sample_missingness_imiss".to_string(),
                "variant_missingness_lmiss".to_string(),
                "plink_log".to_string(),
            ],
            parser_output_ids: vec!["admixture_report".to_string()],
            declared_outputs: vec![
                artifact("bed_matrix", "bed", &raw_path("bed")),
                artifact("bim_index", "bim", &raw_path("bim")),
                artifact("fam_samples", "fam", &raw_path("fam")),
                artifact("allele_frequency_frq", "report_tsv", &raw_path("frq")),
                artifact("sample_missingness_imiss", "report_tsv", &raw_path("imiss")),
                artifact("variant_missingness_lmiss", "report_tsv", &raw_path("lmiss")),
                artifact("plink_log", "log", &raw_path("log")),
                artifact("admixture_report", "report_json", &json_output_path("admixture_report")),
            ],
            command_steps: vec![step(
                "admixture_prep",
                "cohort_prep",
                vec![
                    "plink",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--missing",
                    "--freq",
                    "--make-bed",
                    "--out",
                    output_prefix,
                ],
                &[
                    "bed_matrix",
                    "bim_index",
                    "fam_samples",
                    "allele_frequency_frq",
                    "sample_missingness_imiss",
                    "variant_missingness_lmiss",
                    "plink_log",
                ],
            )],
        }
    }
}
