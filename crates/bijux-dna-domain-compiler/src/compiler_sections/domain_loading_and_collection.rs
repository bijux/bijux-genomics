#[allow(clippy::too_many_lines)]
fn load_domain_tools(
    domain_dir: &Path,
    domain: &str,
    index: &DomainIndex,
    active_scope: &str,
    tools: &mut ToolMap,
    stage_to_tools: &mut StageToolMap,
) -> Result<()> {
    let tools_dir = domain_dir.join(domain).join("tools");
    for tool_id_ref in &index.tool_ids {
        let tool_id_normalized = tool_id_ref.replace('-', "_");
        let path_candidates = [
            tools_dir.join(format!("{tool_id_ref}.yaml")),
            tools_dir.join(format!("{tool_id_normalized}.yaml")),
            domain_dir
                .join(if domain == "fastq" { "bam" } else { "fastq" })
                .join("tools")
                .join(format!("{tool_id_ref}.yaml")),
            domain_dir
                .join(if domain == "fastq" { "bam" } else { "fastq" })
                .join("tools")
                .join(format!("{tool_id_normalized}.yaml")),
        ];
        let Some(path) = path_candidates.into_iter().find(|p| p.exists()) else {
            return Err(anyhow!(
                "index references missing tool file for {} in {}",
                tool_id_ref,
                tools_dir.display()
            ));
        };
        let tool: DomainTool = read_yaml(&path)?;
        let tool_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if tool.tool_id.trim().is_empty() {
            return Err(anyhow!("{} missing tool_id", path.display()));
        }
        if tool.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
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
        if !scope_active(&tool.scope, active_scope) || tool.status != "supported" {
            continue;
        }
        if tool.stage_ids.is_empty() {
            return Err(anyhow!("{} missing stage_ids", path.display()));
        }
        if tool.upstream.trim().is_empty()
            || tool.default_version.trim().is_empty()
            || tool.versioning_strategy.trim().is_empty()
            || tool.license.trim().is_empty()
            || tool.citation.trim().is_empty()
            || tool.version_cmd.trim().is_empty()
            || tool.help_cmd.trim().is_empty()
            || tool.expected_artifacts.is_empty()
            || (tool.status == "supported" && tool.capabilities.is_empty())
            || (tool.metrics_schema_id.trim().is_empty() && tool.metrics_schema.trim().is_empty())
        {
            return Err(anyhow!("{} missing required tool fields", path.display()));
        }
        for stage in &tool.stage_ids {
            stage_to_tools
                .entry(stage.clone())
                .or_default()
                .insert(tool.tool_id.clone());
        }
        let tool_id = tool.tool_id.clone();
        if tools.contains_key(&tool_id) {
            continue;
        }
        let version_rule = tool.versioning_strategy.clone();
        let help_cmd_value = tool.help_cmd.clone();
        let resolved_domain = path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or(domain)
            .to_string();
        let mut domains = tool
            .stage_ids
            .iter()
            .filter_map(|stage_id| stage_id.split('.').next().map(str::to_string))
            .collect::<Vec<_>>();
        domains.sort();
        domains.dedup();
        let mut bindings = tool.stage_ids.clone();
        bindings.sort();
        bindings.dedup();
        let tool_role = infer_tool_role(&bindings);
        tools.insert(
            tool_id.clone(),
            ToolRow {
                id: tool_id.clone(),
                domain: resolved_domain,
                domains,
                stage_ids: tool.stage_ids,
                bindings,
                tool_role,
                default_version: tool.default_version,
                upstream: tool.upstream,
                pin_strategy: if tool.pin_strategy.is_empty() {
                    version_rule.clone()
                } else {
                    tool.pin_strategy
                },
                version_cmd: tool.version_cmd,
                help_cmd: help_cmd_value.clone(),
                expected_artifacts: tool.expected_artifacts,
                metrics_schema: if tool.metrics_schema_id.is_empty() {
                    tool.metrics_schema
                } else {
                    tool.metrics_schema_id
                },
                status: tool.status,
                comparability_notes: tool.comparability_notes,
                version_rule,
                license: tool.license,
                citation: tool.citation,
                container_image: tool
                    .container
                    .as_ref()
                    .map_or_else(String::new, |container| container.image.clone()),
                container_digest: tool
                    .container
                    .as_ref()
                    .map_or_else(String::new, |container| container.digest.clone()),
                expected_version_regex: default_version_regex(&tool_id).to_string(),
                healthcheck_cmd: default_healthcheck_cmd(&tool_id, &help_cmd_value),
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn load_domain_stages(
    domain_dir: &Path,
    domain: &str,
    index: &DomainIndex,
    active_scope: &str,
    stage_to_tools: &mut StageToolMap,
    stage_planned: &mut StagePlannedMap,
    stage_statuses: &mut StageStatusMap,
    stage_output_kinds: &mut StageOutputKindsMap,
) -> Result<()> {
    let stages_dir = domain_dir.join(domain).join("stages");
    for stage_id in &index.stage_ids {
        let stage_suffix = stage_id
            .as_str()
            .split_once('.')
            .map_or(stage_id.as_str(), |(_, suffix)| suffix);
        let stage_file = stage_suffix.replace('.', "_");
        let path = stages_dir.join(format!("{stage_file}.yaml"));
        if !path.exists() {
            return Err(anyhow!(
                "index references missing stage file for {} at {}",
                stage_id,
                path.display()
            ));
        }
        let stage: DomainStage = read_yaml(&path)?;
        let stage_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if stage.stage_id.trim().is_empty() {
            return Err(anyhow!("{} missing stage_id", path.display()));
        }
        if stage.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
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
        if !scope_active(&stage.scope, active_scope) || stage.status != "supported" {
            continue;
        }
        stage_to_tools.entry(stage.stage_id.clone()).or_default();
        let mut kinds = stage
            .outputs
            .iter()
            .map(|port| port.data_type.clone())
            .collect::<Vec<_>>();
        kinds.sort();
        kinds.dedup();
        stage_output_kinds.insert(stage.stage_id.clone(), kinds);
        stage_statuses.insert(stage.stage_id.clone(), stage.status.clone());
        stage_planned.insert(stage.stage_id, stage.planned_out_of_scope);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
fn collect_domain_data(
    domain_dir: &Path,
    active_scope: &str,
) -> Result<(
    ToolMap,
    StageToolMap,
    StagePlannedMap,
    StageDefaultMap,
    StageDefaultRationaleMap,
    StageStatusMap,
    StageOutputKindsMap,
)> {
    let mut tools: ToolMap = BTreeMap::new();
    let mut stage_to_tools: StageToolMap = BTreeMap::new();
    let mut stage_planned: StagePlannedMap = BTreeMap::new();
    let mut stage_defaults: StageDefaultMap = BTreeMap::new();
    let mut stage_default_rationale: StageDefaultRationaleMap = BTreeMap::new();
    let mut stage_statuses: StageStatusMap = BTreeMap::new();
    let mut stage_output_kinds: StageOutputKindsMap = BTreeMap::new();
    for domain in ["fastq", "bam"] {
        let index_path = domain_dir.join(domain).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        if index.domain != domain {
            return Err(anyhow!(
                "{} has domain {} but expected {}",
                index_path.display(),
                index.domain,
                domain
            ));
        }
        load_domain_tools(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut tools,
            &mut stage_to_tools,
        )?;
        load_domain_stages(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut stage_to_tools,
            &mut stage_planned,
            &mut stage_statuses,
            &mut stage_output_kinds,
        )?;
        for (stage_id, tool_ids) in &index.stage_tool_compatibility {
            if !stage_to_tools.contains_key(stage_id) {
                continue;
            }
            let active_tools = stage_to_tools.entry(stage_id.clone()).or_default();
            active_tools.clear();
            for tool_id in tool_ids {
                if tools.contains_key(tool_id) {
                    active_tools.insert(tool_id.clone());
                }
            }
        }
        for (stage_id, default_tool) in &index.active_defaults {
            if !stage_to_tools.contains_key(stage_id) {
                continue;
            }
            if !stage_to_tools
                .get(stage_id)
                .is_some_and(|set| set.contains(default_tool))
            {
                return Err(anyhow!(
                    "index active default {default_tool} for {stage_id} is not compatible"
                ));
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .cloned()
                .unwrap_or_default();
            if is_unspecified(&rationale) {
                return Err(anyhow!(
                    "index active_default_rationale for {stage_id} must be non-empty and not unspecified"
                ));
            }
            let checklist = index
                .stage_completeness_checklist
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_completeness_checklist for stage {stage_id}")
                })?;
            if checklist.is_empty() {
                return Err(anyhow!(
                    "index stage_completeness_checklist for {stage_id} must not be empty"
                ));
            }
            if checklist.iter().any(|item| item.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_completeness_checklist for {stage_id} contains empty item"
                ));
            }
            let stage_settings = index.stage_default_settings.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_default_settings for stage {stage_id}")
            })?;
            if !stage_settings.contains_key(default_tool) {
                return Err(anyhow!(
                    "index stage_default_settings for {stage_id} missing default tool {default_tool}"
                ));
            }
            let comparability =
                index
                    .stage_comparability_mapping
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!("index missing stage_comparability_mapping for stage {stage_id}")
                    })?;
            if comparability.is_empty() {
                return Err(anyhow!(
                    "index stage_comparability_mapping for {stage_id} must not be empty"
                ));
            }
            let quality_gates = index.stage_min_quality_gates.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_min_quality_gates for stage {stage_id}")
            })?;
            if quality_gates.is_empty() {
                return Err(anyhow!(
                    "index stage_min_quality_gates for {stage_id} must not be empty"
                ));
            }
            let diagnosis_hints = index
                .stage_failure_diagnosis_hints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_failure_diagnosis_hints for stage {stage_id}")
                })?;
            if diagnosis_hints.is_empty() {
                return Err(anyhow!(
                    "index stage_failure_diagnosis_hints for {stage_id} must not be empty"
                ));
            }
            let ordering = index
                .stage_ordering_constraints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_ordering_constraints for stage {stage_id}")
                })?;
            if ordering.iter().any(|s| s.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_ordering_constraints for {stage_id} contains empty stage id"
                ));
            }
            let prereqs = index
                .stage_prerequisites
                .get(stage_id)
                .ok_or_else(|| anyhow!("index missing stage_prerequisites for stage {stage_id}"))?;
            if prereqs.iter().any(|s| s.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_prerequisites for {stage_id} contains empty prerequisite"
                ));
            }
            let resources = index.stage_resource_hints.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_resource_hints for stage {stage_id}")
            })?;
            if resources.memory_gb <= 0.0 || resources.time_minutes == 0 || resources.threads == 0 {
                return Err(anyhow!(
                    "index stage_resource_hints for {stage_id} must define positive memory_gb/time_minutes/threads"
                ));
            }
            let size_estimates = index
                .stage_output_size_estimates_mb
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_output_size_estimates_mb for stage {stage_id}")
                })?;
            if size_estimates.is_empty() {
                return Err(anyhow!(
                    "index stage_output_size_estimates_mb for {stage_id} must not be empty"
                ));
            }
            if size_estimates.values().any(|v| *v < 0.0) {
                return Err(anyhow!(
                    "index stage_output_size_estimates_mb for {stage_id} contains negative estimate"
                ));
            }
            let sanity = index.stage_sanity_metrics.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_sanity_metrics for stage {stage_id}")
            })?;
            if sanity.is_empty() {
                return Err(anyhow!(
                    "index stage_sanity_metrics for {stage_id} must not be empty"
                ));
            }
            let qc = index
                .stage_qc_thresholds
                .get(stage_id)
                .ok_or_else(|| anyhow!("index missing stage_qc_thresholds for stage {stage_id}"))?;
            if qc.is_empty()
                || qc
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_qc_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let contam = index
                .stage_contamination_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_contamination_thresholds for stage {stage_id}")
                })?;
            if contam.is_empty()
                || contam
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_contamination_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let authenticity = index
                .stage_authenticity_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_authenticity_thresholds for stage {stage_id}")
                })?;
            if authenticity.is_empty()
                || authenticity
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_authenticity_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let duplication = index
                .stage_duplication_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_duplication_thresholds for stage {stage_id}")
                })?;
            if duplication.is_empty()
                || duplication
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_duplication_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let coverage_logic =
                index
                    .stage_coverage_sufficiency
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!("index missing stage_coverage_sufficiency for stage {stage_id}")
                    })?;
            if coverage_logic.is_empty() {
                return Err(anyhow!(
                    "index stage_coverage_sufficiency for {stage_id} must not be empty"
                ));
            }
            let sex_kinship_logic = index
                .stage_sex_kinship_sufficiency
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_sex_kinship_sufficiency for stage {stage_id}")
                })?;
            if sex_kinship_logic.is_empty() {
                return Err(anyhow!(
                    "index stage_sex_kinship_sufficiency for {stage_id} must not be empty"
                ));
            }
            stage_defaults.insert(stage_id.clone(), default_tool.clone());
            stage_default_rationale.insert(stage_id.clone(), rationale);
        }
        if index.pipeline_compositions.is_empty() {
            return Err(anyhow!("index missing pipeline_compositions"));
        }
        if !index.pipeline_compositions.contains_key("pre_hpc_best") {
            return Err(anyhow!(
                "index pipeline_compositions must include pre_hpc_best"
            ));
        }
        for (pipeline_name, stages) in &index.pipeline_compositions {
            if stages.is_empty() {
                return Err(anyhow!(
                    "index pipeline {pipeline_name} has empty stage list"
                ));
            }
            for s in stages {
                if !index.stage_ids.contains(s) {
                    return Err(anyhow!(
                        "index pipeline {pipeline_name} references unknown stage {s}"
                    ));
                }
            }
        }
        if index.benchmark_scenarios.is_empty() {
            return Err(anyhow!("index missing benchmark_scenarios"));
        }
        for (scenario_id, scenario) in &index.benchmark_scenarios {
            if scenario.stage_id.trim().is_empty()
                || scenario.description.trim().is_empty()
                || scenario.fairness_rules.is_empty()
            {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} missing stage/description/fairness_rules"
                ));
            }
            if !index.stage_ids.contains(&scenario.stage_id) {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} references unknown stage {}",
                    scenario.stage_id
                ));
            }
        }
    }
    for tool in tools.values() {
        for stage in &tool.stage_ids {
            if !stage_to_tools.contains_key(stage) {
                return Err(anyhow!(
                    "tool {} references unknown stage {}",
                    tool.id,
                    stage
                ));
            }
        }
    }
    Ok((
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        stage_statuses,
        stage_output_kinds,
    ))
}

struct ToolRegistryOutputs {
    production_registry: String,
    experimental_registry: String,
    required_tools: String,
}

#[allow(clippy::uninlined_format_args)]
fn build_tool_registries_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_planned: &StagePlannedMap,
    stage_defaults: &StageDefaultMap,
    stage_default_rationale: &StageDefaultRationaleMap,
    source_commit: &str,
) -> ToolRegistryOutputs {
    let mut production_toml = generated_header("domain/**", source_commit);
    let mut experimental_toml = generated_header("domain/**", source_commit);
    let mut required_tools_toml = generated_header("domain/**", source_commit);
    required_tools_toml.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    let mut required_tools = stage_defaults.values().cloned().collect::<Vec<_>>();
    required_tools.sort();
    required_tools.dedup();
    let mut required_tool_set = required_tools.iter().cloned().collect::<BTreeSet<_>>();
    for tool_id in ["seqkit", "vsearch"] {
        required_tool_set.insert(tool_id.to_string());
    }
    let _ = writeln!(
        required_tools_toml,
        "required_tools = {}",
        toml_array(&required_tools)
    );
    required_tools_toml.push('\n');
    let mut production_tool_ids = BTreeSet::new();
    for tool in tools.values() {
        let dockerfile_rel = format!("containers/docker/arm64/Dockerfile.{}", tool.id);
        let apptainer_def_rel = format!("containers/apptainer/bijux/{}.def", tool.id);
        let dockerfile_path = Path::new(&dockerfile_rel);
        let apptainer_def_path = Path::new(&apptainer_def_rel);
        let docker_exists = dockerfile_path.exists();
        let apptainer_exists = apptainer_def_path.exists();
        let mut runtimes = Vec::new();
        if docker_exists {
            runtimes.push("docker".to_string());
        }
        if apptainer_exists {
            runtimes.push("apptainer".to_string());
        }
        if runtimes.is_empty() {
            runtimes = vec!["docker".to_string(), "apptainer".to_string()];
        }
        let is_planned = tool.status == "planned" || tool.default_version == "planned";
        let effective_version = if tool.default_version == "latest-pinned" {
            read_text_if_exists(dockerfile_path)
                .and_then(|recipe| parse_version_from_recipe(&recipe))
                .or_else(|| tool_version_override(&tool.id).map(str::to_string))
                .unwrap_or_else(|| tool.default_version.clone())
        } else {
            tool.default_version.clone()
        };
        let upstream = resolve_tool_upstream(&tool.upstream, &tool.id, dockerfile_path);
        let citation = resolve_tool_citation(&tool.citation, &upstream);
        let upstream_pin = resolve_upstream_pin(
            &tool.container_digest,
            dockerfile_path,
            apptainer_def_path,
            &effective_version,
        );
        let upstream_pin = tool_pin_override(&tool.id).map_or(upstream_pin, str::to_string);
        let container_ref = parse_container_ref(
            &tool.container_image,
            &tool.container_digest,
            &tool.id,
            &effective_version,
        );
        let effective_metrics_schema =
            if tool.metrics_schema == "bijux.unknown.v1" && required_tool_set.contains(&tool.id) {
                "bijux.tool.metrics.v1".to_string()
            } else {
                tool.metrics_schema.clone()
            };
        let is_experimental = effective_metrics_schema == "bijux.unknown.v1"
            || (effective_version == "latest-pinned" && !required_tool_set.contains(&tool.id))
            || (tool.status != "supported" && !required_tool_set.contains(&tool.id))
            || (upstream == "unknown" && !required_tool_set.contains(&tool.id))
            || (upstream_pin == "unresolved" && !required_tool_set.contains(&tool.id));

        let emitted_status = if is_planned {
            "planned"
        } else if is_experimental {
            "experimental"
        } else {
            "production"
        };

        let out = if is_experimental {
            &mut experimental_toml
        } else {
            production_tool_ids.insert(tool.id.clone());
            &mut production_toml
        };

        let _ = writeln!(out, "[[tools]]");
        let _ = writeln!(out, "id = \"{}\"", tool.id);
        let _ = writeln!(out, "tool_id = \"{}\"", tool.id);
        let _ = writeln!(out, "domain = \"{}\"", tool.domain);
        let _ = writeln!(out, "domains = {}", toml_array(&tool.domains));
        let _ = writeln!(out, "status = \"{emitted_status}\"");
        let _ = writeln!(out, "stage_ids = {}", toml_array(&tool.stage_ids));
        let _ = writeln!(out, "bindings = {}", toml_array(&tool.bindings));
        let _ = writeln!(out, "tool_role = \"{}\"", tool.tool_role);
        let _ = writeln!(out, "version = \"{}\"", effective_version);
        let _ = writeln!(out, "default_version = \"{}\"", effective_version);
        let _ = writeln!(out, "upstream = \"{}\"", upstream);
        let _ = writeln!(out, "version_rule = \"{}\"", tool.version_rule);
        let _ = writeln!(out, "license = \"{}\"", tool.license);
        let _ = writeln!(out, "citation = \"{}\"", citation.replace('"', "'"));
        let _ = writeln!(out, "pinned_commit = \"{}\"", upstream_pin);
        let _ = writeln!(out, "pin_strategy = \"{}\"", tool.pin_strategy);
        let _ = writeln!(out, "container_ref = \"{}\"", container_ref);
        let _ = writeln!(out, "runtimes = {}", toml_array(&runtimes));
        let _ = writeln!(
            out,
            "container = {}",
            if is_planned { "false" } else { "true" }
        );
        let _ = writeln!(out, "version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(out, "smoke_version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "smoke_help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(
            out,
            "expected_version_regex = \"{}\"",
            tool.expected_version_regex
        );
        let _ = writeln!(out, "healthcheck_cmd = \"{}\"", tool.healthcheck_cmd);
        let _ = writeln!(out, "expected_bin = \"{}\"", tool.id);
        let _ = writeln!(
            out,
            "expected_artifacts = {}",
            toml_array(&tool.expected_artifacts)
        );
        let _ = writeln!(out, "metrics_schema = \"{}\"", effective_metrics_schema);
        let _ = writeln!(
            out,
            "comparability_notes = \"{}\"",
            tool.comparability_notes.replace('"', "'")
        );
        let _ = writeln!(out, "dockerfile = \"{dockerfile_rel}\"");
        let _ = writeln!(out, "apptainer_def = \"{apptainer_def_rel}\"");
        out.push_str("require_labels = true\n\n");
    }

    for (stage_id, tools_set) in stage_to_tools {
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.retain(|tool_id| production_tool_ids.contains(tool_id));
        all.sort();
        let mut primary = stage_defaults
            .get(stage_id)
            .cloned()
            .filter(|tool_id| production_tool_ids.contains(tool_id))
            .into_iter()
            .collect::<Vec<_>>();
        if primary.is_empty() {
            primary = all.first().cloned().into_iter().collect::<Vec<_>>();
        }
        if primary.is_empty() {
            let stage_domain = stage_id.split('.').next().unwrap_or_default();
            primary.push(if stage_domain == "bam" {
                "samtools".to_string()
            } else {
                "fastp".to_string()
            });
        }
        let optional = all.iter().skip(1).cloned().collect::<Vec<_>>();
        let reporting = if stage_id.contains("qc") {
            vec!["multiqc".to_string()]
        } else {
            Vec::new()
        };
        let _ = writeln!(production_toml, "[[stages]]");
        let _ = writeln!(production_toml, "id = \"{stage_id}\"");
        let _ = writeln!(
            production_toml,
            "required_tool_roles = {}",
            toml_array(&required_tool_roles_for_stage(stage_id))
        );
        let _ = writeln!(production_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(
            production_toml,
            "optional_alternatives = {}",
            toml_array(&optional)
        );
        production_toml.push_str("validation_tools = []\n");
        let _ = writeln!(
            production_toml,
            "reporting_tools = {}",
            toml_array(&reporting)
        );
        let _ = writeln!(
            production_toml,
            "planned_out_of_scope = {}",
            toml_array(stage_planned.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let rationale = stage_default_rationale
            .get(stage_id)
            .map_or("", std::string::String::as_str)
            .replace('"', "'");
        let _ = writeln!(production_toml, "default_rationale = \"{rationale}\"");
        production_toml.push_str("requires_validation = false\n");
        let _ = writeln!(
            production_toml,
            "requires_reporting = {}",
            if reporting.is_empty() {
                "false"
            } else {
                "true"
            }
        );
        production_toml.push('\n');
    }
    ToolRegistryOutputs {
        production_registry: production_toml,
        experimental_registry: experimental_toml,
        required_tools: required_tools_toml,
    }
}

fn collect_vcf_image_versions(domain_dir: &Path) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    let vcf_tools_dir = domain_dir.join("vcf").join("tools");
    if !vcf_tools_dir.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&vcf_tools_dir)
        .with_context(|| format!("read {}", vcf_tools_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
            continue;
        }
        let tool: DomainToolLoose = read_yaml(&path)?;
        if tool.tool_id.trim().is_empty() || tool.status == "out_of_scope" {
            continue;
        }
        out.insert(tool.tool_id, tool.default_version);
    }
    Ok(out)
}

fn build_images_toml(
    tools: &ToolMap,
    vcf_image_versions: &BTreeMap<String, String>,
    source_commit: &str,
) -> String {
    let mut images_toml = generated_header("domain/**", source_commit);
    let mut image_versions = BTreeMap::<String, String>::new();
    for tool in tools.values() {
        image_versions.insert(tool.id.clone(), tool.default_version.clone());
    }
    for (tool_id, version) in vcf_image_versions {
        image_versions
            .entry(tool_id.clone())
            .or_insert_with(|| version.clone());
    }
    for planned_only in ["ibdseq", "shapeit"] {
        image_versions
            .entry(planned_only.to_string())
            .or_insert_with(|| "planned".to_string());
    }
    for (tool_id, version) in image_versions {
        let _ = writeln!(images_toml, "[{tool_id}]");
        let _ = writeln!(images_toml, "version = \"{version}\"");
        if version == "planned" || tool_id == "angsd" {
            let _ = writeln!(images_toml, "enabled = false");
        }
        images_toml.push('\n');
    }
    images_toml
}

fn build_stages_toml(
    stage_to_tools: &StageToolMap,
    stage_statuses: &StageStatusMap,
    stage_output_kinds: &StageOutputKindsMap,
    domain_dir: &Path,
    source_commit: &str,
) -> String {
    let mut ordering_map = BTreeMap::<String, Vec<String>>::new();
    let mut prereq_map = BTreeMap::<String, Vec<String>>::new();
    let mut resource_map = BTreeMap::<String, StageResourceHint>::new();
    let mut output_size_map = BTreeMap::<String, BTreeMap<String, f64>>::new();
    let mut sanity_map = BTreeMap::<String, Vec<String>>::new();
    let mut qc_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut contamination_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut authenticity_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut duplication_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut coverage_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut sex_kinship_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut pipelines = Vec::<(String, Vec<String>)>::new();
    let mut benchmark_scenarios = Vec::<(String, BenchmarkScenario)>::new();
    for dom in ["fastq", "bam"] {
        let index_path = domain_dir.join(dom).join("index.yaml");
        if !index_path.exists() {
            continue;
        }
        if let Ok(index) = read_yaml::<DomainIndex>(&index_path) {
            for (k, v) in index.stage_ordering_constraints {
                ordering_map.insert(k, v);
            }
            for (k, v) in index.stage_prerequisites {
                prereq_map.insert(k, v);
            }
            for (k, v) in index.stage_resource_hints {
                resource_map.insert(k, v);
            }
            for (k, v) in index.stage_output_size_estimates_mb {
                output_size_map.insert(k, v);
            }
            for (k, v) in index.stage_sanity_metrics {
                sanity_map.insert(k, v);
            }
            for (k, v) in index.stage_qc_thresholds {
                qc_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_contamination_thresholds {
                contamination_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_authenticity_thresholds {
                authenticity_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_duplication_thresholds {
                duplication_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_coverage_sufficiency {
                coverage_sufficiency_map.insert(k, v);
            }
            for (k, v) in index.stage_sex_kinship_sufficiency {
                sex_kinship_sufficiency_map.insert(k, v);
            }
            for (pipeline, stages) in index.pipeline_compositions {
                pipelines.push((format!("{dom}.{pipeline}"), stages));
            }
            for (scenario_id, scenario) in index.benchmark_scenarios {
                benchmark_scenarios.push((format!("{dom}.{scenario_id}"), scenario));
            }
        }
    }
    let mut stages_toml = generated_header("domain/**", source_commit);
    for (stage_id, tools_set) in stage_to_tools {
        let status = stage_statuses
            .get(stage_id.as_str())
            .map_or("planned", std::string::String::as_str);
        if status != "supported" {
            continue;
        }
        let _ = writeln!(stages_toml, "[[stages]]");
        let _ = writeln!(stages_toml, "id = \"{stage_id}\"");
        let _ = writeln!(stages_toml, "status = \"{status}\"");
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.sort();
        let output_kinds = stage_output_kinds
            .get(stage_id)
            .cloned()
            .unwrap_or_default();
        let _ = writeln!(stages_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(
            stages_toml,
            "ordering_after = {}",
            toml_array(ordering_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let _ = writeln!(
            stages_toml,
            "prerequisites = {}",
            toml_array(prereq_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        if let Some(resources) = resource_map.get(stage_id) {
            let _ = writeln!(stages_toml, "resource_memory_gb = {}", resources.memory_gb);
            let _ = writeln!(
                stages_toml,
                "resource_time_minutes = {}",
                resources.time_minutes
            );
            let _ = writeln!(stages_toml, "resource_threads = {}", resources.threads);
        }
        if let Some(sanity) = sanity_map.get(stage_id) {
            let _ = writeln!(stages_toml, "sanity_metrics = {}", toml_array(sanity));
        }
        if let Some(size_estimates) = output_size_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "output_size_estimates_mb = {}",
                encode_f64_map(size_estimates)
            );
        }
        if let Some(qc) = qc_thresholds_map.get(stage_id) {
            let _ = writeln!(stages_toml, "qc_thresholds = {}", encode_threshold_map(qc));
        }
        if let Some(contam) = contamination_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "contamination_thresholds = {}",
                encode_threshold_map(contam)
            );
        }
        if let Some(auth) = authenticity_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "authenticity_thresholds = {}",
                encode_threshold_map(auth)
            );
        }
        if let Some(dup) = duplication_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "duplication_thresholds = {}",
                encode_threshold_map(dup)
            );
        }
        if let Some(coverage_logic) = coverage_sufficiency_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "coverage_sufficiency = {}",
                toml_array(coverage_logic)
            );
        }
        if let Some(sex_kinship_logic) = sex_kinship_sufficiency_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "sex_kinship_sufficiency = {}",
                toml_array(sex_kinship_logic)
            );
        }
        let _ = writeln!(stages_toml, "tools = {}\n", toml_array(&v));
    }
    pipelines.sort_by(|a, b| a.0.cmp(&b.0));
    for (pipeline_id, stages) in pipelines {
        let _ = writeln!(stages_toml, "[[pipelines]]");
        let _ = writeln!(stages_toml, "id = \"{pipeline_id}\"");
        let _ = writeln!(stages_toml, "stages = {}", toml_array(&stages));
        stages_toml.push('\n');
    }
    benchmark_scenarios.sort_by(|a, b| a.0.cmp(&b.0));
    for (scenario_id, scenario) in benchmark_scenarios {
        let _ = writeln!(stages_toml, "[[benchmark_scenarios]]");
        let _ = writeln!(stages_toml, "id = \"{scenario_id}\"");
        let _ = writeln!(stages_toml, "stage_id = \"{}\"", scenario.stage_id);
        let _ = writeln!(
            stages_toml,
            "description = \"{}\"",
            scenario.description.replace('"', "'")
        );
        let _ = writeln!(
            stages_toml,
            "fairness_rules = {}",
            toml_array(&scenario.fairness_rules)
        );
        stages_toml.push('\n');
    }
    stages_toml
}
