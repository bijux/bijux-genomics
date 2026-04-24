use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use toml::Value as TomlValue;
use walkdir::WalkDir;

use super::domain_workflow::{
    allowed_payload_keys, domain_directories, failure_block, inline_list, read_utf8, regex,
    required_fields, scalar_from_text, success_line, top_level_keys, yaml_files,
};
use super::{load_toml, toml_stages, toml_tools, tool_registry_files};
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

fn production_bindings(workspace: &Workspace) -> Result<BTreeSet<(String, String)>> {
    let mut bindings = BTreeSet::new();
    for row in toml_tools(&workspace.path("configs/ci/registry/tool_registry.toml"))? {
        let Some(table) = row.as_table() else {
            continue;
        };
        let tool_id = table
            .get("id")
            .or_else(|| table.get("tool_id"))
            .and_then(TomlValue::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let status =
            table.get("status").and_then(TomlValue::as_str).unwrap_or_default().trim().to_string();
        if tool_id.is_empty() || !matches!(status.as_str(), "production" | "supported") {
            continue;
        }
        for binding in table.get("bindings").and_then(TomlValue::as_array).into_iter().flatten() {
            if let Some(stage_id) = binding.as_str() {
                bindings.insert((stage_id.trim().to_string(), tool_id.clone()));
            }
        }
    }
    Ok(bindings)
}

pub(super) fn check_domain_schema(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let production_bindings = production_bindings(workspace)?;
    let downstream = workspace.path("configs/ci/stages/stages_vcf_downstream.toml");
    if downstream.is_file() {
        let rows = toml_stages(&downstream)?;
        if rows.is_empty() {
            errors.push(format!(
                "{}: must define at least one [[stages]] entry",
                downstream.display()
            ));
        }
        for row in rows {
            let Some(table) = row.as_table() else {
                continue;
            };
            let stage_id =
                table.get("id").and_then(TomlValue::as_str).unwrap_or_default().trim().to_string();
            let status = table
                .get("status")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !stage_id.starts_with("vcf.") {
                errors.push(format!(
                    "{}: downstream stage id must start with 'vcf.': {stage_id}",
                    downstream.display()
                ));
            }
            if !matches!(status.as_str(), "planned" | "experimental" | "production" | "supported") {
                errors.push(format!(
                    "{}: invalid stage status '{status}' for {stage_id}",
                    downstream.display()
                ));
            }
        }
    } else {
        errors.push(format!(
            "{}: missing required downstream stages registry file",
            downstream.display()
        ));
    }

    let repeated_token_re = regex(r"__")?;
    let snake_case_re = regex(r"^[a-z0-9_]+$")?;
    let stage_slug_re = regex(r"^[a-z0-9_]+$")?;
    let metric_item_re = regex(r"(?m)^\s*-\s*[a-z0-9_]+\s*$")?;
    let metric_entry_re = regex(r#"(?m)^\s*-\s*id:\s*"?[a-z0-9_]+"?\s*$"#)?;

    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let stage_schema = dom_dir.join("stages/_schema.yaml");
        let tool_schema = dom_dir.join("tools/_schema.yaml");
        if !(stage_schema.is_file() && tool_schema.is_file()) {
            continue;
        }
        let required_stage = required_fields(&stage_schema)?;
        let required_tool = required_fields(&tool_schema)?;
        let required_scope = scalar_from_text(&read_utf8(&stage_schema)?, "required_scope")?;
        let required_domain = scalar_from_text(&read_utf8(&stage_schema)?, "domain")?;
        let required_tool_scope = scalar_from_text(&read_utf8(&tool_schema)?, "required_scope")?;

        let mut stage_ids_seen = BTreeSet::new();
        let mut tool_ids_seen = BTreeSet::new();

        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let keys = top_level_keys(&text)?;
            let missing = required_stage
                .iter()
                .filter(|field| !keys.contains(field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!(
                    "{}: missing required fields: {:?}",
                    stage_file.display(),
                    missing
                ));
            }

            let stage_id = scalar_from_text(&text, "stage_id")?;
            if let Some(stage_id_value) = stage_id.clone() {
                if !stage_ids_seen.insert(stage_id_value.clone()) {
                    errors.push(format!(
                        "{}: duplicate stage_id in domain {dom}: {stage_id_value}",
                        stage_file.display()
                    ));
                }
                let prefix = format!("{dom}.");
                if stage_id_value.starts_with(&prefix) {
                    let slug = stage_id_value.trim_start_matches(&prefix);
                    if !stage_slug_re.is_match(slug) {
                        errors.push(format!(
                            "{}: stage slug '{slug}' must match [a-z0-9_]+",
                            stage_file.display()
                        ));
                    }
                    if repeated_token_re.is_match(slug) {
                        errors.push(format!(
                            "{}: stage slug '{slug}' must not contain '__'",
                            stage_file.display()
                        ));
                    }
                    let parts = slug.split('_').filter(|part| !part.is_empty()).collect::<Vec<_>>();
                    for pair in parts.windows(2) {
                        if pair[0] == pair[1] {
                            errors.push(format!(
                                "{}: stage slug '{slug}' has repeated adjacent token '{}'",
                                stage_file.display(),
                                pair[0]
                            ));
                        }
                    }
                } else {
                    errors.push(format!(
                        "{}: stage_id must use '<domain>.<stage_slug>' format",
                        stage_file.display()
                    ));
                }
            } else {
                errors.push(format!("{}: missing stage_id", stage_file.display()));
            }

            let scope = scalar_from_text(&text, "scope")?;
            if required_scope.as_deref().is_some_and(|required| scope.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: scope must be {} (got {})",
                    stage_file.display(),
                    required_scope.clone().unwrap_or_default(),
                    scope.unwrap_or_default()
                ));
            }
            let declared_domain = scalar_from_text(&text, "domain")?;
            if required_domain
                .as_deref()
                .is_some_and(|required| declared_domain.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: domain must be {} (got {})",
                    stage_file.display(),
                    required_domain.clone().unwrap_or_default(),
                    declared_domain.unwrap_or_default()
                ));
            }
            let defaults_source = scalar_from_text(&text, "defaults_source")?;
            match defaults_source {
                Some(value) if value.starts_with("citation:") || value.starts_with("doc_ref:") => {}
                Some(value) => errors.push(format!(
                    "{}: defaults_source must start with citation: or doc_ref: (got {value})",
                    stage_file.display()
                )),
                None => errors.push(format!("{}: missing defaults_source", stage_file.display())),
            }

            if dom == "vcf" {
                let compatible = inline_list(&text, "compatible_tools")?;
                let single_justification = scalar_from_text(&text, "single_tool_justification")?;
                if compatible.len() < 2 && single_justification.is_none() {
                    errors.push(format!(
                        "{}: single-tool stage requires single_tool_justification when compatible_tools has <2 tools",
                        stage_file.display()
                    ));
                }
            }
        }

        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let keys = top_level_keys(&text)?;
            let missing = required_tool
                .iter()
                .filter(|field| !keys.contains(field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!(
                    "{}: missing required fields: {:?}",
                    tool_file.display(),
                    missing
                ));
            }
            let tool_id = scalar_from_text(&text, "tool_id")?;
            if let Some(tool_id_value) = tool_id.clone() {
                if !tool_ids_seen.insert(tool_id_value.clone()) {
                    errors.push(format!(
                        "{}: duplicate tool_id in domain {dom}: {tool_id_value}",
                        tool_file.display()
                    ));
                }
                if !snake_case_re.is_match(&tool_id_value) {
                    errors.push(format!(
                        "{}: tool_id '{tool_id_value}' must be snake_case ([a-z0-9_]+)",
                        tool_file.display()
                    ));
                }
            } else {
                errors.push(format!("{}: missing tool_id", tool_file.display()));
            }
            let scope = scalar_from_text(&text, "scope")?;
            if required_tool_scope
                .as_deref()
                .is_some_and(|required| scope.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: scope must be {} (got {})",
                    tool_file.display(),
                    required_tool_scope.clone().unwrap_or_default(),
                    scope.unwrap_or_default()
                ));
            }
        }

        let metrics_file = dom_dir.join("metrics.yaml");
        let metrics_schema = dom_dir.join("metrics/_schema.yaml");
        if !metrics_schema.is_file() {
            errors.push(format!(
                "{}: missing metrics schema {}",
                dom_dir.display(),
                workspace.rel(&metrics_schema).display()
            ));
        }
        if metrics_file.is_file() {
            let text = read_utf8(&metrics_file)?;
            let keys = top_level_keys(&text)?;
            if metrics_schema.is_file() {
                let required = required_fields(&metrics_schema)?;
                let missing = required
                    .iter()
                    .filter(|field| !keys.contains(field.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    errors.push(format!(
                        "{}: missing required fields from schema: {:?}",
                        metrics_file.display(),
                        missing
                    ));
                }
                let payload_keys = allowed_payload_keys(&metrics_schema)?;
                if !payload_keys.is_empty() && !payload_keys.iter().any(|key| keys.contains(key)) {
                    errors.push(format!(
                        "{}: must define at least one payload key from {:?}",
                        metrics_file.display(),
                        payload_keys
                    ));
                }
            }
            let schema_version = scalar_from_text(&text, "schema_version")?;
            if !schema_version.as_deref().is_some_and(|value| value.starts_with("bijux.")) {
                errors.push(format!(
                    "{}: schema_version must exist and start with 'bijux.'",
                    metrics_file.display()
                ));
            }
            if scalar_from_text(&text, "domain")?.as_deref() != Some(dom.as_str()) {
                errors.push(format!(
                    "{}: domain must be '{dom}' (got {})",
                    metrics_file.display(),
                    scalar_from_text(&text, "domain")?.unwrap_or_default()
                ));
            }
            let has_metric_ids = regex(r"(?m)^metric_ids:\s*$")?.is_match(&text);
            let has_metrics = regex(r"(?m)^metrics:\s*$")?.is_match(&text);
            if !(has_metric_ids || has_metrics) {
                errors.push(format!(
                    "{}: must define either metric_ids: or metrics:",
                    metrics_file.display()
                ));
            }
            if has_metric_ids && !metric_item_re.is_match(&text) {
                errors.push(format!(
                    "{}: metric_ids must contain at least one snake_case metric id",
                    metrics_file.display()
                ));
            }
            if has_metrics && !metric_entry_re.is_match(&text) {
                errors.push(format!(
                    "{}: metrics entries must include id fields",
                    metrics_file.display()
                ));
            }
        } else {
            errors.push(format!("{}: missing metrics.yaml", dom_dir.display()));
        }

        let artifacts_file = dom_dir.join("artifacts.yaml");
        let artifacts_schema = dom_dir.join("artifacts/_schema.yaml");
        if !artifacts_schema.is_file() {
            errors.push(format!(
                "{}: missing artifacts schema {}",
                dom_dir.display(),
                workspace.rel(&artifacts_schema).display()
            ));
        }
        if artifacts_file.is_file() {
            let text = read_utf8(&artifacts_file)?;
            let keys = top_level_keys(&text)?;
            if artifacts_schema.is_file() {
                let required = required_fields(&artifacts_schema)?;
                let missing = required
                    .iter()
                    .filter(|field| !keys.contains(field.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    errors.push(format!(
                        "{}: missing required fields from schema: {:?}",
                        artifacts_file.display(),
                        missing
                    ));
                }
                let payload_keys = allowed_payload_keys(&artifacts_schema)?;
                if !payload_keys.is_empty() && !payload_keys.iter().any(|key| keys.contains(key)) {
                    errors.push(format!(
                        "{}: must define at least one payload key from {:?}",
                        artifacts_file.display(),
                        payload_keys
                    ));
                }
            }
            let schema_version = scalar_from_text(&text, "schema_version")?;
            if !schema_version.as_deref().is_some_and(|value| value.starts_with("bijux.")) {
                errors.push(format!(
                    "{}: schema_version must exist and start with 'bijux.'",
                    artifacts_file.display()
                ));
            }
            if scalar_from_text(&text, "domain")?.as_deref() != Some(dom.as_str()) {
                errors.push(format!(
                    "{}: domain must be '{dom}' (got {})",
                    artifacts_file.display(),
                    scalar_from_text(&text, "domain")?.unwrap_or_default()
                ));
            }
            let has_artifact_ids = regex(r"(?m)^artifact_ids:\s*$")?.is_match(&text);
            let has_artifacts = regex(r"(?m)^artifacts:\s*$")?.is_match(&text);
            if !(has_artifact_ids || has_artifacts) {
                errors.push(format!(
                    "{}: must define either artifact_ids: or artifacts:",
                    artifacts_file.display()
                ));
            }
            if has_artifact_ids && !metric_item_re.is_match(&text) {
                errors.push(format!(
                    "{}: artifact_ids must contain at least one snake_case artifact id",
                    artifacts_file.display()
                ));
            }
            if has_artifacts && !metric_entry_re.is_match(&text) {
                errors.push(format!(
                    "{}: artifacts entries must include id fields",
                    artifacts_file.display()
                ));
            }
        } else {
            errors.push(format!("{}: missing artifacts.yaml", dom_dir.display()));
        }

        let mut fixture_pairs = BTreeSet::new();
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }
            let stage_id = fixture
                .path()
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture path {}", fixture.path().display()))?
                .to_string();
            let tool_id = fixture
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture path {}", fixture.path().display()))?
                .to_string();
            fixture_pairs.insert((stage_id, tool_id));
        }
        for (stage_id, tool_id) in &production_bindings {
            if !stage_id.starts_with(&format!("{dom}.")) {
                continue;
            }
            if !fixture_pairs.contains(&(stage_id.clone(), tool_id.clone())) {
                errors.push(format!(
                    "domain/{dom}/fixtures: missing production fixture for binding ({stage_id}, {tool_id})"
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("domain schema: OK");
    }
    failure_block("domain schema check failed", errors)
}

pub(super) fn check_domain_tool_metadata(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for tool_path in yaml_files(&dom_dir.join("tools"))? {
            if tool_path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_path)?;
            if scalar_from_text(&text, "status")?.as_deref() == Some("out_of_scope") {
                continue;
            }
            let tool_id = scalar_from_text(&text, "tool_id")?;
            let citation = scalar_from_text(&text, "citation")?;
            let homepage = scalar_from_text(&text, "homepage")?
                .or_else(|| scalar_from_text(&text, "upstream").ok().flatten());
            let license_id = scalar_from_text(&text, "license-id")?
                .or_else(|| scalar_from_text(&text, "license").ok().flatten());

            if tool_id.is_none() {
                errors.push(format!("{} missing tool_id", workspace.rel(&tool_path).display()));
            }
            if homepage.is_none() {
                errors.push(format!(
                    "{} missing homepage/upstream",
                    workspace.rel(&tool_path).display()
                ));
            }
            if citation.is_none() {
                errors.push(format!("{} missing citation", workspace.rel(&tool_path).display()));
            }
            if license_id.is_none() {
                errors.push(format!(
                    "{} missing license-id/license",
                    workspace.rel(&tool_path).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("domain tool metadata: OK");
    }
    failure_block("domain tool metadata check failed", errors)
}

pub(super) fn external_tools(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let config = load_toml(&workspace.path("configs/domain/external_tools.toml"))?;
    Ok(config
        .get("non_container_tools")
        .and_then(TomlValue::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default())
}

pub(super) fn check_external_tool_policy(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let mut registry_tools = BTreeSet::new();
    for path in tool_registry_files(workspace) {
        if !path.is_file() {
            continue;
        }
        for row in toml_tools(&path)? {
            let Some(table) = row.as_table() else {
                continue;
            };
            if let Some(tool_id) =
                table.get("id").or_else(|| table.get("tool_id")).and_then(TomlValue::as_str)
            {
                registry_tools.insert(tool_id.trim().to_string());
            }
        }
    }

    let required = [
        "gatk",
        "picard",
        "preseq",
        "bamutil",
        "ngsbriggs",
        "dustmasker",
        "seqfu",
        "seqprep",
        "seqpurge",
        "diamond",
        "fastq_scan",
    ];
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }
            let tool = fixture
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture file {}", fixture.path().display()))?
                .to_string();
            if !registry_tools.contains(&tool) && !external.contains(&tool) {
                errors.push(format!(
                    "{}: tool '{tool}' missing from registries and external_tools allowlist",
                    workspace.rel(fixture.path()).display()
                ));
            }
        }
    }
    let missing_required = required
        .iter()
        .filter(|tool| !external.contains(**tool))
        .map(|tool| (*tool).to_string())
        .collect::<Vec<_>>();
    if !missing_required.is_empty() {
        errors.push(format!(
            "configs/domain/external_tools.toml missing required external markers: {missing_required:?}"
        ));
    }
    if errors.is_empty() {
        return success_line("external tool policy: OK");
    }
    failure_block("external tool policy check failed", errors)
}

pub(super) fn check_fixture_contracts(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let mut errors = Vec::new();
    let stage_re = regex(r#"(?m)^stage_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let tool_re = regex(r#"(?m)^tool_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let readme_stage_re = regex(r"(?im)^\s*[-*]\s*`?([^`\n]+)`?\s*:.*intent")?;
    let snake_case = regex(r"^[a-z0-9_]+$")?;

    for dom_dir in domain_directories(workspace)? {
        let fixtures_root = dom_dir.join("fixtures");
        let readme = fixtures_root.join("README.md");
        let readme_text = if readme.is_file() {
            read_utf8(&readme)?
        } else {
            errors.push(format!("{} missing", workspace.rel(&readme).display()));
            String::new()
        };

        let mut known_stage_ids = BTreeSet::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            if let Some(captures) = stage_re.captures(&text) {
                if let Some(stage_id) = captures.get(1) {
                    known_stage_ids.insert(stage_id.as_str().trim().to_string());
                }
            }
        }

        let mut known_tools = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let tool_id = tool_re
                .captures(&text)
                .and_then(|captures| captures.get(1))
                .map(|value| value.as_str().trim().to_string())
                .unwrap_or_else(|| {
                    tool_file
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .to_string()
                });
            known_tools.insert(tool_id);
        }

        if fixtures_root.is_dir() {
            let mut stage_dirs = fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(entry.path()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            stage_dirs.sort();

            for stage_dir in stage_dirs {
                let stage_name = stage_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow!("invalid fixture directory {}", stage_dir.display()))?
                    .to_string();
                if !known_stage_ids.contains(&stage_name) {
                    errors.push(format!(
                        "{}: fixture stage directory is not a known stage_id",
                        workspace.rel(&stage_dir).display()
                    ));
                }
                if !readme_text.is_empty() {
                    if !readme_text.contains(&stage_name) {
                        errors.push(format!(
                            "{}: missing fixture directory listing for '{stage_name}'",
                            workspace.rel(&readme).display()
                        ));
                    }
                    let has_intent = readme_stage_re.captures_iter(&readme_text).any(|captures| {
                        captures.get(1).is_some_and(|value| value.as_str().trim() == stage_name)
                    });
                    if !has_intent {
                        errors.push(format!(
                            "{}: '{stage_name}' entry must include intent",
                            workspace.rel(&readme).display()
                        ));
                    }
                }

                let mut fixture_files = fs::read_dir(&stage_dir)
                    .with_context(|| format!("read {}", stage_dir.display()))?
                    .filter_map(std::result::Result::ok)
                    .filter(|entry| {
                        entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
                    })
                    .collect::<Vec<_>>();
                fixture_files.sort_by_key(std::fs::DirEntry::path);

                for fixture in fixture_files {
                    let path = fixture.path();
                    let text = read_utf8(&path)?.trim().to_string();
                    if !text.contains('=') {
                        let parts = text.split_whitespace().collect::<Vec<_>>();
                        if parts.len() < 2 {
                            errors.push(format!(
                                "{}: invalid fixture format",
                                workspace.rel(&path).display()
                            ));
                            continue;
                        }
                        let tool = parts[1].trim().to_string();
                        if !known_stage_ids.contains(&stage_name) {
                            errors.push(format!(
                                "{}: fixture path stage '{stage_name}' is unknown",
                                workspace.rel(&path).display()
                            ));
                        }
                        if !known_tools.contains(&tool) && !external.contains(&tool) {
                            errors.push(format!(
                                "{}: legacy fixture tool '{tool}' not found in domain tools or external tools",
                                workspace.rel(&path).display()
                            ));
                        }
                        if !external.contains(&tool) {
                            errors.push(format!(
                                "{}: legacy fixture format; use key=value contract fields",
                                workspace.rel(&path).display()
                            ));
                        }
                        continue;
                    }

                    let kv = text
                        .lines()
                        .filter_map(|line| line.split_once('='))
                        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                        .collect::<BTreeMap<_, _>>();
                    for required_key in [
                        "tool",
                        "tool_version",
                        "command",
                        "args",
                        "expected_outputs",
                        "expected_stdout_patterns",
                        "stage",
                    ] {
                        if !kv.contains_key(required_key) {
                            errors.push(format!(
                                "{}: missing required key '{required_key}'",
                                workspace.rel(&path).display()
                            ));
                        }
                    }
                    if let Some(tool) = kv.get("tool") {
                        if !snake_case.is_match(tool) {
                            errors.push(format!(
                                "{}: tool id '{}' must be snake_case ([a-z0-9_]+)",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                        let stem =
                            path.file_stem().and_then(|name| name.to_str()).unwrap_or_default();
                        if tool != stem {
                            errors.push(format!(
                                "{}: tool field '{}' must match fixture filename stem '{}'",
                                workspace.rel(&path).display(),
                                tool,
                                stem
                            ));
                        }
                        if !known_tools.contains(tool) && !external.contains(tool) {
                            errors.push(format!(
                                "{}: tool '{}' not found in domain tools or external tools policy",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                        let shipping = kv.get("shipping").cloned().unwrap_or_default();
                        if shipping == "external" && !external.contains(tool) {
                            errors.push(format!(
                                "{}: shipping=external requires tool in configs/domain/external_tools.toml",
                                workspace.rel(&path).display()
                            ));
                        }
                        if external.contains(tool) && shipping != "external" {
                            errors.push(format!(
                                "{}: external tool '{}' must declare shipping=external",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                    }
                    if let Some(stage_value) = kv.get("stage") {
                        if stage_value != &stage_name {
                            errors.push(format!(
                                "{}: stage mismatch ({} != {})",
                                workspace.rel(&path).display(),
                                stage_value,
                                stage_name
                            ));
                        }
                    }
                    if !known_stage_ids.contains(&stage_name) {
                        errors.push(format!(
                            "{}: fixture path stage '{}' is unknown",
                            workspace.rel(&path).display(),
                            stage_name
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("fixture contracts: OK");
    }
    failure_block("fixture contracts check failed", errors)
}
