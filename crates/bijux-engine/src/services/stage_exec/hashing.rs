fn hash_inputs(inputs: &[PathBuf]) -> Result<String> {
    if inputs.is_empty() {
        return Ok("none".to_string());
    }
    let mut hashes = Vec::new();
    for input in inputs {
        hashes.push(hash_file_sha256(input)?);
    }
    Ok(hashes.join(","))
}

fn hash_outputs(outputs: &[PathBuf]) -> Result<Vec<String>> {
    let mut hashes = Vec::new();
    for output in outputs {
        if output.is_file() {
            hashes.push(hash_file_sha256(output)?);
        }
    }
    Ok(hashes)
}

fn is_retention_stage(stage_id: &str) -> bool {
    bijux_stages_fastq::fastq::registry()
        .iter()
        .find(|stage| stage.id == stage_id)
        .is_some_and(|stage| stage.affects_read_counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_core::{
        CommandSpecV1, ContainerImageRefV1, StageIO, StageId, StageVersion, ToolConstraints, ToolId,
    };

    #[test]
    fn polyx_warning_is_stage_wide() {
        let plan = StagePlanV1 {
            stage_id: StageId("fastq.trim".to_string()),
            stage_version: StageVersion(1),
            tool_id: ToolId("cutadapt".to_string()),
            tool_version: "1.0.0".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/test:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: Vec::new(),
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: Vec::new(),
                outputs: Vec::new(),
            },
            out_dir: std::path::PathBuf::from("out"),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({
                "paired_mode": "single_end",
                "threads": 1,
                "min_len": 0,
                "adapter_policy": "none"
            }),
            aux_images: std::collections::BTreeMap::new(),
        };
        let params = serde_json::json!({
            "polyx_bank": {
                "preset": "illumina_twocolor"
            }
        });
        let warnings = warnings_for_plan(&plan, &params);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("polyx preset requested"));
    }
}
