fn deterministic_stage_list(requested: &[VcfDomainStage]) -> Result<Vec<VcfDomainStage>> {
    if requested.is_empty() {
        return Ok(vec![VcfDomainStage::Call, VcfDomainStage::Filter, VcfDomainStage::Stats]);
    }
    let req = requested.to_vec();
    let ordered = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .copied()
        .filter(|s| req.iter().any(|r| r == s))
        .collect::<Vec<_>>();
    if ordered.is_empty() {
        return Err(refusal(
            VcfRefusalCode::PlanningFailed,
            "requested stages resolved to empty deterministic order",
        ));
    }
    for s in requested {
        if !VCF_STAGE_ORDER_DOWNSTREAM.contains(s) {
            return Err(refusal(
                VcfRefusalCode::UnsupportedStage,
                format!("requested stage {} is not in domain stage order", s.as_str()),
            ));
        }
    }
    Ok(ordered)
}

fn validate_request(req: &VcfPipelineRequest) -> Result<()> {
    if !req.input_vcf.exists() {
        return Err(refusal(
            VcfRefusalCode::InvariantsFailed,
            format!("input VCF does not exist: {}", req.input_vcf.display()),
        ));
    }
    if req.sample_name.trim().is_empty() {
        return Err(refusal(VcfRefusalCode::InvariantsFailed, "sample_name is empty"));
    }
    Ok(())
}

fn verify_contract_surface(result: &VcfPipelineResult) -> Result<()> {
    fn expected_stage_artifacts(stage_id: &str) -> &'static [&'static str] {
        match stage_id {
            "vcf.call" | "vcf.call_gl" | "vcf.call_diploid" | "vcf.call_pseudohaploid" => &[
                "call_metrics.json",
                "call_metrics.tsv",
                "call_manifest.json",
            ],
            "vcf.filter" => &[
                "filtered.vcf.gz",
                "filtered.vcf.gz.tbi",
                "filter_breakdown.json",
                "filter_breakdown.tsv",
            ],
            "vcf.stats" => &["bcftools_stats.txt", "stats.json"],
            "vcf.postprocess" => &[
                "postprocess.vcf.gz",
                "postprocess.vcf.gz.tbi",
                "validate_outputs.json",
                "final_manifest.json",
            ],
            _ => &[],
        }
    }

    for stage in &result.stages {
        if !stage
            .artifact_dir
            .starts_with(result.artifact_root.join(""))
        {
            return Err(refusal(
                VcfRefusalCode::ContractViolation,
                format!("artifact root violation for {}", stage.stage_id),
            ));
        }
        for required in ["stage_manifest.json", "stdout.log", "stderr.log", "command.txt", "env.txt"] {
            let p = stage.artifact_dir.join(required);
            if !p.exists() {
                return Err(refusal(
                    VcfRefusalCode::ContractViolation,
                    format!("missing stage sidecar {}", p.display()),
                ));
            }
        }
        for required in expected_stage_artifacts(&stage.stage_id) {
            let p = stage.artifact_dir.join(required);
            if !p.exists() {
                return Err(refusal(
                    VcfRefusalCode::ContractViolation,
                    format!("stage {} missing required artifact {}", stage.stage_id, p.display()),
                ));
            }
        }
    }
    if !result.report_path.exists() {
        return Err(refusal(
            VcfRefusalCode::ContractViolation,
            "missing report.json",
        ));
    }
    Ok(())
}

fn read_json(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<serde_json::Value>(&raw).ok()
}

fn maybe_bam_handoff(request: &VcfPipelineRequest, artifact_root: &std::path::Path) -> Result<Option<serde_json::Value>> {
    let candidates = [
        request.run_root.join("artifacts").join("bam").join("bam_metrics.json"),
        request.run_root.join("artifacts").join("bam").join("metrics.json"),
        request.run_root.join("artifacts").join("bam").join("summary.json"),
    ];
    let bam_metrics = candidates.iter().find_map(|p| read_json(p));
    let Some(bam_metrics) = bam_metrics else {
        return Ok(None);
    };
    let handoff = serde_json::json!({
        "schema_version": "bijux.vcf.handoff.bam_to_vcf.v1",
        "source_domain": "bam",
        "target_domain": "vcf",
        "signals": {
            "damage_ct_ga_rate": bam_metrics.get("damage_ct_ga_rate").cloned().unwrap_or(serde_json::Value::Null),
            "mean_depth": bam_metrics.get("mean_depth").cloned().unwrap_or_else(|| bam_metrics.get("depth_mean").cloned().unwrap_or(serde_json::Value::Null)),
            "udg_treated": bam_metrics.get("udg_treated").cloned().unwrap_or(serde_json::Value::Null),
        },
        "decision_inputs": {
            "damage_filter_considered": true,
            "regime_detection_considered": true,
        },
    });
    let out = artifact_root.join("handoff.bam_to_vcf.json");
    atomic_write_json(&out, &handoff)?;
    Ok(Some(handoff))
}

fn write_runtime_explain(
    preflight: &VcfPreflightResult,
    artifact_root: &std::path::Path,
    stages: &[VcfStageOutputs],
    handoff: Option<serde_json::Value>,
) -> Result<()> {
    let preflight_details = serde_json::json!({
        "input_regime": preflight.regime.regime,
        "lowcov_likelihood_hint": preflight.regime.lowcov_likelihood_hint,
        "pseudohaploid_hint": preflight.regime.pseudohaploid_hint,
    });

    let mut backend = serde_json::Value::Null;
    let mut chunk_strategy = serde_json::Value::Null;
    let mut panel_lock = serde_json::Value::Null;
    let mut map_lock = serde_json::Value::Null;
    for stage in stages {
        let imputation_manifest = stage.artifact_dir.join("imputation_manifest.json");
        if backend.is_null() && imputation_manifest.exists() {
            if let Some(json) = read_json(&imputation_manifest) {
                backend = json.get("backend").cloned().unwrap_or(serde_json::Value::Null);
                chunk_strategy = json
                    .get("chunk_plan")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                panel_lock = serde_json::json!({
                    "panel_id": json.get("panel_id").cloned().unwrap_or(serde_json::Value::Null),
                    "panel_checksums": json.get("panel_checksums").cloned().unwrap_or(serde_json::json!([])),
                });
                map_lock = json.get("map").cloned().unwrap_or(serde_json::Value::Null);
            }
        }
        let panel_manifest = stage.artifact_dir.join("panel_manifest.json");
        if panel_lock.is_null() && panel_manifest.exists() {
            if let Some(json) = read_json(&panel_manifest) {
                panel_lock = serde_json::json!({
                    "panel_id": json.get("panel").and_then(|p| p.get("id")).cloned().unwrap_or(serde_json::Value::Null),
                    "panel_version": json.get("panel").and_then(|p| p.get("version")).cloned().unwrap_or(serde_json::Value::Null),
                });
                map_lock = serde_json::json!({
                    "map_id": json.get("map").and_then(|m| m.get("id")).cloned().unwrap_or(serde_json::Value::Null),
                    "map_version": json.get("map").and_then(|m| m.get("version")).cloned().unwrap_or(serde_json::Value::Null),
                });
            }
        }
    }

    let explain = serde_json::json!({
        "schema_version": "bijux.vcf.runtime_explain.v1",
        "regime_decision": preflight_details,
        "backend_selection": {
            "selected_backend": backend,
            "reason": "selected from executed stage manifests and input regime constraints",
        },
        "panel_map_lock_ids": {
            "panel_lock": panel_lock,
            "map_lock": map_lock,
        },
        "chunk_strategy": chunk_strategy,
        "handoff": handoff.unwrap_or(serde_json::json!({})),
    });
    atomic_write_json(&artifact_root.join("explain.json"), &explain)?;
    Ok(())
}

pub fn run_vcf_pipeline(request: &VcfPipelineRequest) -> Result<VcfPipelineResult> {
    validate_request(request)?;
    let stage_list = deterministic_stage_list(&request.requested_stages)?;
    let artifact_root = request.run_root.join("artifacts").join("vcf");
    bijux_dna_infra::ensure_dir(&artifact_root)?;
    let preflight = run_vcf_preflight(
        &request.input_vcf,
        &artifact_root.join("validate_inputs"),
        &request.species_context,
        &request.invariants,
    )
    .map_err(|err| refusal(VcfRefusalCode::InvariantsFailed, err.to_string()))?;
    let ctx = VcfStageRunContext {
        request,
        artifact_root: artifact_root.clone(),
        preflight: &preflight,
    };

    let mut current = preflight.normalized_input.clone();
    let mut stage_outputs = Vec::<VcfStageOutputs>::new();
    for stage in stage_list {
        let runner = DispatchRunner { stage };
        let out = runner.run(&ctx, &current)?;
        if let Some(primary) = &out.primary_output {
            current = primary.clone();
        }
        stage_outputs.push(out);
    }
    let handoff = maybe_bam_handoff(request, &artifact_root)?;
    write_runtime_explain(&preflight, &artifact_root, &stage_outputs, handoff)?;

    let report_path = request.run_root.join("report.json");
    atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.vcf.report.v1",
            "run_output_contract": "docs/10-architecture/CONTRACTS/RUN_OUTPUT.md",
            "report_contract": "docs/30-operations/REPORT_CONTRACT.md",
            "artifact_root": artifact_root,
            "stages": stage_outputs,
        }),
    )?;

    let result = VcfPipelineResult {
        run_root: request.run_root.clone(),
        artifact_root,
        stages: stage_outputs,
        report_path,
        preflight,
    };
    verify_contract_surface(&result)?;
    Ok(result)
}
