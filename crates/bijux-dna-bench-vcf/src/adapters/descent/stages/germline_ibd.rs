//! Owner: bijux-dna-bench-vcf
//! descent ibd command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::GOVERNED_COHORT_VCF_PATH;

pub(crate) fn build(output_root: &str) -> StageContract {
    {
        let report_tsv_path = |name: &str| format!("{output_root}/{name}.tsv");
        let raw_path = |name: &str, suffix: &str| format!("{output_root}/{name}.{suffix}");
        let input_vcf_path = GOVERNED_COHORT_VCF_PATH.to_string();
        let normalized_output_path = report_tsv_path("ibd_segments");
        let log_output_path = raw_path("ibd_segments", "log");
        let cohort_prefix = format!("{output_root}/ibd_segments.cohort");
        let cohort_ped = format!("{cohort_prefix}.ped");
        let cohort_map = format!("{cohort_prefix}.map");
        let germline_match = raw_path("ibd_segments", "match");
        StageContract {
    input_vcf_path: Some(input_vcf_path.clone()),
    input_ibd_segments_path: None,
    required_inputs: vec![artifact("vcf", "variant", &input_vcf_path)],
    command_contract_source: "domain/vcf/fixtures/vcf.ibd/germline.txt".to_string(),
    normalized_output_artifact_id: "ibd_segments".to_string(),
    normalized_output_path: normalized_output_path.clone(),
    log_output_path: log_output_path.clone(),
    raw_output_ids: vec![
                    "cohort_ped".to_string(),
                    "cohort_map".to_string(),
                    "germline_match".to_string(),
                    "logs_txt".to_string(),
                ],
    parser_output_ids: vec!["ibd_segments".to_string()],
    declared_outputs: vec![
                    artifact("cohort_ped", "ped", &cohort_ped),
                    artifact("cohort_map", "map", &cohort_map),
                    artifact("germline_match", "report_tsv", &germline_match),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("ibd_segments", "report_tsv", &normalized_output_path),
                ],
    command_steps: vec![step(
                    "ibd",
                    "relatedness_segments",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "plink2 --vcf '{input_vcf_path}' --double-id --allow-extra-chr --recode ped --out '{cohort_prefix}' >/dev/null 2>&1 && germline -silent -input '{cohort_ped}' '{cohort_map}' -output '{output_root}/ibd_segments' -bits 50 -min_m 1 -err_hom 2 -err_het 0 > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["cohort_ped", "cohort_map", "germline_match", "logs_txt"],
                )],
}
    }
}
