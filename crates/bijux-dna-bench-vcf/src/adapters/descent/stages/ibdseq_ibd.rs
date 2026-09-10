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
        let output_prefix = format!("{output_root}/ibd_segments");
        let raw_segments_path = format!("{output_prefix}.ibd.gz");
        StageContract {
    input_vcf_path: Some(input_vcf_path.clone()),
    input_ibd_segments_path: None,
    required_inputs: vec![artifact("vcf", "variant", &input_vcf_path)],
    command_contract_source: "domain/vcf/fixtures/vcf.ibd/ibdseq.txt".to_string(),
    normalized_output_artifact_id: "ibd_segments".to_string(),
    normalized_output_path: normalized_output_path.clone(),
    log_output_path: log_output_path.clone(),
    raw_output_ids: vec!["ibdseq_segments_tsv".to_string(), "logs_txt".to_string()],
    parser_output_ids: vec!["ibd_segments".to_string()],
    declared_outputs: vec![
                    artifact("ibdseq_segments_tsv", "report_tsv", &raw_segments_path),
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
                            "ibdseq gt='{input_vcf_path}' out='{output_prefix}' nthreads=8 > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["ibdseq_segments_tsv", "logs_txt"],
                )],
}
    }
}
