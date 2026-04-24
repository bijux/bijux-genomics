use super::{Context, Result, StageResultV1};

pub(super) fn write_stage_resume_contract(
    stage_root: &std::path::Path,
    stage_id: &str,
    execution: &StageResultV1,
    resumed: bool,
) -> Result<()> {
    let mut checksums = serde_json::Map::new();
    for path in &execution.outputs {
        let key = path
            .file_name()
            .and_then(|x| x.to_str())
            .map_or_else(|| path.display().to_string(), std::string::ToString::to_string);
        let value = if path.exists() {
            bijux_dna_infra::hash_file_sha256(path)
                .ok()
                .map_or(serde_json::Value::Null, serde_json::Value::String)
        } else {
            serde_json::Value::Null
        };
        checksums.insert(key, value);
    }
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.stage_resume_contract.v1",
        "stage_id": stage_id,
        "resumed": resumed,
        "exit_code": execution.exit_code,
        "output_count": execution.outputs.len(),
        "outputs_sha256": checksums
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("stage.resume_contract.json"), &payload)
        .context("write stage.resume_contract.json")
}

pub(super) fn write_merge_join_contract(
    stage_root: &std::path::Path,
    execution: &StageResultV1,
    paired_consistent: bool,
) -> Result<()> {
    let expected_files = ["merged.fastq.gz", "unmerged_R1.fastq.gz", "unmerged_R2.fastq.gz"];
    let emitted_names = execution
        .outputs
        .iter()
        .filter_map(|x| x.file_name().and_then(|n| n.to_str()).map(ToString::to_string))
        .collect::<std::collections::BTreeSet<_>>();
    let required_artifacts_present =
        expected_files.iter().all(|name| emitted_names.contains(*name));
    let success = execution.exit_code == 0 && paired_consistent && required_artifacts_present;
    let failure_reason = if success {
        None
    } else if execution.exit_code != 0 {
        Some("merge tool exited non-zero".to_string())
    } else if !paired_consistent {
        Some("paired-end input consistency check failed".to_string())
    } else {
        Some("required merge artifacts missing".to_string())
    };
    let payload = serde_json::json!({
        "schema_version": "bijux.fastq.merge_pairs_join_contract.v1",
        "stage_id": "fastq.merge_pairs",
        "success": success,
        "criteria": {
            "exit_code_zero": execution.exit_code == 0,
            "paired_input_consistent": paired_consistent,
            "outputs_emitted": !execution.outputs.is_empty(),
            "required_artifacts_present": required_artifacts_present,
        },
        "required_artifacts": expected_files,
        "failure_reason": failure_reason,
        "artifacts": execution.outputs,
    });
    bijux_dna_infra::atomic_write_json(&stage_root.join("merge.join_contract.json"), &payload)
        .context("write merge.join_contract.json")
}
