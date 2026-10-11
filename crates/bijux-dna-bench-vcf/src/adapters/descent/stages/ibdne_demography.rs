//! Owner: bijux-dna-bench-vcf
//! descent demography command and output contract.
use super::super::commands::{artifact, path_relative_to_repo, step, StageContract};
use super::super::{materialize_governed_demography_inputs, Path, PathBuf, Result};

pub(crate) fn build(repo_root: &Path, output_root: &str) -> Result<StageContract> {
    Ok({
        let report_json_path = |name: &str| format!("{output_root}/{name}.json");
        let report_tsv_path = |name: &str| format!("{output_root}/{name}.tsv");
        let raw_path = |name: &str, suffix: &str| format!("{output_root}/{name}.{suffix}");
        let (input_ibd_segments_path, genetic_map_path) =
            materialize_governed_demography_inputs(repo_root, &PathBuf::from(output_root))?;
        let input_ibd_segments_path_str =
            path_relative_to_repo(repo_root, &input_ibd_segments_path);
        let genetic_map_path_str = path_relative_to_repo(repo_root, &genetic_map_path);
        let normalized_output_path = report_json_path("demography_report");
        let log_output_path = raw_path("demography_report", "log");
        let trajectory_path = report_tsv_path("ne_trajectory");
        StageContract {
    input_vcf_path: None,
    input_ibd_segments_path: Some(input_ibd_segments_path_str.clone()),
    required_inputs: vec![
                    artifact("ibd_segments", "report_tsv", &input_ibd_segments_path_str),
                    artifact("genetic_map", "reference", &genetic_map_path_str),
                ],
    command_contract_source: "domain/vcf/fixtures/vcf.demography/ibdne.txt".to_string(),
    normalized_output_artifact_id: "demography_report".to_string(),
    normalized_output_path: normalized_output_path.clone(),
    log_output_path: log_output_path.clone(),
    raw_output_ids: vec!["ne_trajectory_tsv".to_string(), "logs_txt".to_string()],
    parser_output_ids: vec!["demography_report".to_string()],
    declared_outputs: vec![
                    artifact("ne_trajectory_tsv", "report_tsv", &trajectory_path),
                    artifact("logs_txt", "log", &log_output_path),
                    artifact("demography_report", "report_json", &normalized_output_path),
                ],
    command_steps: vec![step(
                    "demography",
                    "effective_population_size",
                    vec![
                        "sh".to_string(),
                        "-lc".to_string(),
                        format!(
                            "cat '{input_ibd_segments_path_str}' | ibdne map='{genetic_map_path_str}' out='{output_root}/demography_report' nthreads=8 nboots=0 seed=15052011 > '{log_output_path}' 2>&1"
                        ),
                    ],
                    &["ne_trajectory_tsv", "logs_txt"],
                )],
}
    })
}
