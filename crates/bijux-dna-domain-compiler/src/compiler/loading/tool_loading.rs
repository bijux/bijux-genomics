use super::super::{
    anyhow, bail, default_healthcheck_cmd, default_version_regex, ensure_status,
    has_supported_placeholder_forbidden_token, infer_tool_role, placeholders_allowed, read_yaml,
    scope_active, Context, DomainIndex, DomainTool, Path, Result, StageToolMap, ToolMap, ToolRow,
};

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
                stage_to_tools.entry(stage.clone()).or_default().insert(tool.tool_id.clone());
            }
        }
        let tool_id = tool.tool_id.clone();
        let candidate = build_tool_row(tool, &resolved_domain);
        if let Some(existing) = tools.get_mut(&tool_id) {
            merge_tool_rows(existing, candidate)?;
            continue;
        }
        tools.insert(tool_id.clone(), candidate);
    }
    Ok(())
}

fn build_tool_row(tool: DomainTool, resolved_domain: &str) -> ToolRow {
    let tool_id = tool.tool_id.clone();
    let version_rule = tool.versioning_strategy.clone();
    let help_cmd_value = tool.help_cmd.clone();
    let mut domains = tool
        .declared_stage_ids()
        .filter_map(|stage_id| stage_id.split('.').next().map(str::to_string))
        .collect::<Vec<_>>();
    domains.sort();
    domains.dedup();
    let mut stage_ids = tool.stage_ids;
    stage_ids.sort();
    stage_ids.dedup();
    let bindings = stage_ids.clone();
    let tool_role = infer_tool_role(&bindings);
    ToolRow {
        id: tool_id.clone(),
        domain: resolved_domain.to_string(),
        domains,
        stage_ids,
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
    }
}

fn merge_tool_rows(existing: &mut ToolRow, candidate: ToolRow) -> Result<()> {
    ensure_same_field(&existing.id, &candidate.id, "id")?;
    ensure_same_field(&existing.default_version, &candidate.default_version, "default_version")?;
    ensure_same_field(&existing.upstream, &candidate.upstream, "upstream")?;
    ensure_same_field(&existing.pin_strategy, &candidate.pin_strategy, "pin_strategy")?;
    ensure_same_field(&existing.version_cmd, &candidate.version_cmd, "version_cmd")?;
    ensure_same_field(&existing.help_cmd, &candidate.help_cmd, "help_cmd")?;
    ensure_same_field(&existing.status, &candidate.status, "status")?;
    ensure_same_field(&existing.version_rule, &candidate.version_rule, "version_rule")?;
    ensure_same_field(&existing.license, &candidate.license, "license")?;
    ensure_same_field(&existing.citation, &candidate.citation, "citation")?;
    ensure_same_field(
        &existing.expected_version_regex,
        &candidate.expected_version_regex,
        "expected_version_regex",
    )?;
    ensure_same_field(&existing.healthcheck_cmd, &candidate.healthcheck_cmd, "healthcheck_cmd")?;

    existing.domains.extend(candidate.domains);
    existing.domains.sort();
    existing.domains.dedup();

    existing.stage_ids.extend(candidate.stage_ids);
    existing.stage_ids.sort();
    existing.stage_ids.dedup();

    existing.bindings.extend(candidate.bindings);
    existing.bindings.sort();
    existing.bindings.dedup();

    existing.expected_artifacts.extend(candidate.expected_artifacts);
    existing.expected_artifacts.sort();
    existing.expected_artifacts.dedup();

    if (existing.metrics_schema.is_empty() || existing.metrics_schema == "bijux.unknown.v1")
        && !candidate.metrics_schema.is_empty()
    {
        existing.metrics_schema = candidate.metrics_schema;
    }
    merge_optional_field(&mut existing.container_image, &candidate.container_image);
    merge_optional_field(&mut existing.container_digest, &candidate.container_digest);
    merge_comparability_notes(existing, &candidate.comparability_notes);

    existing.tool_role = infer_tool_role(&existing.bindings);
    Ok(())
}

fn ensure_same_field(existing: &str, candidate: &str, field: &str) -> Result<()> {
    if existing == candidate {
        return Ok(());
    }
    Err(anyhow!("shared tool metadata conflict for {field}: `{existing}` vs `{candidate}`"))
}

fn merge_comparability_notes(existing: &mut ToolRow, candidate_notes: &str) {
    if candidate_notes.is_empty() || existing.comparability_notes == candidate_notes {
        return;
    }
    if existing.comparability_notes.is_empty() {
        existing.comparability_notes = candidate_notes.to_string();
        return;
    }
    existing.comparability_notes =
        "Comparable only within the same governed stage contract and emitted artifact set."
            .to_string();
}

fn merge_optional_field(existing: &mut String, candidate: &str) {
    if existing.is_empty() && !candidate.is_empty() {
        *existing = candidate.to_string();
    }
}
