use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use walkdir::WalkDir;

use super::domain_workflow::{
    declared_stage_key, declared_tool_key, domain_directories, failure_block, inline_list,
    list_block, parse_status, read_utf8, regex, scalar_from_text, success_line, write_utf8,
    yaml_files,
};
use super::DOMAIN_INDEX_REGENERATE_PREFIX;
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

pub(super) fn render_domain_index(workspace: &Workspace, dom: &str) -> Result<String> {
    let dom_dir = workspace.path(&format!("domain/{dom}"));
    let index_path = dom_dir.join("index.yaml");
    if !index_path.is_file() {
        bail!("missing {}", index_path.display());
    }

    let mut stage_ids = BTreeSet::new();
    let mut governed_stage_ids = BTreeSet::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        if let Some(stage_id) = declared_stage_key(&stage_file)? {
            if parse_status(&stage_file)?.as_deref() == Some("supported") {
                governed_stage_ids.insert(stage_id.clone());
            }
            stage_ids.insert(stage_id);
        }
    }

    let mut tool_ids = BTreeSet::new();
    let mut governed_tool_ids = BTreeSet::new();
    for tool_file in yaml_files(&dom_dir.join("tools"))? {
        if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        if let Some(tool_id) = declared_tool_key(&tool_file)? {
            if parse_status(&tool_file)?.as_deref() == Some("supported") {
                governed_tool_ids.insert(tool_id.clone());
            }
            tool_ids.insert(tool_id);
        }
    }

    let existing = read_utf8(&index_path)?;
    let mut lines = existing
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if lines
        .first()
        .is_some_and(|line| line == "# GENERATED FILE - DO NOT EDIT")
    {
        lines.remove(0);
        if lines
            .first()
            .is_some_and(|line| line.starts_with("# Regenerate with:"))
        {
            lines.remove(0);
        }
        while lines.first().is_some_and(|line| line.trim().is_empty()) {
            lines.remove(0);
        }
    }
    let mut body_lines = lines;

    replace_or_insert_block(
        &mut body_lines,
        "stage_ids",
        stage_ids
            .iter()
            .map(|stage_id| format!("  - {stage_id}"))
            .collect(),
        Some("domain_version"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "tool_ids",
        tool_ids
            .iter()
            .map(|tool_id| format!("  - {tool_id}"))
            .collect(),
        Some("stage_ids"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "governed_stage_ids",
        governed_stage_ids
            .iter()
            .map(|stage_id| format!("  - {stage_id}"))
            .collect(),
        Some("tool_ids"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "governed_tool_ids",
        governed_tool_ids
            .iter()
            .map(|tool_id| format!("  - {tool_id}"))
            .collect(),
        Some("governed_stage_ids"),
    )?;
    replace_block(
        &mut body_lines,
        "stage_tool_compatibility",
        render_stage_tool_compatibility_block(&dom_dir)?,
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "stage_tool_integration",
        render_stage_tool_integration_block(&dom_dir)?,
        Some("stage_tool_compatibility"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "reference_index_compatibility",
        render_reference_index_compatibility_block(&dom_dir)?,
        Some("stage_tool_integration"),
    )?;

    if !body_lines
        .iter()
        .any(|line| line.starts_with("domain_version:"))
    {
        let Some(domain_line_index) = body_lines
            .iter()
            .position(|line| line.starts_with("domain:"))
        else {
            bail!("{}: missing domain: field", index_path.display());
        };
        let version = if dom == "vcf" { "v2" } else { "v1" };
        body_lines.insert(domain_line_index + 1, format!("domain_version: {version}"));
    }

    let header = [
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        format!("{DOMAIN_INDEX_REGENERATE_PREFIX}{dom}"),
        String::new(),
    ];
    Ok(format!(
        "{}\n",
        header
            .into_iter()
            .chain(body_lines)
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn render_stage_tool_compatibility_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut stage_map = BTreeMap::<String, Vec<String>>::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&stage_file)?;
        let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
            continue;
        };
        let compatible = {
            let block = list_block(&text, "compatible_tools")?;
            if block.is_empty() {
                inline_list(&text, "compatible_tools")?
            } else {
                block
            }
        };
        stage_map.insert(stage_id, compatible);
    }

    for (stage_id, tools) in stage_map {
        rendered.push(format!("  {stage_id}:"));
        rendered.extend(tools.into_iter().map(|tool_id| format!("  - {tool_id}")));
    }
    Ok(rendered)
}

fn render_stage_tool_integration_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut stage_map = BTreeMap::<String, BTreeMap<String, String>>::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&stage_file)?;
        let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
            continue;
        };
        let mut integration = BTreeMap::new();
        let compatible = {
            let block = list_block(&text, "compatible_tools")?;
            if block.is_empty() {
                inline_list(&text, "compatible_tools")?
            } else {
                block
            }
        };
        for tool_id in compatible {
            integration.insert(tool_id, "governed_contract".to_string());
        }
        let planned = {
            let block = list_block(&text, "planned_out_of_scope")?;
            if block.is_empty() {
                inline_list(&text, "planned_out_of_scope")?
            } else {
                block
            }
        };
        for tool_id in planned {
            integration.insert(tool_id, "planned_contract".to_string());
        }
        stage_map.insert(stage_id, integration);
    }

    for (stage_id, tool_map) in stage_map {
        rendered.push(format!("  {stage_id}:"));
        for (tool_id, level) in tool_map {
            rendered.push(format!("    {tool_id}: {level}"));
        }
    }
    Ok(rendered)
}

fn render_reference_index_compatibility_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut tool_map = BTreeMap::<String, Vec<String>>::new();
    for tool_file in yaml_files(&dom_dir.join("tools"))? {
        if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&tool_file)?;
        let Some(tool_id) = scalar_from_text(&text, "tool_id")? else {
            continue;
        };
        let backends = {
            let block = list_block(&text, "reference_index_backends")?;
            if block.is_empty() {
                inline_list(&text, "reference_index_backends")?
            } else {
                block
            }
        };
        if !backends.is_empty() {
            tool_map.insert(tool_id, backends);
        }
    }

    for (tool_id, backends) in tool_map {
        rendered.push(format!("  {tool_id}:"));
        rendered.extend(backends.into_iter().map(|backend| format!("  - {backend}")));
    }
    Ok(rendered)
}

fn replace_block(lines: &mut Vec<String>, key: &str, items: Vec<String>) -> Result<()> {
    let start_re = regex(&format!(r"^{}:\s*$", regex::escape(key)))?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
    let Some(start) = lines.iter().position(|line| start_re.is_match(line)) else {
        bail!("missing {key}: block");
    };
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
            end = index;
            break;
        }
    }
    let mut replacement = vec![format!("{key}:")];
    replacement.extend(items);
    lines.splice(start..end, replacement);
    Ok(())
}

fn replace_or_insert_block(
    lines: &mut Vec<String>,
    key: &str,
    items: Vec<String>,
    after_key: Option<&str>,
) -> Result<()> {
    if lines.iter().any(|line| line == &format!("{key}:")) {
        return replace_block(lines, key, items);
    }
    let mut replacement = vec![format!("{key}:")];
    replacement.extend(items);
    let insert_at = if let Some(after_key) = after_key {
        let start_re = regex(&format!(r"^{}:\s*$", regex::escape(after_key)))?;
        let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
        let Some(start) = lines.iter().position(|line| start_re.is_match(line)) else {
            bail!("missing {after_key}: block");
        };
        let mut end = lines.len();
        for (index, line) in lines.iter().enumerate().skip(start + 1) {
            if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
                end = index;
                break;
            }
        }
        end
    } else {
        lines.len()
    };
    lines.splice(insert_at..insert_at, replacement);
    Ok(())
}

pub(super) fn check_domain_index(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?;
        let index_path = dom_dir.join("index.yaml");
        if !index_path.is_file() {
            continue;
        }
        let actual = read_utf8(&index_path)?;
        let mut actual_lines = actual.lines();
        if actual_lines.next() != Some("# GENERATED FILE - DO NOT EDIT") {
            errors.push(format!(
                "domain index: missing generated header in domain/{dom}/index.yaml"
            ));
        }
        if actual_lines
            .next()
            .is_none_or(|line| line != format!("{DOMAIN_INDEX_REGENERATE_PREFIX}{dom}"))
        {
            errors.push(format!(
                "domain index: missing regenerate header in domain/{dom}/index.yaml"
            ));
        }

        let expected = render_domain_index(workspace, dom)?;
        if expected != actual {
            errors.push(format!(
                "domain index drift for domain/{dom}/index.yaml; regenerate with cargo run -p bijux-dna-dev -- domain run generate-index -- {dom}"
            ));
        }

        let stage_ids = list_block(&actual, "stage_ids")?;
        let tool_ids = list_block(&actual, "tool_ids")?
            .into_iter()
            .collect::<BTreeSet<_>>();

        let mut stage_file_map = BTreeMap::<String, PathBuf>::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            if let Some(stage_id) = declared_stage_key(&stage_file)? {
                stage_file_map.insert(stage_id, stage_file);
            }
        }

        for stage_id in &stage_ids {
            if !stage_file_map.contains_key(stage_id) {
                errors.push(format!(
                    "{}: stage {stage_id} is listed but no stages/*.yaml declares it",
                    workspace.rel(&index_path).display()
                ));
                continue;
            }
            let fixture_dir = dom_dir.join("fixtures").join(stage_id);
            let has_files = fixture_dir.exists()
                && WalkDir::new(&fixture_dir)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .any(|entry| entry.file_type().is_file());
            if !has_files {
                errors.push(format!(
                    "{}: stage {stage_id} must have at least one fixture under {}",
                    workspace.rel(&index_path).display(),
                    workspace.rel(&fixture_dir).display()
                ));
            }
        }

        let mut declared_tools = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            if let Some(tool_id) = declared_tool_key(&tool_file)? {
                declared_tools.insert(tool_id);
            }
        }

        let fixtures_root = dom_dir.join("fixtures");
        if fixtures_root.is_dir() {
            let stage_dirs = fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(entry.path()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            for stage_dir in stage_dirs {
                for fixture in fs::read_dir(&stage_dir)
                    .with_context(|| format!("read {}", stage_dir.display()))?
                    .filter_map(std::result::Result::ok)
                {
                    if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                        continue;
                    }
                    let tool_id = fixture
                        .path()
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            anyhow!("invalid fixture file {}", fixture.path().display())
                        })?
                        .to_string();
                    if !declared_tools.contains(&tool_id) {
                        errors.push(format!(
                            "{}: fixture tool '{tool_id}' missing matching tools/<tool>.yaml in domain/{dom}",
                            workspace.rel(&fixture.path()).display()
                        ));
                    }
                }
            }
        }

        for tool_id in &tool_ids {
            if !declared_tools.contains(tool_id) {
                errors.push(format!(
                    "{}: tool {tool_id} listed in tool_ids but missing tools/<tool>.yaml",
                    workspace.rel(&index_path).display()
                ));
            }
        }

        for stage_id in stage_file_map.keys() {
            if !stage_ids.contains(stage_id) {
                errors.push(format!(
                    "{}: missing stage_id listing for stages file declaring '{stage_id}'",
                    workspace.rel(&index_path).display()
                ));
            }
        }
        for tool_id in &declared_tools {
            if !tool_ids.contains(tool_id) {
                errors.push(format!(
                    "{}: missing tool_id listing for tools file declaring '{tool_id}'",
                    workspace.rel(&index_path).display()
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("domain index/completeness: OK");
    }
    failure_block("domain completeness check failed", errors)
}

pub(super) fn generate_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    if args.len() != 1 {
        return Ok(DomainCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr:
                "Usage: cargo run -p bijux-dna-dev -- domain run generate-index -- <domain>|--all\n"
                    .to_string(),
        });
    }
    let domains = if args[0] == "--all" {
        domain_directories(workspace)?
            .into_iter()
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>()
    } else {
        vec![args[0].clone()]
    };
    let mut stdout = String::new();
    for dom in domains {
        let index_path = workspace.path(&format!("domain/{dom}/index.yaml"));
        let rendered = render_domain_index(workspace, &dom)?;
        write_utf8(&index_path, &rendered)?;
        stdout.push_str("generated ");
        stdout.push_str(&index_path.display().to_string());
        stdout.push('\n');
    }
    Ok(DomainCommandOutcome::success(stdout))
}
