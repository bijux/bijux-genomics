//! Owner: bijux-dna-bench-vcf
//! descent ibd command and output contract.
use super::super::commands::{artifact, step, StageContract};
use super::super::{GOVERNED_IBDHAP_MAP_PATH, GOVERNED_IBDHAP_VCF_PATH};

pub(crate) fn build(output_root: &str) -> StageContract {
    {
        let report_tsv_path = |name: &str| format!("{output_root}/{name}.tsv");
        let raw_path = |name: &str, suffix: &str| format!("{output_root}/{name}.{suffix}");
        let input_vcf_path = GOVERNED_IBDHAP_VCF_PATH.to_string();
        let genetic_map_path = GOVERNED_IBDHAP_MAP_PATH.to_string();
        let normalized_output_path = report_tsv_path("ibd_segments");
        let log_output_path = raw_path("ibd_state_probabilities", "log");
        let scores_path = report_tsv_path("ibd_state_probabilities");
        let identifiers_path = report_tsv_path("ibd_haplotype_identifiers");
        let work_dir = format!("{output_root}/work");
        StageContract {
    input_vcf_path: Some(input_vcf_path.clone()),
    input_ibd_segments_path: None,
    required_inputs: vec![
                    artifact("vcf", "variant", &input_vcf_path),
                    artifact("genetic_map", "reference", &genetic_map_path),
                ],
    command_contract_source: "domain/vcf/fixtures/vcf.ibd/ibdhap.txt".to_string(),
    normalized_output_artifact_id: "ibd_segments".to_string(),
    normalized_output_path: normalized_output_path.clone(),
    log_output_path: log_output_path.clone(),
    raw_output_ids: vec![
                    "ibdhap_state_probabilities_tsv".to_string(),
                    "ibdhap_haplotype_identifiers_tsv".to_string(),
                    "logs_txt".to_string(),
                ],
    parser_output_ids: vec!["ibd_segments".to_string()],
    declared_outputs: vec![
                    artifact("ibdhap_state_probabilities_tsv", "report_tsv", &scores_path),
                    artifact("ibdhap_haplotype_identifiers_tsv", "report_tsv", &identifiers_path),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("ibd_segments", "report_tsv", &normalized_output_path),
                ],
    command_steps: vec![step(
                    "ibd",
                    "ibd_state_probabilities",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "ibdhap --vcf '{input_vcf_path}' --out '{normalized_output_path}' --genetic-map '{genetic_map_path}' --sample sample_a --sample sample_b --region chr1:2-10 --max-markers 500 --work-dir '{work_dir}' --scores '{scores_path}' --identifiers '{identifiers_path}' > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &[
                        "ibdhap_state_probabilities_tsv",
                        "ibdhap_haplotype_identifiers_tsv",
                        "logs_txt",
                        "ibd_segments",
                    ],
                )],
}
    }
}
