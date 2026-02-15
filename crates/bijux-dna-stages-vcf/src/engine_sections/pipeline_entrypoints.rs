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
