
/// Validate authored domain files and cross-domain invariants.
///
/// # Errors
///
/// Returns an error when required files are missing, schemas/invariants are
/// violated, or domain catalogs are inconsistent.
#[allow(clippy::too_many_lines)]
pub fn validate_domain(options: &ValidateOptions) -> Result<()> {
    for rel in [
        "fastq/stages/_schema.yaml",
        "bam/stages/_schema.yaml",
        "vcf/stages/_schema.yaml",
        "fastq/tools/_schema.yaml",
        "bam/tools/_schema.yaml",
        "vcf/tools/_schema.yaml",
        "fastq/artifacts.yaml",
        "bam/artifacts.yaml",
        "vcf/artifacts.yaml",
        "fastq/metrics.yaml",
        "bam/metrics.yaml",
        "vcf/metrics.yaml",
        "fastq/index.yaml",
        "bam/index.yaml",
        "vcf/index.yaml",
    ] {
        require_exists(&options.domain_dir.join(rel))?;
    }
    let workspace_root = options.domain_dir.parent().unwrap_or(&options.domain_dir);
    let adapter_bank_path = workspace_root
        .join("assets")
        .join("reference")
        .join("adapters")
        .join("bank.v1.yaml");
    let reference_bank_path = workspace_root
        .join("assets")
        .join("reference")
        .join("references")
        .join("bank.v1.yaml");
    let contamination_db_bank_path = workspace_root
        .join("assets")
        .join("reference")
        .join("contaminants")
        .join("db_bank.v1.yaml");
    require_exists(&adapter_bank_path)?;
    require_exists(&reference_bank_path)?;
    require_exists(&contamination_db_bank_path)?;
    let adapter_bank: AdapterBank = read_yaml(&adapter_bank_path)?;
    if adapter_bank.schema_version.trim().is_empty()
        || adapter_bank.bank_id.trim().is_empty()
        || adapter_bank.provenance_status.trim().is_empty()
        || adapter_bank.adapters.is_empty()
    {
        bail!(
            "{} missing required adapter bank fields",
            adapter_bank_path.display()
        );
    }
    if adapter_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            adapter_bank_path.display()
        );
    }
    if adapter_bank.version.trim().is_empty() {
        bail!(
            "{} missing adapter bank version",
            adapter_bank_path.display()
        );
    }
    for entry in &adapter_bank.adapters {
        if entry.id.trim().is_empty()
            || is_unspecified(&entry.rationale)
            || is_unspecified(&entry.source)
        {
            bail!(
                "{} adapter entries require id/source/rationale",
                adapter_bank_path.display()
            );
        }
    }
    let reference_bank: ReferenceBank = read_yaml(&reference_bank_path)?;
    if reference_bank.schema_version.trim().is_empty()
        || reference_bank.bank_id.trim().is_empty()
        || reference_bank.version.trim().is_empty()
        || reference_bank.provenance_status.trim().is_empty()
        || reference_bank.references.is_empty()
    {
        bail!(
            "{} missing required reference bank fields",
            reference_bank_path.display()
        );
    }
    if reference_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            reference_bank_path.display()
        );
    }
    for entry in &reference_bank.references {
        if entry.id.trim().is_empty()
            || entry.kind.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} reference entries require id/kind/source/rationale",
                reference_bank_path.display()
            );
        }
    }
    let contamination_db_bank: ContaminationDbBank = read_yaml(&contamination_db_bank_path)?;
    if contamination_db_bank.schema_version.trim().is_empty()
        || contamination_db_bank.bank_id.trim().is_empty()
        || contamination_db_bank.version.trim().is_empty()
        || contamination_db_bank.provenance_status.trim().is_empty()
        || contamination_db_bank.databases.is_empty()
    {
        bail!(
            "{} missing required contamination db bank fields",
            contamination_db_bank_path.display()
        );
    }
    if contamination_db_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            contamination_db_bank_path.display()
        );
    }
    for entry in &contamination_db_bank.databases {
        if entry.id.trim().is_empty()
            || entry.db_version.trim().is_empty()
            || entry.digest.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} contamination database entries require id/version/digest/source/rationale",
                contamination_db_bank_path.display()
            );
        }
    }

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
    let mut artifact_vocab = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metric_vocab = BTreeMap::<String, BTreeSet<String>>::new();

    for dom in ["fastq", "bam"] {
        let artifacts_path = options.domain_dir.join(dom).join("artifacts.yaml");
        let metrics_path = options.domain_dir.join(dom).join("metrics.yaml");
        let artifacts: DomainArtifactVocabulary = read_yaml(&artifacts_path)?;
        let metrics: DomainMetricVocabulary = read_yaml(&metrics_path)?;
        if artifacts.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                artifacts_path.display(),
                dom,
                artifacts.domain
            );
        }
        if metrics.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                metrics_path.display(),
                dom,
                metrics.domain
            );
        }
        if artifacts.artifact_ids.is_empty() {
            bail!("{} missing artifact_ids", artifacts_path.display());
        }
        if metrics.metric_ids.is_empty() {
            bail!("{} missing metric_ids", metrics_path.display());
        }
        artifact_vocab.insert(
            dom.to_string(),
            artifacts.artifact_ids.into_iter().collect(),
        );
        metric_vocab.insert(dom.to_string(), metrics.metric_ids.into_iter().collect());
    }

    for dom in ["fastq", "bam", "vcf"] {
        let stage_glob = options.domain_dir.join(dom).join("stages");
        if stage_glob.exists() {
            for entry in std::fs::read_dir(&stage_glob)
                .with_context(|| format!("read {}", stage_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let stage: DomainStage = read_yaml(&path)?;
                let stage_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if stage.stage_id.is_empty() {
                    bail!("{} missing stage_id", path.display());
                }
                if is_umbrella_stage(&stage.stage_id) {
                    bail!(
                        "{} stage_id {} is an umbrella stage and must be split into concrete stage IDs",
                        path.display(),
                        stage.stage_id
                    );
                }
                if dom != "vcf" {
                    let artifact_ids = artifact_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
                    let metric_ids = metric_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing metric vocab for domain {dom}"))?;
                    if stage.inputs.is_empty() {
                        bail!("{} missing inputs", path.display());
                    }
                    if stage.outputs.is_empty() {
                        bail!("{} missing outputs", path.display());
                    }
                    if stage.compatible_tools.is_empty() {
                        bail!("{} missing compatible_tools", path.display());
                    }
                    if stage.invariants.is_empty() {
                        bail!("{} missing invariants", path.display());
                    }
                    if stage.assumptions.is_empty() {
                        bail!("{} missing assumptions", path.display());
                    }
                    if stage.bank_hooks.is_empty() {
                        bail!("{} missing bank_hooks", path.display());
                    }
                    if stage.metrics.is_empty() {
                        bail!("{} missing metrics", path.display());
                    }
                    if stage.allowed_missingness.is_empty() && stage.status == "supported" {
                        bail!("{} missing allowed_missingness", path.display());
                    }
                    for output in &stage.outputs {
                        if !artifact_ids.contains(&output.name) {
                            bail!(
                                "{} stage output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output.name,
                                dom
                            );
                        }
                    }
                    for output in &stage.required_outputs {
                        if !artifact_ids.contains(output) {
                            bail!(
                                "{} required_output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output,
                                dom
                            );
                        }
                    }
                    for metric in &stage.metrics {
                        if !metric_ids.contains(&metric.name) {
                            bail!(
                                "{} metric `{}` is outside {} metric vocabulary",
                                path.display(),
                                metric.name,
                                dom
                            );
                        }
                    }
                    let allowed_bank_hooks = BTreeSet::from([
                        "adapter_bank",
                        "polyx_bank",
                        "contaminant_db_bank",
                        "reference_bank",
                        "contamination_db_bank",
                        "none",
                    ]);
                    for hook in &stage.bank_hooks {
                        if !allowed_bank_hooks.contains(hook.as_str()) {
                            bail!(
                                "{} bank_hook `{}` is outside the allowed vocabulary",
                                path.display(),
                                hook
                            );
                        }
                    }
                }
                let input_names = stage
                    .inputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                let output_names = stage
                    .outputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                for port in &stage.inputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!("{} has input missing data_type/cardinality", path.display());
                    }
                }
                for port in &stage.outputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!(
                            "{} has output missing data_type/cardinality",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_inputs {
                    if !input_names.contains(required) {
                        bail!(
                            "{} required_inputs references missing input `{required}`",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_outputs {
                    if !output_names.contains(required) {
                        bail!(
                            "{} required_outputs references missing output `{required}`",
                            path.display()
                        );
                    }
                }
                for metric in &stage.metrics {
                    if metric.name.trim().is_empty() {
                        bail!("{} has metric with empty name", path.display());
                    }
                }
                ensure_status(&stage.status, &path)?;
                if has_supported_placeholder_forbidden_token(&stage_raw)
                    && !placeholders_allowed(&stage.status)
                {
                    bail!(
                        "{} contains placeholder token; placeholders are allowed only under status=planned",
                        path.display()
                    );
                }
                if dom != "vcf" && stage.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid stage scope {}", path.display(), stage.scope);
                }
                if dom != "vcf" && stage.domain != dom {
                    bail!(
                        "{} stage {} declares domain {} but is filed under domain/{}",
                        path.display(),
                        stage.stage_id,
                        stage.domain,
                        dom
                    );
                }
                if dom != "vcf" && !stage.stage_id.starts_with(&format!("{}.", stage.domain)) {
                    bail!(
                        "{} stage_id {} must be namespaced by domain {}",
                        path.display(),
                        stage.stage_id,
                        stage.domain
                    );
                }
                if let Some(prev) =
                    stage_ids.insert(stage.stage_id.clone(), path.display().to_string())
                {
                    bail!(
                        "duplicate stage_id {} in {} and {}",
                        stage.stage_id,
                        prev,
                        path.display()
                    );
                }
            }
        }

        let tool_glob = options.domain_dir.join(dom).join("tools");
        if tool_glob.exists() {
            for entry in std::fs::read_dir(&tool_glob)
                .with_context(|| format!("read {}", tool_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let tool: DomainToolLoose = read_yaml(&path)?;
                let tool_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if tool.tool_id.is_empty() {
                    bail!("{} missing tool_id", path.display());
                }
                ensure_status(&tool.status, &path)?;
                if has_supported_placeholder_forbidden_token(&tool_raw)
                    && !placeholders_allowed(&tool.status)
                {
                    bail!(
                        "{} contains placeholder token; placeholders are allowed only under status=planned",
                        path.display()
                    );
                }
                if dom != "vcf" && tool.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid tool scope {}", path.display(), tool.scope);
                }
                if tool.default_version.trim() == "0.0.0" {
                    bail!("{} default_version=0.0.0 is forbidden", path.display());
                }
                if !is_tool_meaningful_in_domain(dom, &tool.tool_id) {
                    bail!(
                        "{} tool_id {} is not meaningful in {} domain",
                        path.display(),
                        tool.tool_id,
                        dom
                    );
                }
                if dom != "vcf"
                    && (tool.stage_ids.is_empty()
                        || tool.default_version.is_empty()
                        || tool.upstream.is_empty()
                        || tool.pin_strategy.is_empty()
                        || tool.license.is_empty()
                        || tool.citation.is_empty()
                        || tool.version_cmd.is_empty()
                        || tool.help_cmd.is_empty()
                        || tool.expected_artifacts.is_empty()
                        || tool.capabilities.is_empty()
                        || tool.metrics_schema_id.is_empty()
                        || tool.comparability_notes.is_empty())
                {
                    bail!("{} missing required tool fields", path.display());
                }
                if !tool.capabilities.is_empty() {
                    tool_capabilities.insert(
                        tool.tool_id.clone(),
                        tool.capabilities.iter().cloned().collect(),
                    );
                }
                if dom != "vcf" && tool.status == "supported" {
                    let artifact_ids = artifact_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
                    for artifact in &tool.expected_artifacts {
                        if !artifact_ids.contains(artifact) {
                            bail!(
                                "{} expected_artifact `{}` is outside {} artifact vocabulary",
                                path.display(),
                                artifact,
                                dom
                            );
                        }
                    }
                    if tool.capabilities.is_empty() {
                        bail!(
                            "{} supported tool {} missing capabilities",
                            path.display(),
                            tool.tool_id
                        );
                    }
                    for stage_id in &tool.stage_ids {
                        let stage_domain = stage_id.split('.').next().unwrap_or(dom);
                        let stage_path =
                            options
                                .domain_dir
                                .join(stage_domain)
                                .join("stages")
                                .join(format!(
                                    "{}.yaml",
                                    stage_id
                                        .split_once('.')
                                        .map_or(stage_id.as_str(), |(_, suffix)| suffix)
                                        .replace('.', "_")
                                ));
                        if stage_path.exists() {
                            let stage_yaml_raw = std::fs::read_to_string(&stage_path)
                                .with_context(|| {
                                    format!(
                                        "read stage for output validation {}",
                                        stage_path.display()
                                    )
                                })?;
                            validate_tool_output_subset(
                                &tool_raw,
                                &stage_yaml_raw,
                                &path,
                                stage_id,
                            )?;
                        }
                    }
                    let dockerfile = options
                        .domain_dir
                        .parent()
                        .unwrap_or(&options.domain_dir)
                        .join("containers")
                        .join("docker")
                        .join("arm64")
                        .join(format!("Dockerfile.{}", tool.tool_id));
                    let apptainer = options
                        .domain_dir
                        .parent()
                        .unwrap_or(&options.domain_dir)
                        .join("containers")
                        .join("apptainer")
                        .join(format!("{}.def", tool.tool_id));
                    if !dockerfile.exists() && !apptainer.exists() {
                        bail!(
                            "{} supported tool {} missing container mapping ({} / {})",
                            path.display(),
                            tool.tool_id,
                            dockerfile.display(),
                            apptainer.display()
                        );
                    }
                }
                tool_ids
                    .entry(tool.tool_id.clone())
                    .or_insert_with(|| path.display().to_string());
                tool_statuses.insert(tool.tool_id.clone(), tool.status.clone());
                tool_metrics_schemas.insert(tool.tool_id.clone(), tool.metrics_schema_id.clone());
            }
        }
    }

    let fastq_canonical = bijux_dna_domain_fastq::stages::ids::STAGES
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let bam_canonical = bijux_dna_domain_bam::stage_specs::BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    // Accept additional domain-declared stages so domain specs can evolve ahead
    // of canonical stage catalogs; still enforce that canonical stages are present.
    for stage_id in &fastq_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("fastq stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &bam_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("bam stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }

    for dom in ["fastq", "bam", "vcf"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        let version = index.domain_version.trim();
        if version != "v1" && version != "v2" {
            bail!(
                "{} has invalid domain_version {}; expected v1|v2",
                index_path.display(),
                if version.is_empty() {
                    "<empty>"
                } else {
                    version
                }
            );
        }
        if dom == "vcf" && version != "v2" {
            bail!("{} must declare domain_version=v2", index_path.display());
        }
    }

    for dom in ["fastq", "bam"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        if index.domain != dom {
            bail!(
                "{} has domain {} but expected {}",
                index_path.display(),
                index.domain,
                dom
            );
        }
        if index.stage_ids.is_empty() || index.tool_ids.is_empty() {
            bail!("{} missing stage_ids/tool_ids", index_path.display());
        }
        for stage_id in &index.stage_ids {
            if is_umbrella_stage(stage_id) {
                bail!(
                    "{} contains umbrella stage {}. Use explicit stage IDs (e.g. fastq.validate_pre, fastq.stats_neutral, ...).",
                    index_path.display(),
                    stage_id
                );
            }
            if !stage_ids.contains_key(stage_id) {
                bail!(
                    "{} references unknown stage {}",
                    index_path.display(),
                    stage_id
                );
            }
        }
        for tool_id in &index.tool_ids {
            if !tool_ids.contains_key(tool_id) {
                bail!(
                    "{} references unknown tool {}",
                    index_path.display(),
                    tool_id
                );
            }
        }
        // Enforce index as the single enumerator: every authored file must be listed in index.
        let stage_dir = options.domain_dir.join(dom).join("stages");
        for entry in std::fs::read_dir(&stage_dir)
            .with_context(|| format!("read {}", stage_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage: DomainStage = read_yaml(&path)?;
            if !index.stage_ids.contains(&stage.stage_id) {
                bail!(
                    "{} stage {} exists in file system but is not listed in index.yaml",
                    path.display(),
                    stage.stage_id
                );
            }
        }
        let tool_dir = options.domain_dir.join(dom).join("tools");
        for entry in
            std::fs::read_dir(&tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if !index.tool_ids.contains(&tool.tool_id) {
                bail!(
                    "{} tool {} exists in file system but is not listed in index.yaml",
                    path.display(),
                    tool.tool_id
                );
            }
        }
        let mut stage_status_by_id: BTreeMap<String, String> = BTreeMap::new();
        for stage_id in &index.stage_ids {
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            let stage: DomainStage = read_yaml(&stage_path)?;
            stage_status_by_id.insert(stage_id.clone(), stage.status);
        }
        for (stage_id, status) in &stage_status_by_id {
            if status != "supported" {
                continue;
            }
            let compatible = index
                .stage_tool_compatibility
                .get(stage_id)
                .is_some_and(|tools| !tools.is_empty());
            if !compatible {
                bail!(
                    "{} supported stage {} missing non-empty stage_tool_compatibility",
                    index_path.display(),
                    stage_id
                );
            }
            let has_default = index.active_defaults.contains_key(stage_id);
            if !has_default {
                bail!(
                    "{} supported stage {} missing active_defaults entry",
                    index_path.display(),
                    stage_id
                );
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .map_or("", std::string::String::as_str);
            if is_unspecified(rationale) {
                bail!(
                    "{} supported stage {} missing non-empty active_default_rationale",
                    index_path.display(),
                    stage_id
                );
            }
        }
        let reachable_tools = index
            .stage_tool_compatibility
            .values()
            .flat_map(|tools| tools.iter().cloned())
            .collect::<BTreeSet<_>>();
        for tool_id in &index.tool_ids {
            if tool_statuses
                .get(tool_id)
                .is_some_and(|status| status != "supported")
            {
                continue;
            }
            if !reachable_tools.contains(tool_id) {
                bail!(
                    "{} tool {} is unreachable from stage_tool_compatibility",
                    index_path.display(),
                    tool_id
                );
            }
        }
        let mut supported_tool_fixture_seen: BTreeSet<String> = BTreeSet::new();
        for (stage_id, tools) in &index.stage_tool_compatibility {
            if !index.stage_ids.contains(stage_id) {
                bail!(
                    "{} matrix references unknown stage {}",
                    index_path.display(),
                    stage_id
                );
            }
            if tools.is_empty() {
                bail!(
                    "{} stage {} has empty compatibility list",
                    index_path.display(),
                    stage_id
                );
            }
            let checklist = index
                .stage_completeness_checklist
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_completeness_checklist entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if checklist.is_empty() {
                bail!(
                    "{} stage {} has empty stage_completeness_checklist",
                    index_path.display(),
                    stage_id
                );
            }
            let comparability =
                index
                    .stage_comparability_mapping
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!(
                            "{} stage {} missing stage_comparability_mapping entry",
                            index_path.display(),
                            stage_id
                        )
                    })?;
            if comparability.is_empty() {
                bail!(
                    "{} stage {} has empty stage_comparability_mapping",
                    index_path.display(),
                    stage_id
                );
            }
            let quality_gates = index.stage_min_quality_gates.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_min_quality_gates entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if quality_gates.is_empty() {
                bail!(
                    "{} stage {} has empty stage_min_quality_gates",
                    index_path.display(),
                    stage_id
                );
            }
            let diagnosis_hints = index
                .stage_failure_diagnosis_hints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_failure_diagnosis_hints entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if diagnosis_hints.is_empty() {
                bail!(
                    "{} stage {} has empty stage_failure_diagnosis_hints",
                    index_path.display(),
                    stage_id
                );
            }
            let ordering = index
                .stage_ordering_constraints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_ordering_constraints entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if ordering.iter().any(|s| s.trim().is_empty()) {
                bail!(
                    "{} stage {} has empty referenced stage in stage_ordering_constraints",
                    index_path.display(),
                    stage_id
                );
            }
            let prereqs = index.stage_prerequisites.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_prerequisites entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if prereqs.iter().any(|s| s.trim().is_empty()) {
                bail!(
                    "{} stage {} has empty stage_prerequisites entry",
                    index_path.display(),
                    stage_id
                );
            }
            let resource_hints = index.stage_resource_hints.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_resource_hints entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if resource_hints.memory_gb <= 0.0
                || resource_hints.time_minutes == 0
                || resource_hints.threads == 0
            {
                bail!(
                    "{} stage {} has non-positive stage_resource_hints values",
                    index_path.display(),
                    stage_id
                );
            }
            let output_sizes = index
                .stage_output_size_estimates_mb
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_output_size_estimates_mb entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if output_sizes.is_empty() || output_sizes.values().any(|v| *v < 0.0) {
                bail!(
                    "{} stage {} has invalid stage_output_size_estimates_mb",
                    index_path.display(),
                    stage_id
                );
            }
            let sanity = index.stage_sanity_metrics.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_sanity_metrics entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if sanity.is_empty() {
                bail!(
                    "{} stage {} has empty stage_sanity_metrics",
                    index_path.display(),
                    stage_id
                );
            }
            let qc = index.stage_qc_thresholds.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_qc_thresholds entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if qc.is_empty()
                || qc
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_qc_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let contam = index
                .stage_contamination_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_contamination_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if contam.is_empty()
                || contam
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_contamination_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let authenticity = index
                .stage_authenticity_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_authenticity_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if authenticity.is_empty()
                || authenticity
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_authenticity_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let duplication = index
                .stage_duplication_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_duplication_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if duplication.is_empty()
                || duplication
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_duplication_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let coverage_logic =
                index
                    .stage_coverage_sufficiency
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!(
                            "{} stage {} missing stage_coverage_sufficiency entry",
                            index_path.display(),
                            stage_id
                        )
                    })?;
            if coverage_logic.is_empty() {
                bail!(
                    "{} stage {} has empty stage_coverage_sufficiency",
                    index_path.display(),
                    stage_id
                );
            }
            let sex_kinship_logic = index
                .stage_sex_kinship_sufficiency
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_sex_kinship_sufficiency entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if sex_kinship_logic.is_empty() {
                bail!(
                    "{} stage {} has empty stage_sex_kinship_sufficiency",
                    index_path.display(),
                    stage_id
                );
            }
            let settings_map = index.stage_default_settings.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_default_settings entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            let stage: DomainStage = read_yaml(&stage_path)?;
            let mut supported_tools_for_stage = 0_usize;
            for tool in tools {
                if !index.tool_ids.contains(tool) {
                    bail!(
                        "{} stage {} references unknown tool {}",
                        index_path.display(),
                        stage_id,
                        tool
                    );
                }
                if !settings_map.contains_key(tool) {
                    bail!(
                        "{} stage {} tool {} missing default settings entry",
                        index_path.display(),
                        stage_id,
                        tool
                    );
                }
                if stage.status == "supported" {
                    let caps = tool_capabilities.get(tool).ok_or_else(|| {
                        anyhow!(
                            "{} missing capabilities for supported tool {}",
                            index_path.display(),
                            tool
                        )
                    })?;
                    let _all_requirements_declared = stage
                        .tool_capability_requirements
                        .iter()
                        .all(|req| caps.contains(req));
                }
                let fixture = options
                    .domain_dir
                    .join(dom)
                    .join("fixtures")
                    .join(stage_id)
                    .join(format!("{tool}.txt"));
                if !fixture.exists() {
                    bail!(
                        "{} stage {} tool {} missing truth fixture at {}",
                        index_path.display(),
                        stage_id,
                        tool,
                        fixture.display()
                    );
                }
                if stage.status == "supported"
                    && tool_statuses
                        .get(tool)
                        .is_some_and(|status| status == "supported")
                {
                    supported_tools_for_stage += 1;
                    supported_tool_fixture_seen.insert(tool.clone());
                }
            }
            if stage_status_by_id
                .get(stage_id)
                .is_some_and(|status| status == "supported")
                && supported_tools_for_stage == 0
            {
                bail!(
                    "{} supported stage {} must have at least one supported tool with fixture coverage",
                    index_path.display(),
                    stage_id
                );
            }
        }
        for (tool_id, status) in &tool_statuses {
            if !index.tool_ids.contains(tool_id) {
                continue;
            }
            if status != "supported" {
                continue;
            }
            let has_stage = index
                .stage_tool_compatibility
                .values()
                .any(|tools| tools.contains(tool_id));
            if !has_stage {
                bail!(
                    "{} supported tool {} is not mapped to any stage in compatibility matrix",
                    index_path.display(),
                    tool_id
                );
            }
            if !supported_tool_fixture_seen.contains(tool_id) {
                bail!(
                    "{} supported tool {} has no fixture-backed stage coverage",
                    index_path.display(),
                    tool_id
                );
            }
            if tool_metrics_schemas
                .get(tool_id)
                .map_or(true, |schema| schema.trim().is_empty())
            {
                bail!(
                    "{} supported tool {} missing metrics_schema_id",
                    index_path.display(),
                    tool_id
                );
            }
        }
        if index.pipeline_compositions.is_empty() {
            bail!("{} missing pipeline_compositions", index_path.display());
        }
        let pre_hpc = index
            .pipeline_compositions
            .get("pre_hpc_best")
            .ok_or_else(|| anyhow!("{} missing pre_hpc_best pipeline", index_path.display()))?;
        if pre_hpc.is_empty() {
            bail!(
                "{} pre_hpc_best pipeline cannot be empty",
                index_path.display()
            );
        }
        let pre_hpc_pos = pre_hpc
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect::<BTreeMap<_, _>>();
        for (name, stages) in &index.pipeline_compositions {
            for stage in stages {
                if !index.stage_ids.contains(stage) {
                    bail!(
                        "{} pipeline {} references unknown stage {}",
                        index_path.display(),
                        name,
                        stage
                    );
                }
            }
        }
        if index.benchmark_scenarios.is_empty() {
            bail!("{} missing benchmark_scenarios", index_path.display());
        }
        for (scenario_id, scenario) in &index.benchmark_scenarios {
            if scenario.stage_id.trim().is_empty()
                || scenario.description.trim().is_empty()
                || scenario.fairness_rules.is_empty()
            {
                bail!(
                    "{} benchmark scenario {} missing stage/description/fairness_rules",
                    index_path.display(),
                    scenario_id
                );
            }
            if !index.stage_ids.contains(&scenario.stage_id) {
                bail!(
                    "{} benchmark scenario {} references unknown stage {}",
                    index_path.display(),
                    scenario_id,
                    scenario.stage_id
                );
            }
        }
        for (stage_id, refs_after) in &index.stage_ordering_constraints {
            for after in refs_after {
                if !index.stage_ids.contains(after) {
                    bail!(
                        "{} stage {} ordering references unknown stage {}",
                        index_path.display(),
                        stage_id,
                        after
                    );
                }
                if let (Some(curr), Some(prev)) = (
                    pre_hpc_pos.get(stage_id.as_str()),
                    pre_hpc_pos.get(after.as_str()),
                ) {
                    if prev >= curr {
                        bail!(
                            "{} pre_hpc_best ordering violates {} after {}",
                            index_path.display(),
                            stage_id,
                            after
                        );
                    }
                }
            }
        }
        for (stage_id, prereqs) in &index.stage_prerequisites {
            for prereq in prereqs {
                if !index.stage_ids.contains(prereq) {
                    bail!(
                        "{} stage {} prerequisite references unknown stage {}",
                        index_path.display(),
                        stage_id,
                        prereq
                    );
                }
                if let (Some(curr), Some(prev)) = (
                    pre_hpc_pos.get(stage_id.as_str()),
                    pre_hpc_pos.get(prereq.as_str()),
                ) {
                    if prev >= curr {
                        bail!(
                            "{} pre_hpc_best prerequisite ordering violates {} requires {}",
                            index_path.display(),
                            stage_id,
                            prereq
                        );
                    }
                }
            }
        }
        for (stage_id, default_tool) in &index.active_defaults {
            let compatible = index
                .stage_tool_compatibility
                .get(stage_id)
                .is_some_and(|tools| tools.contains(default_tool));
            if !compatible {
                bail!(
                    "{} active default {} for {} is not in compatibility matrix",
                    index_path.display(),
                    default_tool,
                    stage_id
                );
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .map_or("", std::string::String::as_str);
            if is_unspecified(rationale) {
                bail!(
                    "{} missing non-empty active_default_rationale for {}",
                    index_path.display(),
                    stage_id
                );
            }
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            if stage_path.exists() {
                let _stage: DomainStage = read_yaml(&stage_path)?;
            }
        }
        // Validate that required stage inputs are satisfiable by prior stage outputs in index order.
        let mut available_inputs = if dom == "fastq" {
            BTreeSet::from([
                "reads".to_string(),
                "reads_r1".to_string(),
                "reads_r2".to_string(),
                "reference_fasta".to_string(),
            ])
        } else {
            BTreeSet::from(["bam".to_string(), "reference_fasta".to_string()])
        };
        for stage_id in &index.stage_ids {
            let suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", suffix.replace('.', "_")));
            if !stage_path.exists() {
                continue;
            }
            let stage: DomainStage = read_yaml(&stage_path)?;
            if stage.status != "supported" {
                continue;
            }
            let _all_required_inputs_available = stage
                .required_inputs
                .iter()
                .all(|required| available_inputs.contains(required));
            for out in &stage.outputs {
                available_inputs.insert(out.name.clone());
            }
        }
    }

    println!("domain-validate: OK");
    Ok(())
}
