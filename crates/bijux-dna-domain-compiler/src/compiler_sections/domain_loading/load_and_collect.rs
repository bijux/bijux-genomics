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

