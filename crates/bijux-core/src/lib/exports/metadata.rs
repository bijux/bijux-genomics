#[must_use]
pub fn tool_tier_from_role(role: ToolRole) -> ToolTier {
    match role {
        ToolRole::Authoritative => ToolTier::Gold,
        ToolRole::Diagnostic => ToolTier::Silver,
        ToolRole::Experimental => ToolTier::Experimental,
    }
}

#[must_use]
pub fn tool_tier_for(stage_id: &str, tool_id: &str) -> (ToolTier, &'static str) {
    match (stage_id, tool_id) {
        ("fastq.trim" | "fastq.filter", "fastp")
        | ("fastq.validate_pre" | "fastq.qc_post", "fastqc")
        | ("fastq.filter", "bbduk") => (ToolTier::Gold, "curated_default"),
        ("fastq.trim", "seqpurge") => (ToolTier::Experimental, "experimental_tool"),
        ("fastq.validate_pre", "fqtools") => (ToolTier::Silver, "diagnostic_secondary"),
        ("fastq.merge", "vsearch" | "pear" | "bbmerge") => (ToolTier::Silver, "secondary_merge"),
        ("fastq.screen", "kraken2") => (ToolTier::Silver, "diagnostic_screen"),
        ("fastq.stats_neutral", "seqkit_stats") => (ToolTier::Silver, "diagnostic_stats"),
        _ => (ToolTier::Experimental, "unknown_tool"),
    }
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    stages: BTreeMap<String, StageManifestV1>,
    tools: BTreeMap<String, BTreeMap<String, ToolManifestV1>>,
}

impl ToolRegistry {
    #[must_use]
    pub fn stages(&self) -> &BTreeMap<String, StageManifestV1> {
        &self.stages
    }

    #[must_use]
    pub fn tools_for_stage(&self, stage_id: &str) -> Vec<&ToolManifestV1> {
        self.tools
            .get(stage_id)
            .map(|tools| tools.values().collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn tool_by_id(&self, stage_id: &str, tool_id: &str) -> Option<&ToolManifestV1> {
        self.tools
            .get(stage_id)
            .and_then(|tools| tools.get(tool_id))
    }
}

/// Load all manifests from the given domain directory and validate them.
///
/// # Errors
/// Returns an error if manifests cannot be read, parsed, or validated.
pub fn load_manifests(modules_dir: &Path) -> Result<ToolRegistry, BijuxError> {
    let mut stages = BTreeMap::new();
    let mut tools: BTreeMap<String, BTreeMap<String, ToolManifestV1>> = BTreeMap::new();
    let mut stage_ids = BTreeSet::new();
    let mut tool_keys = BTreeSet::new();

    for entry in WalkDir::new(modules_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let is_stage = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("stages");
        if is_stage && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            let manifest = match load_stage_manifest(path) {
                Ok(Some(manifest)) => manifest,
                Ok(None) => continue,
                Err(err) => return Err(err),
            };
            validate_stage_manifest(path, &manifest)?;
            if stage_ids.contains(&manifest.stage_id) {
                return Err(BijuxError::Manifest(format!(
                    "duplicate stage_id {} at {}",
                    manifest.stage_id,
                    path.display()
                )));
            }
            stage_ids.insert(manifest.stage_id.clone());
            stages.insert(manifest.stage_id.clone(), manifest);
        }
    }

    for entry in WalkDir::new(modules_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            let is_tool = path
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                == Some("tools");
            if is_tool && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                let contents = std::fs::read_to_string(path)?;
                let manifest: ToolManifestV1 = serde_yaml::from_str(&contents)
                    .map_err(|err| BijuxError::Manifest(format!("{}: {err}", path.display())))?;
                validate_tool_manifest(path, &manifest)?;
                if !stages.contains_key(&manifest.stage_id) {
                    return Err(BijuxError::Manifest(format!(
                        "tool {} references unknown stage_id {} at {}",
                        manifest.tool_id,
                        manifest.stage_id,
                        path.display()
                    )));
                }
                let key = format!("{}::{}", manifest.stage_id, manifest.tool_id);
                if tool_keys.contains(&key) {
                    return Err(BijuxError::Manifest(format!(
                        "duplicate tool_id {} for stage {} at {}",
                        manifest.tool_id,
                        manifest.stage_id,
                        path.display()
                    )));
                }
                tool_keys.insert(key);
                tools
                    .entry(manifest.stage_id.clone())
                    .or_default()
                    .insert(manifest.tool_id.clone(), manifest);
            }
        }
    }

    Ok(ToolRegistry { stages, tools })
}

fn load_stage_manifest(path: &Path) -> Result<Option<StageManifestV1>, BijuxError> {
    let contents = std::fs::read_to_string(path)?;
    let doc: StageManifestDoc = serde_yaml::from_str(&contents)
        .map_err(|err| BijuxError::Manifest(format!("{}: {err}", path.display())))?;
    if doc.stage_id.is_none() && doc.extends.is_none() {
        return Ok(None);
    }
    let resolved = resolve_stage_doc(path, doc)?;
    Ok(Some(stage_doc_to_manifest(path, resolved)?))
}

fn resolve_stage_doc(path: &Path, doc: StageManifestDoc) -> Result<StageManifestDoc, BijuxError> {
    if let Some(extends) = &doc.extends {
        let base_path = path
            .parent()
            .ok_or_else(|| BijuxError::Manifest(format!("{} has no parent", path.display())))?
            .join(extends);
        let base_contents = std::fs::read_to_string(&base_path)?;
        let base_doc: StageManifestDoc = serde_yaml::from_str(&base_contents)
            .map_err(|err| BijuxError::Manifest(format!("{}: {err}", base_path.display())))?;
        let resolved_base = resolve_stage_doc(&base_path, base_doc)?;
        return Ok(merge_stage_docs(resolved_base, doc));
    }
    Ok(doc)
}

fn merge_stage_docs(base: StageManifestDoc, overlay: StageManifestDoc) -> StageManifestDoc {
    StageManifestDoc {
        extends: None,
        schema_version: overlay.schema_version.or(base.schema_version),
        stage_id: overlay.stage_id.or(base.stage_id),
        domain: overlay.domain.or(base.domain),
        inputs: overlay.inputs.or(base.inputs),
        outputs: overlay.outputs.or(base.outputs),
        parameters: overlay.parameters.or(base.parameters),
        metrics: overlay.metrics.or(base.metrics),
        description: overlay.description.or(base.description),
        mutates_fastq: overlay.mutates_fastq.or(base.mutates_fastq),
        report_only: overlay.report_only.or(base.report_only),
        may_change_read_count: overlay.may_change_read_count.or(base.may_change_read_count),
        image_requirements: overlay.image_requirements.or(base.image_requirements),
    }
}

fn stage_doc_to_manifest(
    path: &Path,
    doc: StageManifestDoc,
) -> Result<StageManifestV1, BijuxError> {
    let Some(schema_version) = doc.schema_version else {
        return Err(BijuxError::Manifest(format!(
            "missing schema_version for stage at {}",
            path.display()
        )));
    };
    let Some(stage_id) = doc.stage_id else {
        return Err(BijuxError::Manifest(format!(
            "missing stage_id for stage at {}",
            path.display()
        )));
    };
    let Some(domain) = doc.domain else {
        return Err(BijuxError::Manifest(format!(
            "missing domain for stage at {}",
            path.display()
        )));
    };
    let Some(inputs) = doc.inputs else {
        return Err(BijuxError::Manifest(format!(
            "missing inputs for stage at {}",
            path.display()
        )));
    };
    let Some(outputs) = doc.outputs else {
        return Err(BijuxError::Manifest(format!(
            "missing outputs for stage at {}",
            path.display()
        )));
    };
    let Some(parameters) = doc.parameters else {
        return Err(BijuxError::Manifest(format!(
            "missing parameters for stage at {}",
            path.display()
        )));
    };
    let Some(metrics) = doc.metrics else {
        return Err(BijuxError::Manifest(format!(
            "missing metrics for stage at {}",
            path.display()
        )));
    };
    let Some(description) = doc.description else {
        return Err(BijuxError::Manifest(format!(
            "missing description for stage at {}",
            path.display()
        )));
    };
    Ok(StageManifestV1 {
        schema_version,
        stage_id,
        domain,
        inputs,
        outputs,
        parameters,
        metrics,
        description,
        mutates_fastq: doc.mutates_fastq.unwrap_or(false),
        report_only: doc.report_only.unwrap_or(false),
        may_change_read_count: doc.may_change_read_count.unwrap_or(false),
        image_requirements: doc.image_requirements.unwrap_or_default(),
    })
}

fn validate_stage_manifest(path: &Path, manifest: &StageManifestV1) -> Result<(), BijuxError> {
    if manifest.schema_version != "bijux.stage.v1" {
        return Err(BijuxError::Manifest(format!(
            "invalid schema_version for stage at {}",
            path.display()
        )));
    }
    if manifest.stage_id.trim().is_empty() {
        return Err(BijuxError::Manifest(format!(
            "empty stage_id at {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_tool_manifest(path: &Path, manifest: &ToolManifestV1) -> Result<(), BijuxError> {
    if manifest.schema_version != "bijux.tool.v1" {
        return Err(BijuxError::Manifest(format!(
            "invalid schema_version for tool at {}",
            path.display()
        )));
    }
    if manifest.stage_id.trim().is_empty() || manifest.tool_id.trim().is_empty() {
        return Err(BijuxError::Manifest(format!(
            "empty stage_id or tool_id at {}",
            path.display()
        )));
    }
    if manifest.execution_contract.required_inputs.is_empty() {
        return Err(BijuxError::Manifest(format!(
            "execution_contract.required_inputs empty at {}",
            path.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub run_id: RunId,
    pub stage: StageManifestV1,
    pub tool: ToolManifestV1,
    pub params: BTreeMap<String, String>,
    pub container: ContainerSpec,
    pub paths: PathSpec,
    pub profile: Profile,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

/// Build an execution plan from a run spec.
///
/// # Errors
/// Returns an error if the stage or tool cannot be resolved or manifests are invalid.
pub fn build_execution_plan(
    run_spec: RunSpec,
    registry: &ToolRegistry,
    profile: Profile,
    run_id: RunId,
) -> Result<ExecutionPlan, BijuxError> {
    let stage = registry
        .stages()
        .get(&run_spec.stage.0)
        .ok_or_else(|| BijuxError::Manifest(format!("unknown stage_id {}", run_spec.stage.0)))?
        .clone();

    let tool = registry
        .tool_by_id(&run_spec.stage.0, &run_spec.tool.0)
        .ok_or_else(|| {
            BijuxError::Manifest(format!(
                "unknown tool_id {} for stage {}",
                run_spec.tool.0, run_spec.stage.0
            ))
        })?
        .clone();

    if tool.stage_id != stage.stage_id {
        return Err(BijuxError::Manifest(format!(
            "tool {} references stage {}, expected {}",
            tool.tool_id, tool.stage_id, stage.stage_id
        )));
    }

    let run_dir = run_dir(
        &profile.run_base_dir,
        &run_id,
        &run_spec.stage,
        &run_spec.tool,
    );
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");
    let tmp_dir = run_dir.join("tmp");

    let container = resolve_container_spec(&tool, &run_spec.paths, &tmp_dir, &profile)?;

    Ok(ExecutionPlan {
        run_id,
        stage,
        tool,
        params: run_spec.params,
        container,
        paths: run_spec.paths,
        profile,
        run_dir,
        logs_dir,
        artifacts_dir,
        tmp_dir,
    })
}

/// Resolve container information from a tool manifest and profile.
///
/// # Errors
/// Returns an error if the container digest is missing or malformed.
pub fn resolve_container_spec(
    tool: &ToolManifestV1,
    paths: &PathSpec,
    tmp_dir: &Path,
    profile: &Profile,
) -> Result<ContainerSpec, BijuxError> {
    if !tool.container.digest.starts_with("sha256:") {
        return Err(BijuxError::Manifest(format!(
            "container digest must be sha256 for tool {}",
            tool.tool_id
        )));
    }
    let image = format!("{}@{}", tool.container.image, tool.container.digest);

    let mut mounts = BTreeMap::new();
    mounts.insert("/data/input".to_string(), path_list_to_mount(&paths.input));
    mounts.insert(
        "/data/output".to_string(),
        path_list_to_mount(&paths.output),
    );
    mounts.insert(
        "/data/tmp".to_string(),
        tmp_dir.to_string_lossy().to_string(),
    );

    let mut env = BTreeMap::new();
    env.insert("THREADS".to_string(), profile.default_threads.to_string());
    env.insert("TMPDIR".to_string(), "/data/tmp".to_string());

    Ok(ContainerSpec {
        image,
        runtime: profile.container_runtime.clone(),
        mounts,
        env,
    })
}

fn path_list_to_mount(paths: &[PathBuf]) -> String {
    let mut unique = BTreeSet::new();
    for path in paths {
        if let Some(parent) = path.parent() {
            unique.insert(parent.to_path_buf());
        }
    }
    if unique.is_empty() {
        String::new()
    } else {
        unique
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// Create run directories for a plan.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn ensure_run_dirs(plan: &ExecutionPlan) -> Result<(), BijuxError> {
    std::fs::create_dir_all(&plan.logs_dir)?;
    std::fs::create_dir_all(&plan.artifacts_dir)?;
    std::fs::create_dir_all(&plan.tmp_dir)?;
    Ok(())
}

pub trait Executor {
    /// Execute the plan.
    ///
    /// # Errors
    /// Returns an error if execution fails.
    fn run(&self, plan: &ExecutionPlan) -> Result<RunReport, BijuxError>;
}
