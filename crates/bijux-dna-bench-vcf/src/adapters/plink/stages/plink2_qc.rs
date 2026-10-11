//! Owner: bijux-dna-bench-vcf
//! plink qc command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str, output_prefix: &str) -> StageContract {
    {
        let json_output_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |suffix: &str| format!("{output_prefix}.{suffix}");
        StageContract {
            command_contract_source: "domain/vcf/fixtures/vcf.qc/plink2.txt".to_string(),
            normalized_metrics_artifact_id: "qc_report".to_string(),
            normalized_metrics_path: json_output_path("qc_report"),
            raw_output_ids: vec![
                "sample_missingness_smiss".to_string(),
                "variant_missingness_vmiss".to_string(),
                "allele_frequency_afreq".to_string(),
                "heterozygosity_het".to_string(),
                "hardy_weinberg_hardy".to_string(),
                "plink2_log".to_string(),
            ],
            parser_output_ids: vec!["qc_report".to_string()],
            declared_outputs: vec![
                artifact("sample_missingness_smiss", "report_tsv", &raw_path("smiss")),
                artifact("variant_missingness_vmiss", "report_tsv", &raw_path("vmiss")),
                artifact("allele_frequency_afreq", "report_tsv", &raw_path("afreq")),
                artifact("heterozygosity_het", "report_tsv", &raw_path("het")),
                artifact("hardy_weinberg_hardy", "report_tsv", &raw_path("hardy")),
                artifact("plink2_log", "log", &raw_path("log")),
                artifact("qc_report", "report_json", &json_output_path("qc_report")),
            ],
            command_steps: vec![step(
                "qc",
                "quality_control",
                vec![
                    "plink2",
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
                    "sample_missingness_smiss",
                    "variant_missingness_vmiss",
                    "allele_frequency_afreq",
                    "heterozygosity_het",
                    "hardy_weinberg_hardy",
                    "plink2_log",
                ],
            )],
        }
    }
}
