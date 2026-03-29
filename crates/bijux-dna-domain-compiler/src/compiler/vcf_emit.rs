use super::*;

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

    let mut stages_vcf_toml = vcf_header.clone();
    let vcf_stage_dir = domain_dir.join("vcf").join("stages");
    if vcf_stage_dir.exists() {
        for entry in std::fs::read_dir(&vcf_stage_dir)
            .with_context(|| format!("read {}", vcf_stage_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage: DomainStage = read_yaml(&path)?;
            if stage.scope != "post_vcf" || stage.status == "out_of_scope" {
                continue;
            }
            let tools = if stage.compatible_tools.is_empty() {
                vec!["bcftools".to_string()]
            } else {
                stage.compatible_tools.clone()
            };
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
            let _ = writeln!(stages_vcf_toml, "experimental = true");
            let _ = writeln!(stages_vcf_toml, "metrics_schema = \"{metrics_schema}\"");
            let _ = writeln!(stages_vcf_toml, "smoke_required = true");
            let _ = writeln!(stages_vcf_toml, "tools = {}", toml_array(&tools));
            stages_vcf_toml.push('\n');
        }
    }

    let mut tools_vcf_toml = vcf_header;
    let vcf_tool_dir = domain_dir.join("vcf").join("tools");
    if vcf_tool_dir.exists() {
        for entry in std::fs::read_dir(&vcf_tool_dir)
            .with_context(|| format!("read {}", vcf_tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if tool.scope != "post_vcf" || tool.status == "out_of_scope" {
                continue;
            }
            let _ = writeln!(tools_vcf_toml, "[[tools]]");
            let _ = writeln!(tools_vcf_toml, "id = \"{}\"", tool.tool_id);
            let _ = writeln!(tools_vcf_toml, "tool_id = \"{}\"", tool.tool_id);
            let _ = writeln!(tools_vcf_toml, "domain = \"vcf\"");
            let vcf_status = if tool.status == "supported" {
                "production"
            } else {
                "planned"
            };
            let _ = writeln!(tools_vcf_toml, "status = \"{vcf_status}\"");
            let _ = writeln!(
                tools_vcf_toml,
                "stage_ids = {}",
                toml_array(&tool.stage_ids)
            );
            let _ = writeln!(tools_vcf_toml, "version = \"{}\"", tool.default_version);
            let _ = writeln!(
                tools_vcf_toml,
                "default_version = \"{}\"",
                tool.default_version
            );
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
            let _ = writeln!(
                tools_vcf_toml,
                "smoke_version_cmd = \"{}\"",
                tool.version_cmd
            );
            let _ = writeln!(tools_vcf_toml, "smoke_help_cmd = \"{}\"", tool.help_cmd);
            let _ = writeln!(
                tools_vcf_toml,
                "expected_version_regex = \"bcftools [0-9]+[.][0-9]+\""
            );
            let _ = writeln!(tools_vcf_toml, "healthcheck_cmd = \"{}\"", tool.help_cmd);
            let expected_bin = tool
                .version_cmd
                .split_whitespace()
                .next()
                .unwrap_or(tool.tool_id.as_str());
            let _ = writeln!(tools_vcf_toml, "expected_bin = \"{}\"", expected_bin);
            let _ = writeln!(
                tools_vcf_toml,
                "expected_artifacts = {}",
                toml_array(&tool.expected_artifacts)
            );
            let _ = writeln!(
                tools_vcf_toml,
                "metrics_schema = \"{}\"",
                tool.metrics_schema_id
            );
            let _ = writeln!(
                tools_vcf_toml,
                "comparability_notes = \"{}\"",
                tool.comparability_notes.replace('"', "'")
            );
            let _ = writeln!(
                tools_vcf_toml,
                "dockerfile = \"containers/docker/arm64/Dockerfile.bcftools\""
            );
            let _ = writeln!(
                tools_vcf_toml,
                "apptainer_def = \"containers/apptainer/shared/bcftools.def\""
            );
            let _ = writeln!(tools_vcf_toml, "require_labels = true\n");
        }
    }

    write_string(&tool_registry_vcf_path, &tools_vcf_toml)
        .with_context(|| format!("write {}", tool_registry_vcf_path.display()))?;
    write_string(&stages_vcf_path, &stages_vcf_toml)
        .with_context(|| format!("write {}", stages_vcf_path.display()))?;
    println!("generated: {}", tool_registry_vcf_path.display());
    println!("generated: {}", stages_vcf_path.display());
    Ok(())
}
