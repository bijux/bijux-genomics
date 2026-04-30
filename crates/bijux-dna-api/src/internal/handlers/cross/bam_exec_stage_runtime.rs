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
    let rg_policy_override = boundary.rg_policy.as_deref();

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
            rg_policy_override,
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
    let args = args.iter().map(|arg| (*arg).to_string()).collect::<Vec<_>>();
    let out = bijux_dna_runner::command_runner::run_command(bin, &args).ok()?;
    if out.exit_code != 0 {
        return None;
    }
    Some(out.stdout)
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
    let args = vec!["quickcheck".to_string(), bam_path.to_string_lossy().into_owned()];
    let output = bijux_dna_runner::command_runner::run_command("samtools", &args).ok()?;
    Some(output.exit_code == 0)
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
        return "missing_header";
    };
    if raw.lines().any(|line| line.starts_with("@RG")) {
        "present"
    } else if raw.lines().any(|line| line.starts_with('@')) {
        "absent"
    } else {
        "missing_header"
    }
}

fn bam_read_group_records(bam_path: &Path) -> Vec<std::collections::BTreeMap<String, String>> {
    let header = command_stdout("samtools", &["view", "-H", bam_path.to_string_lossy().as_ref()])
        .or_else(|| std::fs::read_to_string(bam_path).ok())
        .unwrap_or_default();
    header
        .lines()
        .filter(|line| line.starts_with("@RG"))
        .map(|line| {
            line.split('\t')
                .skip(1)
                .filter_map(|field| field.split_once(':'))
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .collect()
}

fn bam_read_group_missing_required_fields(bam_path: &Path) -> Vec<String> {
    let required = ["ID", "SM", "PL", "LB"];
    let rgs = bam_read_group_records(bam_path);
    if rgs.is_empty() {
        return Vec::new();
    }
    let mut missing = std::collections::BTreeSet::new();
    for rg in rgs {
        for field in required {
            if rg.get(field).is_none_or(|value| value.trim().is_empty()) {
                missing.insert(field.to_string());
            }
        }
    }
    missing.into_iter().collect()
}

fn bam_read_group_deterministic_naming(bam_path: &Path) -> bool {
    let rgs = bam_read_group_records(bam_path);
    if rgs.is_empty() {
        return false;
    }
    rgs.iter().all(|rg| {
        rg.get("ID")
            .is_some_and(|value| value.starts_with("rg-") || value.starts_with("RG-"))
            && rg.get("SM").is_some_and(|value| !value.trim().is_empty())
    })
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
            .map_or("unspecified", |contract| contract.sorting)
            .to_string()
        });
    let duplicate_policy = bijux_dna_domain_bam::contract_for_stage(stage.as_str())
        .map_or("unspecified", |contract| contract.duplicate_policy)
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
    let read_group_missing_fields = bam_read_group_missing_required_fields(bam_path);
    let deterministic_read_group_naming = bam_read_group_deterministic_naming(bam_path);
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
            "required_fields": ["ID", "SM", "PL", "LB"],
            "missing_required_fields": read_group_missing_fields,
            "deterministic_naming": deterministic_read_group_naming,
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
        let ext = std::path::Path::new(artifact.filename)
            .extension()
            .and_then(|value| value.to_str());
        let lower = artifact.filename.to_ascii_lowercase();
        if ext.is_some_and(|value| value.eq_ignore_ascii_case("bam")) {
            bams.push(output);
        } else if lower.ends_with(".bam.bai") {
            indices.push(output);
        }
    }
    (bams, indices)
}

fn write_bam_output_contract(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    stage_dir: &Path,
) -> Result<()> {
    let (bams, indices) = expected_bam_contract_outputs(stage, stage_dir);
    let contract_path = stage_dir.join("bam_output_contract.json");
    let inventory =
        bijux_dna_domain_bam::bam_artifact_inventory_from_outputs(stage.as_str(), stage_dir, &plan.io.outputs);
    let payload = serde_json::json!({
        "schema_version": bijux_dna_domain_bam::BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION,
        "stage_id": stage.as_str(),
        "stage_family": inventory.stage_family,
        "deterministic_root": stage_dir,
        "required_bam": bams,
        "required_bai": indices,
        "all_required_present": bams.iter().all(|path| path.exists()) && indices.iter().all(|path| path.exists()),
        "naming_policy": "deterministic",
        "outputs": inventory.outputs,
    });
    bijux_dna_infra::atomic_write_json(&contract_path, &payload)
        .with_context(|| format!("write {}", contract_path.display()))
}

fn read_group_spec_from_record(
    record: &std::collections::BTreeMap<String, String>,
) -> Option<bijux_dna_domain_bam::params::ReadGroupSpec> {
    Some(bijux_dna_domain_bam::params::ReadGroupSpec {
        id: record.get("ID")?.to_string(),
        sample: record.get("SM")?.to_string(),
        platform: record.get("PL")?.to_string(),
        library: record.get("LB")?.to_string(),
        platform_unit: record.get("PU").cloned(),
        lane_id: None,
        run_id: None,
    })
}

fn planned_read_group_spec(
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Option<bijux_dna_domain_bam::params::ReadGroupSpec> {
    plan.params
        .get("read_group")
        .cloned()
        .or_else(|| plan.effective_params.get("read_group").cloned())
        .and_then(|value| serde_json::from_value(value).ok())
}

fn plan_string(plan: &bijux_dna_stage_contract::StagePlanV1, key: &str) -> Option<String> {
    plan.params.get(key).and_then(serde_json::Value::as_str).map(ToOwned::to_owned)
}

fn write_bam_sample_identity_manifest(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    bam_path: Option<&Path>,
    rg_policy_override: Option<&str>,
) -> Result<bijux_dna_domain_bam::BamSampleIdentityV1> {
    let read_group = planned_read_group_spec(plan)
        .or_else(|| bam_path.and_then(|path| bam_read_group_records(path).first().and_then(read_group_spec_from_record)))
        .unwrap_or_else(|| {
            let sample_id = plan_string(plan, "sample_id").unwrap_or_else(|| "sample".to_string());
            bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults(&sample_id)
        });
    let sample_id = plan_string(plan, "sample_id")
        .or_else(|| bam_path.and_then(|path| {
            bam_read_group_records(path)
                .first()
                .and_then(|record| record.get("SM").cloned())
        }))
        .unwrap_or_else(|| read_group.sample.clone());
    let plan_rg_policy = plan_string(plan, "rg_policy");
    let plan_lane_id = plan_string(plan, "lane_id");
    let plan_library_id = plan_string(plan, "library_id").or_else(|| plan_string(plan, "rg_lb"));
    let plan_platform_unit = plan_string(plan, "rg_pu");
    let plan_run_id = plan_string(plan, "run_id");
    let plan_subject_id = plan_string(plan, "subject_id");
    let plan_cohort_id = plan_string(plan, "cohort_id");
    let identity = bijux_dna_domain_bam::bam_sample_identity(
        &sample_id,
        &read_group,
        rg_policy_override.or(plan_rg_policy.as_deref()),
        plan_lane_id.as_deref(),
        plan_library_id.as_deref(),
        plan_platform_unit.as_deref(),
        plan_run_id.as_deref(),
        plan_subject_id.as_deref(),
        plan_cohort_id.as_deref(),
    );
    let path = stage_dir.join("sample_identity.json");
    bijux_dna_infra::atomic_write_json(&path, &identity)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(identity)
}

fn write_bam_reference_preflight(
    stage_dir: &Path,
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    reference: &Path,
) -> Result<()> {
    let mut required_assets = Vec::new();
    let push_asset = |items: &mut Vec<bijux_dna_domain_bam::BamReferenceAssetIdentityV1>,
                      asset_kind: &str,
                      path: PathBuf| {
        let sha256 = path.exists().then(|| bijux_dna_infra::hash_file_sha256(&path).ok()).flatten();
        items.push(bijux_dna_domain_bam::BamReferenceAssetIdentityV1 {
            asset_kind: asset_kind.to_string(),
            path,
            sha256,
            exists: false,
        });
    };
    push_asset(&mut required_assets, "reference_fasta", reference.to_path_buf());
    push_asset(
        &mut required_assets,
        "reference_fai",
        PathBuf::from(format!("{}.fai", reference.display())),
    );
    push_asset(
        &mut required_assets,
        "reference_dict",
        PathBuf::from(format!("{}.dict", reference.display())),
    );
    if stage == bijux_dna_planner_bam::stage_api::BamStage::Align {
        match plan.tool_id.as_str() {
            "bwa" => {
                push_asset(
                    &mut required_assets,
                    "bwa_index",
                    PathBuf::from(format!("{}.bwt", reference.display())),
                );
            }
            "bowtie2" => {
                for suffix in [".1.bt2", ".2.bt2", ".3.bt2", ".4.bt2", ".rev.1.bt2", ".rev.2.bt2"] {
                    push_asset(
                        &mut required_assets,
                        "bowtie2_index",
                        PathBuf::from(format!("{}{}", reference.display(), suffix)),
                    );
                }
            }
            _ => {}
        }
    }
    for asset in &mut required_assets {
        asset.exists = asset.path.exists();
    }
    let payload = bijux_dna_domain_bam::BamReferencePreflightV1 {
        schema_version: bijux_dna_domain_bam::BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION.to_string(),
        stage_id: stage.as_str().to_string(),
        reference_fasta: reference.to_path_buf(),
        reference_digest: plan_string(plan, "reference_digest"),
        contig_alias_policy: "exact_match_or_chr_alias".to_string(),
        passes: required_assets.iter().all(|asset| asset.exists),
        required_assets,
        notes: vec![format!("tool_id={}", plan.tool_id)],
    };
    let path = stage_dir.join("reference_preflight.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
}

fn write_bam_alignment_provenance(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    sample_identity: bijux_dna_domain_bam::BamSampleIdentityV1,
) -> Result<()> {
    let read_group = planned_read_group_spec(plan).unwrap_or_else(|| {
        bijux_dna_domain_bam::params::ReadGroupSpec::with_defaults(&sample_identity.sample_id)
    });
    let payload = bijux_dna_domain_bam::BamAlignmentProvenanceV1 {
        schema_version: bijux_dna_domain_bam::BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION
            .to_string(),
        stage_id: plan.stage_id.to_string(),
        backend_tool_id: plan.tool_id.to_string(),
        preset: plan_string(plan, "preset"),
        sensitivity_profile: plan_string(plan, "sensitivity_profile"),
        seed_length: plan.params.get("seed_length").and_then(serde_json::Value::as_u64).map(|v| v as u32),
        reference_fasta: PathBuf::from(
            plan_string(plan, "reference").unwrap_or_else(|| "reference.fasta".to_string()),
        ),
        reference_digest: plan_string(plan, "reference_digest"),
        sample_identity,
        read_group,
        outputs: bijux_dna_domain_bam::bam_artifact_inventory_from_outputs(
            plan.stage_id.as_str(),
            stage_dir,
            &plan.io.outputs,
        ),
    };
    let path = stage_dir.join("alignment_provenance.json");
    bijux_dna_infra::atomic_write_json(&path, &payload)
        .with_context(|| format!("write {}", path.display()))
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

fn build_resumed_stage_result(outputs: Vec<PathBuf>) -> bijux_dna_runner::step_runner::StageResultV1 {
    bijux_dna_runner::step_runner::StageResultV1 {
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
        .ok_or_else(|| anyhow::anyhow!("stage accounting missing declared `output_checksums` array"))?;
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
    result: &bijux_dna_runner::step_runner::StageResultV1,
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
            input_hash: bijux_dna_core::prelude::input_fingerprint(
                &plan
                    .io
                    .inputs
                    .iter()
                    .map(|artifact| artifact.path.display().to_string())
                    .collect::<Vec<_>>(),
            ),
            params_hash: bijux_dna_core::prelude::params_hash(&plan.effective_params)
                .context("hash BAM effective params")?,
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

include!("bam_exec_stage_runtime_tail.rs");
