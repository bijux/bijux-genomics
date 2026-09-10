//! Owner: bijux-dna-bench-vcf
//! descent roh command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str) -> StageContract {
    {
        let report_json_path = |name: &str| format!("{output_root}/{name}.json");
        let raw_path = |name: &str, suffix: &str| format!("{output_root}/{name}.{suffix}");
        let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
        let normalized_output_path = report_json_path("roh_report");
        let log_output_path = raw_path("roh_report", "log");
        StageContract {
            input_vcf_path: Some(input_vcf_path.clone()),
            input_ibd_segments_path: None,
            required_inputs: vec![artifact("vcf", "variant", &input_vcf_path)],
            command_contract_source: "domain/vcf/fixtures/vcf.roh/plink2.txt".to_string(),
            normalized_output_artifact_id: "roh_report".to_string(),
            normalized_output_path: normalized_output_path.clone(),
            log_output_path: log_output_path.clone(),
            raw_output_ids: vec!["roh_hom".to_string(), "logs_txt".to_string()],
            parser_output_ids: vec!["roh_report".to_string()],
            declared_outputs: vec![
                artifact("roh_hom", "report_tsv", &raw_path("roh_report", "hom")),
                artifact("logs_txt", "log", &log_output_path),
                artifact("roh_report", "report_json", &normalized_output_path),
            ],
            command_steps: vec![step(
                "roh",
                "runs_of_homozygosity",
                vec![
                    "plink2".to_string(),
                    "--vcf".to_string(),
                    GOVERNED_COHORT_VCF_PATH.to_string(),
                    "--double-id".to_string(),
                    "--allow-extra-chr".to_string(),
                    "--homozyg".to_string(),
                    "--out".to_string(),
                    format!("{output_root}/roh_report"),
                ],
                &["roh_hom", "logs_txt"],
            )],
        }
    }
}
