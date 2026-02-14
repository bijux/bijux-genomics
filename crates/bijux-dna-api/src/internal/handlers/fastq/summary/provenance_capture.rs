pub(crate) fn write_scientific_provenance(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
) -> Result<()> {
    let defaults_path = out_dir.join("defaults_ledger.json");
    let (pipeline_id, planner_version) = if defaults_path.exists() {
        let raw = fs::read_to_string(&defaults_path)?;
        let value: serde_json::Value = serde_json::from_str(&raw)?;
        let pipeline_id = value
            .get("pipeline_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let planner_version =
            std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string());
        (pipeline_id, planner_version)
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };
    let mut invocations = Vec::new();
    let mut parameters_fingerprints = std::collections::BTreeMap::new();
    for entry in stage_runs {
        let artifacts_dir =
            bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
        let invocation_path = artifacts_dir
            .join("invocations")
            .join(format!("{}.tool_invocation.json", entry.plan.step_id.0));
        if invocation_path.exists() {
            let raw = fs::read_to_string(&invocation_path)?;
            let invocation: ToolInvocationV1 = serde_json::from_str(&raw)?;
            let key = format!("{}:{}", invocation.stage_id, invocation.tool_id);
            let metrics_path = artifacts_dir.join("metrics_envelope.json");
            if metrics_path.exists() {
                let metrics_raw = fs::read_to_string(&metrics_path)?;
                if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&metrics_raw) {
                    if let Some(params_hash) = metrics
                        .get("parameters_fingerprint")
                        .and_then(|v| v.as_str())
                    {
                        parameters_fingerprints.insert(key, params_hash.to_string());
                    }
                }
            }
            invocations.push(invocation);
        }
    }
    let provenance = bijux_dna_runtime::provenance::build_scientific_provenance(
        pipeline_id,
        planner_version,
        &parameters_fingerprints,
        &invocations,
    );
    bijux_dna_runtime::recording::write_scientific_provenance(out_dir, &provenance)?;
    Ok(())
}

fn read_json_if_exists(path: &Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn run_provenance_from_stage_runs(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
) -> serde_json::Value {
    let mut params_by_stage = std::collections::BTreeMap::new();
    let mut input_hashes = Vec::new();
    let mut tool_versions = std::collections::BTreeSet::new();
    let mut image_digests = std::collections::BTreeSet::new();
    for entry in stage_runs {
        tool_versions.insert("unknown".to_string());
        if let Some(digest) = entry.plan.image.digest.clone() {
            image_digests.insert(digest);
        }
        let envelope_path =
            bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("metrics_envelope.json");
        if let Some(value) = read_json_if_exists(&envelope_path) {
            if let Some(hash) = value
                .get("input_fingerprint")
                .and_then(serde_json::Value::as_str)
            {
                input_hashes.push(hash.to_string());
            }
            if let Some(hash) = value
                .get("parameters_fingerprint")
                .and_then(serde_json::Value::as_str)
            {
                params_by_stage.insert(entry.plan.step_id.to_string(), hash.to_string());
            }
        }
    }
    input_hashes.sort();
    input_hashes.dedup();
    let params_hash =
        params_hash(&serde_json::json!(params_by_stage)).unwrap_or_else(|_| "unknown".to_string());
    let tool_version = if tool_versions.len() == 1 {
        tool_versions
            .into_iter()
            .next()
            .unwrap_or_else(|| "unknown".to_string())
    } else {
        "multiple".to_string()
    };
    let tool_image_digest = if image_digests.len() == 1 {
        image_digests.into_iter().next()
    } else {
        None
    };
    let pipeline_id = std::env::var("BIJUX_PIPELINE_ID").unwrap_or_else(|_| "unknown".to_string());
    let git_commit = std::env::var("BIJUX_GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string());
    let build_profile =
        std::env::var("BIJUX_BUILD_PROFILE").unwrap_or_else(|_| "unknown".to_string());
    let reference_genome = std::env::var("BIJUX_REFERENCE_GENOME").ok();
    let plan_hash = std::env::var("BIJUX_PLAN_HASH").ok();
    let workspace_root = out_dir.parent().and_then(Path::parent).unwrap_or(out_dir);
    let adapter_bank_hash = hash_optional(
        &workspace_root
            .join("assets")
            .join("adapters")
            .join("bank.v1.yaml"),
    );
    let reference_bank_hash = hash_optional(
        &workspace_root
            .join("assets")
            .join("references")
            .join("bank.v1.yaml"),
    );
    let contamination_db_bank_hash = hash_optional(
        &workspace_root
            .join("assets")
            .join("contaminants")
            .join("db_bank.v1.yaml"),
    );
    serde_json::json!({
        "schema_version": "bijux.run_provenance.v1",
        "tool_image_digest": tool_image_digest,
        "tool_version": tool_version,
        "params_hash": params_hash,
        "input_hashes": input_hashes,
        "reference_genome": reference_genome,
        "pipeline_id": pipeline_id,
        "git_commit": git_commit,
        "build_profile": build_profile,
        "plan_hash": plan_hash,
        "bank_hashes": {
            "adapter_bank_hash": adapter_bank_hash,
            "reference_bank_hash": reference_bank_hash,
            "contamination_db_bank_hash": contamination_db_bank_hash,
            "taxonomy_db_hash": contamination_db_bank_hash,
        },
        "contamination_db_version": "v1",
        "taxonomy_db_version": "v1",
    })
}

