fn validate_domain_indexes_and_pipelines(
    options: &ValidateOptions,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
    tool_capabilities: &BTreeMap<String, BTreeSet<String>>,
    tool_statuses: &BTreeMap<String, String>,
    tool_metrics_schemas: &BTreeMap<String, String>,
) -> Result<()> {
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
        for (tool_id, status) in tool_statuses {
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
    Ok(())
}
