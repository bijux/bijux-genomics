use super::super::*;

pub(super) fn load_domain_tools(
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
        let resolved_domain = path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or(domain)
            .to_string();
        for stage in &tool.stage_ids {
            if stage.split('.').next() == Some(resolved_domain.as_str()) {
                stage_to_tools
                    .entry(stage.clone())
                    .or_default()
                    .insert(tool.tool_id.clone());
            }
        }
        let tool_id = tool.tool_id.clone();
        if tools.contains_key(&tool_id) {
            continue;
        }
        let version_rule = tool.versioning_strategy.clone();
        let help_cmd_value = tool.help_cmd.clone();
        let mut domains = tool
            .declared_stage_ids()
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
