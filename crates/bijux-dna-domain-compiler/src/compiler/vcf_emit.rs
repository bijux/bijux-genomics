use super::*;

fn vcf_output_kinds(stage: &DomainStage) -> Vec<String> {
    let mut kinds = stage.outputs.iter().map(|port| port.data_type.clone()).collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    kinds
}

fn vcf_stage_default_tools(
    index: &DomainIndex,
    stage: &DomainStage,
    tools: &[String],
) -> Vec<String> {
    index
        .active_defaults
        .get(&stage.stage_id)
        .cloned()
        .filter(|tool_id| tools.iter().any(|candidate| candidate == tool_id))
        .or_else(|| tools.first().cloned())
        .into_iter()
        .collect()
}

fn vcf_stage_default_rationale(index: &DomainIndex, stage: &DomainStage) -> String {
    index.active_default_rationale.get(&stage.stage_id).cloned().unwrap_or_default()
}

fn vcf_apptainer_def(tool: &DomainToolLoose) -> String {
    let shared = format!("containers/apptainer/shared/{}.def", tool.tool_id);
    if Path::new(&shared).exists() {
        return shared;
    }
    if let Some(container) = tool.container.as_ref() {
        let image = container.image.trim();
        if Path::new(image)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("def"))
            && Path::new(image).exists()
        {
            return image.to_string();
        }
    }
    shared
}

fn vcf_dockerfile(tool: &DomainToolLoose) -> String {
    format!("containers/docker/arm64/Dockerfile.{}", tool.tool_id)
}

fn vcf_expected_version_regex(expected_bin: &str) -> String {
    format!("{expected_bin} [0-9]+([.][0-9]+)?")
}

/// # Errors
/// Returns an error when VCF post-processing domain views cannot be read or written.
pub(super) fn write_vcf_generated_views(
    domain_dir: &Path,
    ci_registry_dir: &Path,
    ci_stages_dir: &Path,
    source_commit: &str,
) -> Result<()> {
    let tool_registry_vcf_path = ci_registry_dir.join("tool_registry_vcf.toml");
    let stages_vcf_path = ci_stages_dir.join("stages_vcf.toml");
    let vcf_header = generated_header("domain/vcf/**", source_commit);
    let vcf_index_path = domain_dir.join("vcf").join("index.yaml");
    let vcf_index: DomainIndex = read_yaml(&vcf_index_path)?;

    let mut stages_vcf_toml = vcf_header.clone();
    let mut registry_stage_toml = String::new();
    let vcf_stage_dir = domain_dir.join("vcf").join("stages");
    for path in collect_yaml_files(&vcf_stage_dir)? {
        let stage: DomainStage = read_yaml(&path)?;
        if stage.scope != "post_vcf" || stage.status == "out_of_scope" {
            continue;
        }
        let tools = if stage.compatible_tools.is_empty() {
            vec!["bcftools".to_string()]
        } else {
            stage.compatible_tools.clone()
        };
        let output_kinds = vcf_output_kinds(&stage);
        let metrics_schema = if stage.stage_id == "vcf.stats" {
            "bijux.vcf.stats.v1"
        } else if stage.stage_id == "vcf.filter" {
            "bijux.vcf.filter.v1"
        } else {
            "bijux.vcf.call.v1"
        };
        let _ = writeln!(stages_vcf_toml, "[[stages]]");
        let _ = writeln!(stages_vcf_toml, "id = \"{}\"", stage.stage_id);
        let _ = writeln!(stages_vcf_toml, "status = \"{}\"", stage.status);
        let _ = writeln!(stages_vcf_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(stages_vcf_toml, "experimental = true");
        let _ = writeln!(stages_vcf_toml, "metrics_schema = \"{metrics_schema}\"");
        let _ = writeln!(stages_vcf_toml, "smoke_required = true");
        let _ = writeln!(stages_vcf_toml, "tools = {}", toml_array(&tools));
        stages_vcf_toml.push('\n');

        if stage.status != "supported" {
            continue;
        }

        let primary = vcf_stage_default_tools(&vcf_index, &stage, &tools);
        let optional = tools
            .iter()
            .filter(|tool_id| !primary.iter().any(|selected| selected == *tool_id))
            .cloned()
            .collect::<Vec<_>>();
        let rationale = vcf_stage_default_rationale(&vcf_index, &stage).replace('"', "'");
        let _ = writeln!(registry_stage_toml, "[[stages]]");
        let _ = writeln!(registry_stage_toml, "id = \"{}\"", stage.stage_id);
        let _ = writeln!(
            registry_stage_toml,
            "required_tool_roles = {}",
            toml_array(&required_tool_roles_for_stage(&stage.stage_id))
        );
        let _ = writeln!(registry_stage_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(registry_stage_toml, "optional_alternatives = {}", toml_array(&optional));
        registry_stage_toml.push_str("validation_tools = []\n");
        registry_stage_toml.push_str("reporting_tools = []\n");
        let _ = writeln!(
            registry_stage_toml,
            "planned_out_of_scope = {}",
            toml_array(&stage.planned_out_of_scope)
        );
        let _ = writeln!(registry_stage_toml, "default_rationale = \"{rationale}\"");
        registry_stage_toml.push_str("requires_validation = false\n");
        registry_stage_toml.push_str("requires_reporting = false\n\n");
    }

    let mut tools_vcf_toml = vcf_header;
    let vcf_tool_dir = domain_dir.join("vcf").join("tools");
    for path in collect_yaml_files(&vcf_tool_dir)? {
        let tool: DomainToolLoose = read_yaml(&path)?;
        if tool.scope != "post_vcf" || tool.status == "out_of_scope" {
            continue;
        }
        let stage_ids = tool.declared_stage_ids().cloned().collect::<Vec<_>>();
        let _ = writeln!(tools_vcf_toml, "[[tools]]");
        let _ = writeln!(tools_vcf_toml, "id = \"{}\"", tool.tool_id);
        let _ = writeln!(tools_vcf_toml, "tool_id = \"{}\"", tool.tool_id);
        let _ = writeln!(tools_vcf_toml, "domain = \"vcf\"");
        let vcf_status = if tool.status == "supported" { "production" } else { "planned" };
        let _ = writeln!(tools_vcf_toml, "status = \"{vcf_status}\"");
        let _ = writeln!(tools_vcf_toml, "stage_ids = {}", toml_array(&stage_ids));
        let _ = writeln!(tools_vcf_toml, "version = \"{}\"", tool.default_version);
        let _ = writeln!(tools_vcf_toml, "default_version = \"{}\"", tool.default_version);
        let _ = writeln!(tools_vcf_toml, "upstream = \"{}\"", tool.upstream);
        let _ = writeln!(tools_vcf_toml, "version_rule = \"pinned\"");
        let _ = writeln!(tools_vcf_toml, "license = \"{}\"", tool.license);
        let _ = writeln!(tools_vcf_toml, "citation = \"{}\"", tool.citation);
        let digest = if let Some(container) = tool.container.as_ref() {
            container.digest.clone()
        } else {
            String::new()
        };
        let image = if let Some(container) = tool.container.as_ref() {
            container.image.clone()
        } else {
            String::new()
        };
        let _ = writeln!(tools_vcf_toml, "pinned_commit = \"{digest}\"");
        let _ = writeln!(tools_vcf_toml, "pin_strategy = \"{}\"", tool.pin_strategy);
        let _ = writeln!(tools_vcf_toml, "container_ref = \"{image}@{digest}\"");
        let _ = writeln!(tools_vcf_toml, "runtimes = [\"docker\", \"apptainer\"]");
        let _ = writeln!(tools_vcf_toml, "container = true");
        let _ = writeln!(tools_vcf_toml, "version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(tools_vcf_toml, "help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(tools_vcf_toml, "smoke_version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(tools_vcf_toml, "smoke_help_cmd = \"{}\"", tool.help_cmd);
        let expected_bin =
            tool.version_cmd.split_whitespace().next().unwrap_or(tool.tool_id.as_str());
        let _ = writeln!(
            tools_vcf_toml,
            "expected_version_regex = \"{}\"",
            vcf_expected_version_regex(expected_bin)
        );
        let _ = writeln!(tools_vcf_toml, "healthcheck_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(tools_vcf_toml, "expected_bin = \"{}\"", expected_bin);
        let _ = writeln!(
            tools_vcf_toml,
            "expected_artifacts = {}",
            toml_array(&tool.expected_artifacts)
        );
        let _ = writeln!(tools_vcf_toml, "metrics_schema = \"{}\"", tool.metrics_schema_id);
        let _ = writeln!(
            tools_vcf_toml,
            "comparability_notes = \"{}\"",
            tool.comparability_notes.replace('"', "'")
        );
        let _ = writeln!(tools_vcf_toml, "dockerfile = \"{}\"", vcf_dockerfile(&tool));
        let _ = writeln!(tools_vcf_toml, "apptainer_def = \"{}\"", vcf_apptainer_def(&tool));
        let _ = writeln!(tools_vcf_toml, "require_labels = true\n");
    }
    tools_vcf_toml.push_str(&registry_stage_toml);

    write_string(&tool_registry_vcf_path, &tools_vcf_toml)
        .with_context(|| format!("write {}", tool_registry_vcf_path.display()))?;
    write_string(&stages_vcf_path, &stages_vcf_toml)
        .with_context(|| format!("write {}", stages_vcf_path.display()))?;
    println!("generated: {}", tool_registry_vcf_path.display());
    println!("generated: {}", stages_vcf_path.display());
    Ok(())
}
