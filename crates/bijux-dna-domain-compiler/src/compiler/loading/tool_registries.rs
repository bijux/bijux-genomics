use super::super::*;

pub(super) struct ToolRegistryOutputs {
    pub(super) production_registry: String,
    pub(super) experimental_registry: String,
    pub(super) required_tools: String,
    pub(super) production_tool_ids: BTreeSet<String>,
}

fn tool_supports_stage_domain(tools: &ToolMap, tool_id: &str, stage_domain: &str) -> bool {
    tools.get(tool_id).is_some_and(|tool| {
        tool.domain == stage_domain || tool.domains.iter().any(|domain| domain == stage_domain)
    })
}

#[allow(clippy::uninlined_format_args)]
pub(super) fn build_tool_registries_toml(
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
    let _ = writeln!(required_tools_toml, "required_tools = {}", toml_array(&required_tools));
    required_tools_toml.push('\n');
    let mut production_tool_ids = BTreeSet::new();
    for tool in tools.values() {
        let dockerfile_rel = format!("containers/docker/arm64/Dockerfile.{}", tool.id);
        let apptainer_def_rel = format!("containers/apptainer/shared/{}.def", tool.id);
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

        let out = if is_planned || is_experimental {
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
        let _ = writeln!(out, "container = {}", if is_planned { "false" } else { "true" });
        let _ = writeln!(out, "version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(out, "smoke_version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "smoke_help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(out, "expected_version_regex = \"{}\"", tool.expected_version_regex);
        let _ = writeln!(out, "healthcheck_cmd = \"{}\"", tool.healthcheck_cmd);
        let expected_bin = tool.version_cmd.split_whitespace().next().unwrap_or(tool.id.as_str());
        let _ = writeln!(out, "expected_bin = \"{}\"", expected_bin);
        let _ = writeln!(out, "expected_artifacts = {}", toml_array(&tool.expected_artifacts));
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
        let stage_domain = stage_id.split('.').next().unwrap_or_default();
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.retain(|tool_id| {
            production_tool_ids.contains(tool_id)
                && tool_supports_stage_domain(tools, tool_id, stage_domain)
        });
        all.sort();
        let mut primary = stage_defaults
            .get(stage_id)
            .cloned()
            .filter(|tool_id| {
                production_tool_ids.contains(tool_id)
                    && tool_supports_stage_domain(tools, tool_id, stage_domain)
            })
            .into_iter()
            .collect::<Vec<_>>();
        if primary.is_empty() {
            primary = all.first().cloned().into_iter().collect::<Vec<_>>();
        }
        if primary.is_empty() {
            primary.push(if stage_domain == "bam" {
                "samtools".to_string()
            } else {
                "fastp".to_string()
            });
        }
        let optional = all
            .iter()
            .filter(|tool_id| !primary.iter().any(|selected| selected == *tool_id))
            .cloned()
            .collect::<Vec<_>>();
        let reporting =
            if stage_id.contains("qc") { vec!["multiqc".to_string()] } else { Vec::new() };
        let _ = writeln!(production_toml, "[[stages]]");
        let _ = writeln!(production_toml, "id = \"{stage_id}\"");
        let _ = writeln!(
            production_toml,
            "required_tool_roles = {}",
            toml_array(&required_tool_roles_for_stage(stage_id))
        );
        let _ = writeln!(production_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(production_toml, "optional_alternatives = {}", toml_array(&optional));
        production_toml.push_str("validation_tools = []\n");
        let _ = writeln!(production_toml, "reporting_tools = {}", toml_array(&reporting));
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
            if reporting.is_empty() { "false" } else { "true" }
        );
        production_toml.push('\n');
    }
    ToolRegistryOutputs {
        production_registry: production_toml,
        experimental_registry: experimental_toml,
        required_tools: required_tools_toml,
        production_tool_ids,
    }
}
