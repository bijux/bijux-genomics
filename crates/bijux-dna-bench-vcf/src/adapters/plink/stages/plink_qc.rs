//! Owner: bijux-dna-bench-vcf
//! plink qc command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.qc/plink.txt".to_string(),
            normalized_metrics_artifact_id: "qc_report".to_string(),
            normalized_metrics_path: json_output_path("qc_report"),
            raw_output_ids: vec![
                "sample_missingness_imiss".to_string(),
                "variant_missingness_lmiss".to_string(),
                "allele_frequency_frq".to_string(),
                "heterozygosity_het".to_string(),
                "hardy_weinberg_hwe".to_string(),
                "plink_log".to_string(),
            ],
            parser_output_ids: vec!["qc_report".to_string()],
            declared_outputs: vec![
                artifact("sample_missingness_imiss", "report_tsv", &raw_path("imiss")),
                artifact("variant_missingness_lmiss", "report_tsv", &raw_path("lmiss")),
                artifact("allele_frequency_frq", "report_tsv", &raw_path("frq")),
                artifact("heterozygosity_het", "report_tsv", &raw_path("het")),
                artifact("hardy_weinberg_hwe", "report_tsv", &raw_path("hwe")),
                artifact("plink_log", "log", &raw_path("log")),
                artifact("qc_report", "report_json", &json_output_path("qc_report")),
            ],
            command_steps: vec![step(
                "qc",
                "quality_control",
                vec![
                    "plink",
                    "--vcf",
                    GOVERNED_COHORT_VCF_PATH,
                    "--double-id",
                    "--allow-extra-chr",
                    "--missing",
                    "--freq",
                    "--het",
                    "--hardy",
                    "--out",
                    output_prefix,
                ],
                &[
                    "sample_missingness_imiss",
                    "variant_missingness_lmiss",
                    "allele_frequency_frq",
                    "heterozygosity_het",
                    "hardy_weinberg_hwe",
                    "plink_log",
                ],
            )],
        }
    }
}
