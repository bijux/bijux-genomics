fn deterministic_stage_list(requested: &[VcfDomainStage]) -> Result<Vec<VcfDomainStage>> {
    if requested.is_empty() {
        return Ok(vec![
            VcfDomainStage::Call,
            VcfDomainStage::Filter,
            VcfDomainStage::Stats,
        ]);
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
                format!(
                    "requested stage {} is not in domain stage order",
                    s.as_str()
                ),
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
        return Err(refusal(
            VcfRefusalCode::InvariantsFailed,
            "sample_name is empty",
        ));
    }
    Ok(())
}

pub fn run_vcf_pipeline(request: &VcfPipelineRequest) -> Result<VcfPipelineResult> {
    validate_request(request)?;
    let stage_list = deterministic_stage_list(&request.requested_stages)?;
    let adna_mode = std::env::var("BIJUX_ADNA_MODE")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"));
    let allow_skip_damage = std::env::var("BIJUX_ALLOW_SKIP_DAMAGE_FILTER")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes"));
    if adna_mode && !allow_skip_damage && !stage_list.contains(&VcfDomainStage::DamageFilter) {
        return Err(refusal(
            VcfRefusalCode::PlanningFailed,
            "aDNA mode requires vcf.damage_filter; set BIJUX_ALLOW_SKIP_DAMAGE_FILTER=1 only with explicit override rationale",
        ));
    }
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
    let provenance_line = build_vcf_provenance_line(&stage_outputs);
    atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.vcf.report.v1",
            "run_output_contract": "docs/10-architecture/CONTRACTS/RUN_OUTPUT.md",
            "report_contract": "docs/30-operations/REPORT_CONTRACT.md",
            "artifact_root": artifact_root,
            "vcf_provenance_line": provenance_line,
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
