pub(crate) fn write_run_manifest(
    out_dir: &Path,
    stage_runs: &[StageExecutionSummary],
    failures: &[bijux_dna_planner_fastq::stage_api::RawFailure],
) -> Result<()> {
    let run_id = stage_runs
        .first()
        .map(|entry| entry.result.run_id.clone())
        .unwrap_or_default();
    let stages: Vec<serde_json::Value> = stage_runs
        .iter()
        .map(|entry| {
            let artifacts_dir =
                bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
            let mut artifacts = Vec::new();
            let add_artifact = |artifacts: &mut Vec<serde_json::Value>, name: &str, path: &Path| {
                if path.exists() {
                    if let Ok(hash) = bijux_dna_infra::hash_file_sha256(path) {
                        artifacts.push(serde_json::json!({
                            "name": name,
                            "path": relative_path_string(out_dir, path),
                            "sha256": hash,
                        }));
                    }
                }
            };
            add_artifact(
                &mut artifacts,
                "metrics_envelope",
                &artifacts_dir.join("metrics_envelope.json"),
            );
            add_artifact(
                &mut artifacts,
                "metrics",
                &artifacts_dir.join("metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_metrics",
                &artifacts_dir.join("stage_metrics.json"),
            );
            add_artifact(
                &mut artifacts,
                "stage_report",
                &artifacts_dir.join("stage_report.json"),
            );
            add_artifact(
                &mut artifacts,
                "effective_config",
                &artifacts_dir.join("effective_config.json"),
            );
            add_artifact(
                &mut artifacts,
                "retention_report",
                &artifacts_dir
                    .join("reports")
                    .join(format!("{}.retention.json", entry.plan.step_id.0)),
            );
            add_artifact(
                &mut artifacts,
                "telemetry_events",
                &artifacts_dir.join("telemetry").join("events.jsonl"),
            );
            let mut primary_outputs = Vec::new();
            if entry.plan.step_id.as_str() == STAGE_QC_POST.as_str() {
                let qc_report = artifacts_dir
                    .join("reports")
                    .join(format!("{}.qc_post_report.json", entry.plan.step_id.0));
                if qc_report.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &qc_report));
                }
                let multiqc_report = entry.plan.out_dir.join("multiqc_report.html");
                if multiqc_report.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &multiqc_report));
                }
                let multiqc_data = entry.plan.out_dir.join("multiqc_data");
                if multiqc_data.exists() {
                    primary_outputs.push(relative_path_string(out_dir, &multiqc_data));
                }
            }
            let contract_hash = stage_contract_hash_for(entry.plan.step_id.as_str());
            serde_json::json!({
                "stage_id": entry.plan.step_id.0,
                "stage_contract_hash": contract_hash,
                "tool_id": entry.plan.image.image,
                "artifacts": artifacts,
                "primary_outputs": primary_outputs,
            })
        })
        .collect();
    let failures_json: Vec<serde_json::Value> = failures
        .iter()
        .map(|failure| {
            serde_json::json!({
                "stage": failure.stage,
                "tool": failure.tool,
                "reason": failure.reason,
                "category": format!("{:?}", failure.category),
            })
        })
        .collect();
    let defaults_path = out_dir.join("defaults_ledger.json");
    let defaults_hash =
        bijux_dna_infra::hash_file_sha256(&defaults_path).context("hash defaults_ledger.json")?;
    let defaults_payload = read_json_if_exists(&defaults_path);
    let pipeline_id = defaults_payload
        .as_ref()
        .and_then(|value| value.get("pipeline_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| std::env::var("BIJUX_PIPELINE_ID").ok())
        .unwrap_or_else(|| "unknown".to_string());
    let profile_id = defaults_payload
        .as_ref()
        .and_then(|value| value.get("profile_id"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .or_else(|| std::env::var("BIJUX_PROFILE_ID").ok())
        .unwrap_or_else(|| "unknown".to_string());
    let graph_hash = std::env::var("BIJUX_PLAN_HASH").ok().unwrap_or_else(|| {
        let steps = stage_runs.iter().map(|entry| entry.plan.clone()).collect();
        ExecutionGraph::new(
            pipeline_id.clone(),
            "unknown",
            PlanPolicy::PreferAccuracy,
            steps,
            Vec::new(),
        )
        .and_then(|graph| graph.hash())
        .unwrap_or_else(|_| "unknown".to_string())
    });
    let toolchain_versions = serde_json::json!({
        "planner": std::env::var("BIJUX_PLANNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        "engine": std::env::var("BIJUX_ENGINE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
        "runner": std::env::var("BIJUX_RUNNER_VERSION").unwrap_or_else(|_| "unknown".to_string()),
    });
    let mut dataset_fingerprints = Vec::new();
    for entry in stage_runs {
        let metrics_path =
            bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("metrics_envelope.json");
        if let Some(value) = read_json_if_exists(&metrics_path) {
            if let Some(hash) = value
                .get("input_fingerprint")
                .and_then(serde_json::Value::as_str)
            {
                dataset_fingerprints.push(hash.to_string());
            }
        }
    }
    dataset_fingerprints.sort();
    dataset_fingerprints.dedup();
    let output_artifacts: Vec<serde_json::Value> = stage_runs
        .iter()
        .flat_map(|entry| {
            entry.plan.io.outputs.iter().map(|artifact| {
                let sha256 = artifact
                    .path
                    .exists()
                    .then(|| bijux_dna_infra::hash_file_sha256(&artifact.path).ok())
                    .flatten();
                serde_json::json!({
                    "stage_id": entry.plan.step_id.0,
                    "name": artifact.name,
                    "role": artifact.role.as_str(),
                    "optional": artifact.optional,
                    "path": relative_path_string(out_dir, &artifact.path),
                    "sha256": sha256,
                })
            })
        })
        .collect::<Vec<_>>();
    let root = bijux_dna_runtime::recording::run_artifacts_dir_for_out(out_dir);
    let mut output_artifacts = output_artifacts;
    for (name, role, path) in [
        (
            "summary",
            bijux_dna_core::contract::ArtifactRole::SummaryJson,
            root.join("summary.json"),
        ),
        (
            "summary_tsv",
            bijux_dna_core::contract::ArtifactRole::SummaryTsv,
            root.join("summary.tsv"),
        ),
        (
            "report_html",
            bijux_dna_core::contract::ArtifactRole::ReportHtml,
            root.join("report.html"),
        ),
    ] {
        let sha256 = path
            .exists()
            .then(|| bijux_dna_infra::hash_file_sha256(&path).ok())
            .flatten();
        output_artifacts.push(serde_json::json!({
            "stage_id": "report.aggregate",
            "name": name,
            "role": role.as_str(),
            "optional": false,
            "path": relative_path_string(out_dir, &path),
            "sha256": sha256,
        }));
    }
    let run_provenance = run_provenance_from_stage_runs(out_dir, stage_runs);
    let tool_invocations: Vec<bijux_dna_core::metrics::ToolInvocationV1> = stage_runs
        .iter()
        .filter_map(|entry| {
            let path = bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                .join("tool_invocation.json");
            let raw = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str(&raw).ok()
        })
        .collect();
    let tool_provenance = serde_json::json!({
        "schema_version": "bijux.tool_provenance.v1",
        "tools": tool_invocations.iter().map(|inv| {
            serde_json::json!({
                "stage_id": inv.stage_id,
                "tool_id": inv.tool_id,
                "image_digest": inv.image_digest,
                "resolved_tool_version": inv.resolved_tool_version,
                "executed_command": inv.executed_command,
            })
        }).collect::<Vec<_>>()
    });
    bijux_dna_infra::atomic_write_json(&out_dir.join("tool_provenance.json"), &tool_provenance)
        .context("write tool_provenance.json")?;
    let cache_key = {
        let params_hash = run_provenance
            .get("params_hash")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let input_hashes = run_provenance
            .get("input_hashes")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(std::string::ToString::to_string))
            .collect::<Vec<_>>();
        let tool_version = run_provenance
            .get("tool_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let env_digest = run_provenance
            .get("tool_image_digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        bijux_dna_core::prelude::CacheKey::new(
            bijux_dna_core::prelude::input_fingerprint(&input_hashes),
            params_hash,
            tool_version,
            env_digest,
        )
    };
    let manifest = serde_json::json!({
        "schema_version": "bijux.run_manifest.v3",
        "contract_version": bijux_dna_core::contract::ContractVersion::v1(),
        "run_id": run_id,
        "pipeline_id": pipeline_id,
        "profile_id": profile_id,
        "graph_hash": graph_hash,
        "cache_key": cache_key,
        "toolchain_versions": toolchain_versions,
        "dataset_fingerprints": dataset_fingerprints,
        "tool_invocations": tool_invocations,
        "tool_provenance": "tool_provenance.json",
        "output_artifacts": output_artifacts,
        "stages": stages,
        "failures": failures_json,
        "defaults_ledger": relative_path_string(out_dir, &defaults_path),
        "defaults_ledger_sha256": defaults_hash,
        "run_provenance": run_provenance,
        "execution_replay_identity": {
            "tool_image_ref": tool_invocations
                .first()
                .map_or_else(|| "unknown".to_string(), |inv| inv.tool_id.to_string()),
            "tool_image_digest": tool_invocations
                .first()
                .map_or_else(|| "unknown".to_string(), |inv| inv.image_digest.clone()),
            "tool_version_output": tool_invocations
                .first()
                .and_then(|inv| inv.resolved_tool_version.clone())
                .unwrap_or_else(|| "unknown".to_string()),
        },
        "telemetry": {
            "events": stage_runs.iter().map(|entry| {
                relative_path_string(
                    out_dir,
                    &bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir)
                        .join("telemetry")
                        .join("events.jsonl"),
                )
            }).collect::<Vec<_>>()
        }
    });
    let path = out_dir.join("run_manifest.json");
    let payload = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)?;
    bijux_dna_infra::atomic_write_bytes(&path, payload.as_slice())
        .context("write run_manifest.json")?;
    bijux_dna_runtime::recording::write_profile_and_lock_manifests(&path)?;
    let manifest_hash =
        bijux_dna_infra::hash_file_sha256(&path).context("hash run_manifest.json")?;
    for entry in stage_runs {
        let artifacts_dir =
            bijux_dna_runtime::recording::run_artifacts_dir_for_out(&entry.plan.out_dir);
        backfill_metric_manifest_hash(
            &artifacts_dir.join("metrics_envelope.json"),
            &manifest_hash,
        )?;
        backfill_metric_manifest_hash(&artifacts_dir.join("stage_metrics.json"), &manifest_hash)?;
    }
    Ok(())
}

fn backfill_metric_manifest_hash(path: &Path, manifest_hash: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(path)?;
    let mut value: serde_json::Value = serde_json::from_str(&raw)?;
    let mut changed = false;
    if let Some(obj) = value.as_object_mut() {
        let metric_provenance = obj
            .entry("metric_provenance")
            .or_insert_with(|| serde_json::json!({}));
        if let Some(prov_obj) = metric_provenance.as_object_mut() {
            if !prov_obj.contains_key("manifest_hash") {
                prov_obj.insert(
                    "manifest_hash".to_string(),
                    serde_json::Value::String(manifest_hash.to_string()),
                );
                changed = true;
            }
        }
    }
    if changed {
        bijux_dna_infra::atomic_write_json(path, &value)?;
    }
    Ok(())
}

