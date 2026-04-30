fn write_alignment_regime_validation(
    stage_dir: &Path,
    regime: AlignmentRegime,
    tool_id: &str,
    step: &bijux_dna_core::contract::ExecutionStep,
) -> Result<()> {
    let command_line = step.command.template.join(" ");
    let expected_flags: Vec<&str> = match (tool_id, regime) {
        ("bwa", AlignmentRegime::Adna) => vec!["bwa aln", "-l 1024", "-n 0.01"],
        ("bwa", AlignmentRegime::Modern) => vec!["bwa mem"],
        ("bowtie2", AlignmentRegime::Adna) => vec!["--very-sensitive-local", "-N 1", "-L 20"],
        ("bowtie2", AlignmentRegime::Edna) => vec!["--very-sensitive-local", "-N 1", "-L 22"],
        ("bowtie2", AlignmentRegime::Modern) => vec!["--very-sensitive"],
        _ => Vec::new(),
    };
    let missing = expected_flags
        .iter()
        .filter(|flag| !command_line.contains(**flag))
        .map(|flag| (*flag).to_string())
        .collect::<Vec<_>>();
    let valid = missing.is_empty();
    let path = stage_dir.join("alignment_regime_validation.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "schema_version": "bijux.bam.alignment_regime_validation.v1",
            "tool": tool_id,
            "regime": regime,
            "expected_flags": expected_flags,
            "missing_flags": missing,
            "valid": valid,
        }),
    )
    .with_context(|| format!("write {}", path.display()))?;
    if !valid {
        return Err(anyhow!(
            "bam.alignment_regime_validation failed for tool={tool_id:?} regime={regime:?}; see {}",
            path.display()
        ));
    }
    Ok(())
}

fn write_duplicate_policy_split(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    let umi_policy = plan
        .params
        .get("umi_policy")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("ignore");
    let duplicate_action = plan
        .params
        .get("duplicate_action")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("mark");
    let library_type = plan
        .params
        .get("library_type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("dsdna");
    let selected = match umi_policy {
        "collapse" => "bam.collapse",
        "use_tag" => "bam.umi_dedup",
        _ => "bam.markdup",
    };
    let path = stage_dir.join("duplicate_policy_split.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "schema_version": "bijux.bam.duplicate_policy_split.v1",
            "selected_executor": selected,
            "library_type": library_type,
            "duplicate_action": duplicate_action,
            "optical_duplicates": plan.params.get("optical_duplicates").cloned(),
            "library_policy": {
                "dsdna": "markdup interprets duplicates as PCR/optical duplicates in double-stranded libraries",
                "ssdna": "markdup is conservative for single-stranded libraries; downstream authenticity metrics should be consulted",
            },
            "modes": {
                "bam.markdup": {
                    "supported": true,
                    "description": "PCR/optical duplicate marking or removal"
                },
                "bam.collapse": {
                    "supported": false,
                    "reason_code": "BAM_COLLAPSE_NOT_ENABLED",
                    "issue_id": "BAM-4B-81"
                },
                "bam.umi_dedup": {
                    "supported": true,
                    "description": "UMI-aware deduplication via tag-aware policy"
                }
            }
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn run_bam_truth_stage<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
    rg_policy_override: Option<&str>,
    out_dir: &Path,
) -> Result<StageExecutionSummary> {
    enforce_stage_refusal_rules(stage, bam_path, bai_path, reference, rg_policy_override)?;
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Recalibration
        && profile.id.as_str().to_ascii_lowercase().contains("adna")
    {
        return Err(anyhow!(
            "bam.recalibration refusal: modern-DNA-only by default; select a modern profile"
        ));
    }
    let stage_key = bijux_dna_core::ids::parse_stage_id(stage.as_str()).unwrap_or_else(|_| bijux_dna_core::ids::StageId::new(stage.as_str()));
    let tool_id = profile
        .defaults
        .tools
        .get(&stage_key)
        .cloned()
        .unwrap_or_else(|| bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage));
    let spec = build_tool_execution_spec(
        stage.as_str(),
        tool_id.as_str(),
        registry_core,
        catalog,
        platform,
    )?;

    let stage_dir = out_dir
        .join("bam")
        .join(stage.as_str().trim_start_matches(STAGE_PREFIX));
    bijux_dna_infra::ensure_dir(&stage_dir).context("create bam stage dir")?;
    write_stage_refusal_catalog(&stage_dir, stage)?;

    let mut args = base_bam_args(
        stage,
        profile,
        bam_path.to_path_buf(),
        stage_dir.clone(),
        bai_path.cloned(),
        reference.cloned(),
    );
    args.tool = Some(tool_id.as_str().to_string());

    let plan = plan_for_bam_stage_with_profile(stage, &spec, &args, profile, &stage_dir)?;
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Markdup
        && plan
            .params
            .get("umi_policy")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == "collapse")
    {
        return Err(anyhow!(
            "bam.collapse refusal: collapse mode is codified but not enabled in this runner (issue=BAM-4B-81)"
        ));
    }
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Contamination {
        let has_panels = plan
            .params
            .get("reference_panels")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|arr| !arr.is_empty());
        let scope = plan
            .params
            .get("scope")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("both");
        if !has_panels {
            return Err(anyhow!(
                "bam.contamination refusal: reference_panels must be non-empty for scope={scope}"
            ));
        }
    }
    let step = bijux_dna_stage_contract::execution_step_from_stage_plan(&plan);
    write_bam_invariants(&stage_dir, stage, bam_path, bai_path, reference)?;
    write_tool_wrapper_contract(&stage_dir, stage, &plan, &step)?;
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Markdup {
        write_duplicate_policy_split(&stage_dir, &plan)?;
    }
    if let Some(summary) = maybe_resume_bam_stage(stage, &stage_dir, &step)? {
        write_normalized_bam_metrics(stage, &stage_dir, &plan, &summary.result)?;
        return Ok(summary);
    }
    let context = ToolContext {
        run_id: format!("bam-{}-{}", stage.as_str(), tool_id.as_str()),
        stage_id: stage.as_str().to_string(),
        tool_id: tool_id.as_str().to_string(),
        sample_id: None,
        stage_root: bijux_dna_runtime::recording::run_artifacts_dir_for_out(&stage_dir),
        input_root: bam_path
            .parent()
            .map_or_else(|| out_dir.to_path_buf(), Path::to_path_buf),
        output_root: stage_dir.clone(),
        tmp_root: stage_dir.join("tmp"),
        threads: plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(plan.resources.mem_gb).saturating_mul(1024)),
        compression_threads: Some(1),
        seed: None,
        network_policy: NetworkPolicy::Allow,
    };
    let result = crate::execution_kernel::ToolExec::invoke(&ToolInvocationRequest {
        step: step.clone(),
        runner: platform.runner,
        context,
        timeout: None,
        mode: crate::execution_kernel::ToolExecMode::Execute,
    })?
    .stage_result;
    stage_postprocess(stage, &stage_dir, &plan)?;
    if let Some(bam_root) = stage_dir.parent() {
        write_bam_qc_aggregator_tsv(bam_root)?;
    }
    write_bam_output_contract(stage, &plan, &stage_dir)?;
    enforce_bam_output_contract(stage, &stage_dir)?;
    write_stage_accounting(&stage_dir, stage.as_str(), &result)?;
    write_normalized_bam_metrics(stage, &stage_dir, &plan, &result)?;
    if result.exit_code != 0 {
        write_stage_failure_hint(&stage_dir, stage, &result)?;
    }
    Ok(StageExecutionSummary { plan: step, result })
}

#[allow(clippy::too_many_lines)]
pub(crate) fn run_bam_align_and_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    reference: &ReferenceRecord,
    args: &FastqCrossArgs,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let r1 = args
        .r1
        .as_ref()
        .ok_or_else(|| anyhow!("--r1 required for cross align"))?;
    let sample_id = args.sample_id.as_deref().unwrap_or("sample").to_string();
    let regime = infer_alignment_regime(profile, args);
    let align_out = out_dir.join("bam").join("align");
    bijux_dna_infra::ensure_dir(&align_out)?;
    write_stage_refusal_catalog(&align_out, bijux_dna_planner_bam::stage_api::BamStage::Align)?;
    let align_stage = bijux_dna_core::ids::StageId::from_static(
        bijux_dna_planner_bam::stage_api::BamStage::Align.as_str(),
    );
    let tool_id = profile
        .defaults
        .tools
        .get(&align_stage)
        .cloned()
        .unwrap_or_else(|| bijux_dna_core::ids::ToolId::from_static("bwa"));
    let spec = build_tool_execution_spec(
        bijux_dna_planner_bam::stage_api::BamStage::Align.as_str(),
        tool_id.as_str(),
        registry_core,
        catalog,
        platform,
    )?;
    let mut align_args = base_bam_args(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        profile,
        align_out.join("align.bam"),
        align_out.clone(),
        None,
        Some(reference.fasta.clone()),
    );
    align_args.sample_id = Some(sample_id.clone());
    align_args.r1 = Some(r1.clone());
    align_args.r2.clone_from(&args.r2);
    align_args.tool = Some(tool_id.as_str().to_string());
    align_args.build_reference_indices = true;
    align_args.aligner_preset = Some(match regime {
        AlignmentRegime::Adna => "adna_sensitive".to_string(),
        AlignmentRegime::Modern => "modern_default".to_string(),
        AlignmentRegime::Edna => "edna_metagenomic".to_string(),
    });
    let align_plan = plan_for_bam_stage_with_profile(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &spec,
        &align_args,
        profile,
        &align_out,
    )?;
    let align_step = bijux_dna_stage_contract::execution_step_from_stage_plan(&align_plan);
    let align_bam_path = align_out.join("align.bam");
    let align_bai_path = align_out.join("align.bam.bai");
    write_bam_invariants(
        &align_out,
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_bam_path,
        Some(&align_bai_path),
        Some(&reference.fasta),
    )?;
    write_tool_wrapper_contract(
        &align_out,
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_plan,
        &align_step,
    )?;
    write_alignment_regime_validation(&align_out, regime, tool_id.as_str(), &align_step)?;
    let alignment_parameters_path = align_out.join("alignment.parameters.json");
    bijux_dna_infra::atomic_write_json(
        &alignment_parameters_path,
        &serde_json::json!({
            "schema_version": "bijux.bam.alignment_parameters.v1",
            "tool_id": tool_id.as_str(),
            "regime": regime,
            "command_template": align_step.command.template.clone(),
            "derived": {
                "preset": align_args.aligner_preset.clone(),
                "read_group": {
                    "rg_id": align_args.rg_id.clone().unwrap_or_else(|| format!("rg-{sample_id}")),
                    "rg_sm": align_args.rg_sm.clone().unwrap_or_else(|| sample_id.clone()),
                    "rg_pl": align_args.rg_pl.clone().unwrap_or_else(|| "ILLUMINA".to_string()),
                    "rg_lb": align_args.rg_lb.clone().unwrap_or_else(|| "lib1".to_string()),
                    "policy": align_args.rg_policy.clone().unwrap_or_else(|| "regenerate".to_string()),
                }
            }
        }),
    )
    .with_context(|| format!("write {}", alignment_parameters_path.display()))?;
    if let Some(summary) = maybe_resume_bam_stage(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_out,
        &align_step,
    )? {
        write_normalized_bam_metrics(
            bijux_dna_planner_bam::stage_api::BamStage::Align,
            &align_out,
            &align_plan,
            &summary.result,
        )?;
        return Ok(vec![summary]);
    }
    let align_context = ToolContext {
        run_id: format!("bam-{}-{}", align_step.step_id, tool_id.as_str()),
        stage_id: align_step.step_id.to_string(),
        tool_id: tool_id.as_str().to_string(),
        sample_id: Some(sample_id.clone()),
        stage_root: bijux_dna_runtime::recording::run_artifacts_dir_for_out(&align_out),
        input_root: r1
            .parent()
            .map_or_else(|| out_dir.to_path_buf(), Path::to_path_buf),
        output_root: align_out.clone(),
        tmp_root: align_out.join("tmp"),
        threads: align_plan.resources.threads.max(1),
        memory_hint_mb: Some(u64::from(align_plan.resources.mem_gb).saturating_mul(1024)),
        compression_threads: Some(1),
        seed: None,
        network_policy: NetworkPolicy::Allow,
    };
    let align_result = crate::execution_kernel::ToolExec::invoke(&ToolInvocationRequest {
        step: align_step.clone(),
        runner: platform.runner,
        context: align_context,
        timeout: None,
        mode: crate::execution_kernel::ToolExecMode::Execute,
    })?
    .stage_result;
    write_stage_accounting(&align_out, align_step.step_id.as_str(), &align_result)?;
    write_bam_output_contract(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_plan,
        &align_out,
    )?;
    enforce_bam_output_contract(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_out,
    )?;
    write_normalized_bam_metrics(
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_out,
        &align_plan,
        &align_result,
    )?;
    let header_normalization = serde_json::json!({
        "stage_id": align_step.step_id,
        "regime": regime,
        "policy": "read_group_normalization",
        "rg_id": align_args.rg_id.as_deref().unwrap_or("auto"),
        "rg_sm": align_args.rg_sm.as_deref().unwrap_or(&sample_id),
        "rg_pl": align_args.rg_pl.as_deref().unwrap_or("ILLUMINA"),
        "rg_lb": align_args.rg_lb.as_deref().unwrap_or("lib1"),
    });
    let header_norm_path = align_out.join("header_normalization.json");
    bijux_dna_infra::atomic_write_json(&header_norm_path, &header_normalization)
        .with_context(|| format!("write {}", header_norm_path.display()))?;
    let regime_path = align_out.join("alignment_regime.json");
    bijux_dna_infra::atomic_write_json(
        &regime_path,
        &serde_json::json!({
            "regime": regime,
            "profile_id": profile.id,
            "source": "explicit_or_profile_inference",
        }),
    )
    .with_context(|| format!("write {}", regime_path.display()))?;
    let mut runs = vec![StageExecutionSummary {
        plan: align_step,
        result: align_result,
    }];

    let boundary = AlignmentBoundary {
        bam_path: align_out.join("align.bam").display().to_string(),
        bai_path: Some(align_out.join("align.bam.bai").display().to_string()),
        reference: Some(reference.fasta.display().to_string()),
        rg_policy: args.alignment_rg_policy.clone(),
        aligner_meta: None,
    };
    let mut rest = run_bam_truth_stages(
        registry_core,
        catalog,
        platform,
        profile,
        &boundary,
        out_dir,
    )?;
    runs.append(&mut rest);
    Ok(runs)
}
