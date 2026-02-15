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
        let expected_bin = tool
            .version_cmd
            .split_whitespace()
            .next()
            .unwrap_or(tool.id.as_str());
        let _ = writeln!(out, "expected_bin = \"{}\"", expected_bin);
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
