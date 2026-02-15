pub(crate) fn run_bam_truth_stages<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    boundary: &AlignmentBoundary,
    out_dir: &Path,
) -> Result<Vec<StageExecutionSummary>> {
    let bam_path = PathBuf::from(&boundary.bam_path);
    let bai_path = boundary.bai_path.as_ref().map(PathBuf::from);
    let reference = boundary.reference.as_ref().map(PathBuf::from);

    let mut runs = Vec::new();
    for stage_id in bijux_dna_planner_bam::pipeline_id_catalog(profile.id.as_str()) {
        let stage = bijux_dna_planner_bam::stage_api::BamStage::try_from(stage_id.as_str())?;
        if should_skip_bam_truth_stage(stage) {
            continue;
        }
        let summary = run_bam_truth_stage(
            registry_core,
            catalog,
            platform,
            profile,
            stage,
            &bam_path,
            bai_path.as_ref(),
            reference.as_ref(),
            out_dir,
        )?;
        runs.push(summary);
    }

    Ok(runs)
}

fn should_skip_bam_truth_stage(stage: bijux_dna_planner_bam::stage_api::BamStage) -> bool {
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align {
        return true;
    }
    if !downstream_enabled()
        && matches!(
            stage,
            bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
                | bijux_dna_planner_bam::stage_api::BamStage::Genotyping
                | bijux_dna_planner_bam::stage_api::BamStage::Kinship
        )
    {
        return true;
    }
    false
}

fn parse_sort_order_from_header_hint(bam_path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(bam_path).ok()?;
    for line in raw.lines() {
        if !line.starts_with("@HD") {
            continue;
        }
        for field in line.split('\t') {
            if let Some(value) = field.strip_prefix("SO:") {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn command_stdout(bin: &str, args: &[&str]) -> Option<String> {
    let out = std::process::Command::new(bin).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

fn bam_sort_order_samtools(bam_path: &Path) -> Option<String> {
    let out = command_stdout("samtools", &["view", "-H", bam_path.to_string_lossy().as_ref()])?;
    for line in out.lines() {
        if !line.starts_with("@HD") {
            continue;
        }
        for field in line.split('\t') {
            if let Some(value) = field.strip_prefix("SO:") {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn bam_header_contig_names_samtools(bam_path: &Path) -> Vec<String> {
    let Some(out) = command_stdout("samtools", &["view", "-H", bam_path.to_string_lossy().as_ref()])
    else {
        return Vec::new();
    };
    out.lines()
        .filter(|line| line.starts_with("@SQ"))
        .filter_map(|line| {
            line.split('\t')
                .find_map(|field| field.strip_prefix("SN:"))
                .map(std::string::ToString::to_string)
        })
        .collect()
}

fn bam_read_group_presence_samtools(bam_path: &Path) -> Option<bool> {
    let out = command_stdout("samtools", &["view", "-H", bam_path.to_string_lossy().as_ref()])?;
    Some(out.lines().any(|line| line.starts_with("@RG")))
}

fn bam_quickcheck_ok(bam_path: &Path) -> Option<bool> {
    let status = std::process::Command::new("samtools")
        .args(["quickcheck", bam_path.to_string_lossy().as_ref()])
        .status()
        .ok()?;
    Some(status.success())
}

fn bam_has_dup_tag_samtools(bam_path: &Path) -> Option<bool> {
    let out = command_stdout("samtools", &["view", bam_path.to_string_lossy().as_ref()])?;
    Some(out.lines().take(200).any(|line| line.contains("\tDT:")))
}

fn bam_header_contig_names_hint(bam_path: &Path) -> Vec<String> {
    let Ok(raw) = std::fs::read_to_string(bam_path) else {
        return Vec::new();
    };
    raw.lines()
        .filter(|line| line.starts_with("@SQ"))
        .filter_map(|line| {
            line.split('\t')
                .find_map(|field| field.strip_prefix("SN:"))
                .map(std::string::ToString::to_string)
        })
        .collect()
}

fn read_group_presence_hint(bam_path: &Path) -> &'static str {
    let Ok(raw) = std::fs::read_to_string(bam_path) else {
        return "unknown";
    };
    if raw.lines().any(|line| line.starts_with("@RG")) {
        "present"
    } else if raw.lines().any(|line| line.starts_with('@')) {
        "absent"
    } else {
        "unknown"
    }
}

fn reference_contig_names(reference: Option<&PathBuf>) -> Vec<String> {
    let Some(reference) = reference else {
        return Vec::new();
    };
    let fai = PathBuf::from(format!("{}.fai", reference.display()));
    let Ok(raw) = std::fs::read_to_string(fai) else {
        return Vec::new();
    };
    raw.lines()
        .filter_map(|line| line.split('\t').next())
        .map(std::string::ToString::to_string)
        .collect()
}

fn write_bam_invariants(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
) -> Result<()> {
    let sort_order = bam_sort_order_samtools(bam_path)
        .or_else(|| parse_sort_order_from_header_hint(bam_path))
        .unwrap_or_else(|| {
        bijux_dna_domain_bam::contract_for_stage(stage.as_str())
            .map(|contract| contract.sorting)
            .unwrap_or("unspecified")
            .to_string()
        });
    let duplicate_policy = bijux_dna_domain_bam::contract_for_stage(stage.as_str())
        .map(|contract| contract.duplicate_policy)
        .unwrap_or("unspecified")
        .to_string();
    let header_contigs = {
        let contigs = bam_header_contig_names_samtools(bam_path);
        if contigs.is_empty() {
            bam_header_contig_names_hint(bam_path)
        } else {
            contigs
        }
    };
    let reference_contigs = reference_contig_names(reference);
    let contig_mismatch = !header_contigs.is_empty()
        && !reference_contigs.is_empty()
        && header_contigs != reference_contigs;
    let read_group_status = if let Some(has_rg) = bam_read_group_presence_samtools(bam_path) {
        if has_rg { "present" } else { "absent" }
    } else {
        read_group_presence_hint(bam_path)
    };
    let quickcheck_ok = bam_quickcheck_ok(bam_path);
    let has_duplicate_tags = bam_has_dup_tag_samtools(bam_path);
    let path = stage_dir.join("bam_invariants.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.invariants.v1",
        "stage_id": stage.as_str(),
        "bam_path": bam_path,
        "sort_order": sort_order,
        "header_contigs": header_contigs,
        "reference_contigs": reference_contigs,
        "contig_mismatch": contig_mismatch,
        "read_groups": {
            "status": read_group_status,
        },
        "index": {
            "required": bai_path.is_some(),
            "path": bai_path,
            "exists": bai_path.is_some_and(|path| path.exists()),
        },
        "bam_quickcheck_ok": quickcheck_ok,
        "duplicate_tags": {
            "policy": duplicate_policy,
            "has_dt_tag": has_duplicate_tags,
        },
        "duplicate_tags_policy": duplicate_policy,
    });
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn expected_bam_contract_outputs(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut bams = Vec::new();
    let mut indices = Vec::new();
    for artifact in bijux_dna_domain_bam::required_audit_artifacts(stage) {
        let output = stage_dir.join(artifact.filename);
        if artifact.filename.ends_with(".bam") {
            bams.push(output);
        } else if artifact.filename.ends_with(".bam.bai") {
            indices.push(output);
        }
    }
    (bams, indices)
}

fn write_bam_output_contract(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
) -> Result<()> {
    let (bams, indices) = expected_bam_contract_outputs(stage, stage_dir);
    let contract_path = stage_dir.join("bam_output_contract.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.output_contract.v1",
        "stage_id": stage.as_str(),
        "deterministic_root": stage_dir,
        "required_bam": bams,
        "required_bai": indices,
        "all_required_present": bams.iter().all(|path| path.exists()) && indices.iter().all(|path| path.exists()),
        "naming_policy": "deterministic",
    });
    bijux_dna_infra::atomic_write_json(&contract_path, &payload)
        .with_context(|| format!("write {}", contract_path.display()))
}

fn enforce_bam_output_contract(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
) -> Result<()> {
    let (bams, indices) = expected_bam_contract_outputs(stage, stage_dir);
    if bams.is_empty() && indices.is_empty() {
        return Ok(());
    }
    let missing = bams
        .iter()
        .chain(indices.iter())
        .filter(|path| !path.exists())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!(
            "{} output contract violation: missing artifacts: {}",
            stage.as_str(),
            missing.join(", ")
        ));
    }
    for bam in &bams {
        if bam.extension().and_then(|ext| ext.to_str()) != Some("bam") {
            return Err(anyhow!(
                "{} output contract violation: non-.bam artifact in BAM slot: {}",
                stage.as_str(),
                bam.display()
            ));
        }
        let has_matching_index = indices
            .iter()
            .any(|idx| idx == &PathBuf::from(format!("{}.bai", bam.display())));
        if !has_matching_index {
            return Err(anyhow!(
                "{} output contract violation: missing matching .bai for {}",
                stage.as_str(),
                bam.display()
            ));
        }
    }
    Ok(())
}

fn stage_resume_summary_path(stage_dir: &Path) -> PathBuf {
    stage_dir.join("stage_resume.json")
}

fn build_resumed_stage_result(outputs: Vec<PathBuf>) -> bijux_dna_runner::execute::StageResultV1 {
    bijux_dna_runner::execute::StageResultV1 {
        run_id: "resume-skip".to_string(),
        exit_code: 0,
        runtime_s: 0.0,
        memory_mb: 0.0,
        outputs,
        metrics_path: None,
        stdout: "skipped: existing outputs verified".to_string(),
        stderr: String::new(),
        command: "resume-skip".to_string(),
    }
}

fn maybe_resume_bam_stage(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
    step: &bijux_dna_core::contract::ExecutionStep,
) -> Result<Option<StageExecutionSummary>> {
    let accounting = stage_dir.join("stage_loss_accounting.json");
    if !accounting.exists() {
        return Ok(None);
    }
    let (bams, indices) = expected_bam_contract_outputs(stage, stage_dir);
    let expected = bams.into_iter().chain(indices).collect::<Vec<_>>();
    if !expected.iter().all(|path| path.exists()) {
        return Ok(None);
    }
    let accounting_raw = std::fs::read_to_string(&accounting)
        .with_context(|| format!("read {}", accounting.display()))?;
    let accounting_json: serde_json::Value =
        serde_json::from_str(&accounting_raw).with_context(|| "parse stage accounting json")?;
    let prior_checksums = accounting_json
        .get("output_checksums")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut checksum_map = std::collections::BTreeMap::new();
    for entry in prior_checksums {
        if let (Some(path), Some(sha)) = (
            entry.get("path").and_then(serde_json::Value::as_str),
            entry.get("sha256").and_then(serde_json::Value::as_str),
        ) {
            checksum_map.insert(path.to_string(), sha.to_string());
        }
    }
    for path in &expected {
        let key = path.display().to_string();
        let Some(previous) = checksum_map.get(&key) else {
            return Ok(None);
        };
        let current = bijux_dna_infra::hash_file_sha256(path)
            .with_context(|| format!("hash output {}", path.display()))?;
        if &current != previous {
            return Ok(None);
        }
    }
    let resume_path = stage_resume_summary_path(stage_dir);
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.resume.v1",
        "stage_id": stage.as_str(),
        "resume": true,
        "reason": "existing_artifacts_match_contract",
        "output_count": expected.len(),
    });
    bijux_dna_infra::atomic_write_json(&resume_path, &payload)
        .with_context(|| format!("write {}", resume_path.display()))?;
    Ok(Some(StageExecutionSummary {
        plan: step.clone(),
        result: build_resumed_stage_result(expected),
    }))
}

fn write_tool_wrapper_contract(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    step: &bijux_dna_core::contract::ExecutionStep,
) -> Result<()> {
    let path = stage_dir.join("tool_wrapper.json");
    let payload = serde_json::json!({
        "schema_version": "bijux.bam.tool_wrapper.v1",
        "wrapper_kind": "standard_tool_exec_v1",
        "stage_id": stage.as_str(),
        "tool_id": plan.tool_id,
        "tool_version": plan.tool_version,
        "container_image": plan.image.image,
        "command_template": step.command.template,
        "resources": {
            "threads": plan.resources.threads,
            "memory_gb": plan.resources.mem_gb,
        },
        "logs": {
            "policy": "stage_root/stdout.log + stage_root/stderr.log",
            "stdout": "stdout.log",
            "stderr": "stderr.log",
        }
    });
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn write_normalized_bam_metrics(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    result: &bijux_dna_runner::execute::StageResultV1,
) -> Result<()> {
    let mut artifact_status = serde_json::Map::new();
    let mut missing = Vec::new();
    for artifact in bijux_dna_domain_bam::required_audit_artifacts(stage) {
        let path = stage_dir.join(artifact.filename);
        let exists = path.exists();
        if !exists {
            missing.push(artifact.name.to_string());
        }
        artifact_status.insert(
            artifact.name.to_string(),
            serde_json::json!({
                "path": path,
                "exists": exists,
            }),
        );
    }
    let normalized_metrics = serde_json::json!({
        "schema_version": "bijux.bam.metrics.normalized.v1",
        "stage_id": stage.as_str(),
        "tool_id": plan.tool_id,
        "tool_version": plan.tool_version,
        "execution": {
            "runtime_s": result.runtime_s,
            "memory_mb": result.memory_mb,
            "exit_code": result.exit_code,
            "status": if result.exit_code == 0 { "ok" } else { "failed" },
        },
        "outputs_count": result.outputs.len(),
        "artifacts": {
            "required_count": artifact_status.len(),
            "missing_required": missing,
            "items": artifact_status,
        },
        "contracts": {
            "bam_invariants": stage_dir.join("bam_invariants.json").exists(),
            "output_contract": stage_dir.join("bam_output_contract.json").exists(),
            "tool_wrapper": stage_dir.join("tool_wrapper.json").exists(),
        },
        "normalized_keys": [
            "stage_id",
            "tool_id",
            "tool_version",
            "execution.runtime_s",
            "execution.memory_mb",
            "execution.exit_code",
            "artifacts.required_count",
            "artifacts.missing_required",
            "contracts.bam_invariants",
            "contracts.output_contract",
            "contracts.tool_wrapper"
        ],
    });
    let stage_metrics = bijux_dna_core::metrics::StageMetricsV1 {
        schema_version: "bijux.stage_metrics.v1".to_string(),
        stage_id: stage.as_str().to_string(),
        stage_version: plan.stage_version.0,
        tool_id: plan.tool_id.to_string(),
        tool_version: plan.tool_version.clone(),
        context: bijux_dna_core::metrics::MetricContextV1 {
            tool_id: plan.tool_id.to_string(),
            tool_version: plan.tool_version.clone(),
            image_digest: plan.image.digest.clone(),
            runner: "container".to_string(),
            platform: "cross".to_string(),
            input_hash: "unknown".to_string(),
            params_hash: "unknown".to_string(),
            presets: std::collections::BTreeMap::new(),
            banks: std::collections::BTreeMap::new(),
        },
        metric_provenance: None,
        execution: bijux_dna_core::prelude::measure::ExecutionMetrics {
            runtime_s: result.runtime_s,
            memory_mb: result.memory_mb,
            exit_code: result.exit_code,
        },
        failure_class: (result.exit_code != 0).then_some("stage_failed".to_string()),
        failure_reason: (result.exit_code != 0).then_some("non_zero_exit_code".to_string()),
        metrics: normalized_metrics.clone(),
    };
    let run_artifacts = bijux_dna_runtime::recording::run_artifacts_dir_for_out(stage_dir);
    bijux_dna_infra::ensure_dir(&run_artifacts)
        .with_context(|| format!("create {}", run_artifacts.display()))?;
    bijux_dna_runtime::recording::write_stage_metrics_json(&run_artifacts, &stage_metrics)
        .with_context(|| format!("write normalized metrics for {}", stage.as_str()))?;
    let stage_metrics_path = stage_dir.join("metrics.json");
    bijux_dna_infra::atomic_write_json(&stage_metrics_path, &normalized_metrics)
        .with_context(|| format!("write {}", stage_metrics_path.display()))?;
    Ok(())
}

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

fn run_bam_truth_stage<S: std::hash::BuildHasher>(
    registry_core: &ToolRegistry,
    catalog: &std::collections::HashMap<String, bijux_dna_environment::api::ToolImageSpec, S>,
    platform: &bijux_dna_environment::api::PlatformSpec,
    profile: &PipelineProfile,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    bam_path: &Path,
    bai_path: Option<&PathBuf>,
    reference: Option<&PathBuf>,
    out_dir: &Path,
) -> Result<StageExecutionSummary> {
    enforce_stage_refusal_rules(stage, bam_path, bai_path, reference)?;
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
    write_bam_output_contract(stage, &stage_dir)?;
    enforce_bam_output_contract(stage, &stage_dir)?;
    write_stage_accounting(&stage_dir, stage.as_str(), &result)?;
    write_normalized_bam_metrics(stage, &stage_dir, &plan, &result)?;
    if result.exit_code != 0 {
        write_stage_failure_hint(&stage_dir, stage, &result)?;
    }
    Ok(StageExecutionSummary { plan: step, result })
}

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
    let align_bam = align_out.join("align.bam");
    let align_bai = align_out.join("align.bam.bai");
    write_bam_invariants(
        &align_out,
        bijux_dna_planner_bam::stage_api::BamStage::Align,
        &align_bam,
        Some(&align_bai),
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
                    "rg_id": align_args.rg_id.clone().unwrap_or_else(|| format!("rg-{}", sample_id)),
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
