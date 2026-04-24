use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::id_catalog;
use serde::Serialize;
use toml::Value as TomlValue;
use walkdir::WalkDir;

use super::domain_workflow::{
    declared_stage_key, declared_tool_key, domain_directories, failure_block, list_block,
    read_utf8, regex, scalar_from_text, success_line, write_utf8, yaml_files,
};
use super::schema_policy::external_tools;
use super::{
    cargo_registry_list_stages, cargo_registry_list_tools, cargo_registry_stage_tools, toml_tools,
};
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

#[derive(Debug, Serialize, PartialEq, Eq)]
struct InventoryRow {
    domain: String,
    stages: usize,
    tools: usize,
    fixture_stage_dirs: usize,
    fixture_files: usize,
    has_artifacts_yaml: bool,
    has_metrics_yaml: bool,
    has_default_settings_doc: bool,
}

fn build_inventory_rows(workspace: &Workspace) -> Result<Vec<InventoryRow>> {
    let mut rows = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let domain = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let stages = yaml_files(&dom_dir.join("stages"))?
            .into_iter()
            .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("_schema.yaml"))
            .count();
        let tools = yaml_files(&dom_dir.join("tools"))?
            .into_iter()
            .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("_schema.yaml"))
            .count();
        let fixtures_root = dom_dir.join("fixtures");
        let fixture_stage_dirs = if fixtures_root.is_dir() {
            fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(()),
                    _ => None,
                })
                .count()
        } else {
            0
        };
        let fixture_files = if fixtures_root.is_dir() {
            WalkDir::new(&fixtures_root)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
                })
                .count()
        } else {
            0
        };
        rows.push(InventoryRow {
            domain,
            stages,
            tools,
            fixture_stage_dirs,
            fixture_files,
            has_artifacts_yaml: dom_dir.join("artifacts.yaml").is_file(),
            has_metrics_yaml: dom_dir.join("metrics.yaml").is_file(),
            has_default_settings_doc: dom_dir.join("docs/DEFAULT_SETTINGS.md").is_file(),
        });
    }
    Ok(rows)
}

fn render_inventory_json(rows: &[InventoryRow]) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "bijux.domain.inventory.v1",
            "domains": rows,
        }))?
    ))
}

fn render_inventory_markdown(rows: &[InventoryRow]) -> String {
    let mut lines = vec![
        "# Domain Inventory".to_string(),
        String::new(),
        "| Domain | Stages | Tools | Fixture Stage Dirs | Fixture Files | artifacts.yaml | metrics.yaml | DEFAULT_SETTINGS.md |".to_string(),
        "|---|---:|---:|---:|---:|:---:|:---:|:---:|".to_string(),
    ];
    for row in rows {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            row.domain,
            row.stages,
            row.tools,
            row.fixture_stage_dirs,
            row.fixture_files,
            yes_no(row.has_artifacts_yaml),
            yes_no(row.has_metrics_yaml),
            yes_no(row.has_default_settings_doc)
        ));
    }
    format!("{}\n", lines.join("\n"))
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

pub(super) fn check_inventory(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let rows_first = build_inventory_rows(workspace)?;
    let json_first = render_inventory_json(&rows_first)?;
    let md_first = render_inventory_markdown(&rows_first);
    let rows_second = build_inventory_rows(workspace)?;
    let json_second = render_inventory_json(&rows_second)?;
    let md_second = render_inventory_markdown(&rows_second);

    if json_first != json_second {
        return Ok(DomainCommandOutcome::failure(
            "domain inventory is non-deterministic across consecutive generations\n",
        ));
    }
    if md_first != md_second {
        return Ok(DomainCommandOutcome::failure(
            "domain inventory markdown is non-deterministic across consecutive generations\n",
        ));
    }

    let out_json = workspace.path("artifacts/domain/inventory.json");
    let out_md = workspace.path("artifacts/domain/inventory.md");
    write_utf8(&out_json, &json_first)?;
    write_utf8(&out_md, &md_first)?;
    success_line(format!("domain inventory: OK ({}, {})", out_json.display(), out_md.display()))
}

pub(super) fn check_orphan_files(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external_tools = external_tools(workspace)?;
    let mut registry_tools_by_domain = BTreeMap::<String, BTreeSet<String>>::new();
    let registry_dir = workspace.path("configs/ci/registry");
    let mut registry_files =
        fs::read_dir(&registry_dir)
            .with_context(|| format!("read {}", registry_dir.display()))?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry.path().file_name().and_then(|name| name.to_str()).is_some_and(|name| {
                    name.starts_with("tool_registry") && name.ends_with(".toml")
                })
            })
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
    registry_files.sort();
    for registry in registry_files {
        for row in toml_tools(&registry)? {
            let Some(table) = row.as_table() else {
                continue;
            };
            let tool_id = table
                .get("tool_id")
                .or_else(|| table.get("id"))
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let status = table
                .get("status")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if tool_id.is_empty()
                || tool_id.contains('.')
                || !matches!(status.as_str(), "production" | "supported")
            {
                continue;
            }
            for binding in table.get("bindings").and_then(TomlValue::as_array).into_iter().flatten()
            {
                let Some(stage_id) = binding.as_str() else {
                    continue;
                };
                let Some((domain, _)) = stage_id.split_once('.') else {
                    continue;
                };
                registry_tools_by_domain
                    .entry(domain.to_string())
                    .or_default()
                    .insert(tool_id.clone());
            }
        }
    }

    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let index = dom_dir.join("index.yaml");
        if !index.is_file() {
            continue;
        }
        let text = read_utf8(&index)?;
        let indexed_stages = list_block(&text, "stage_ids")?.into_iter().collect::<BTreeSet<_>>();
        let indexed_tools = list_block(&text, "tool_ids")?.into_iter().collect::<BTreeSet<_>>();
        let mut fixture_tools = BTreeSet::new();
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
            })
        {
            if let Some(stem) = fixture.path().file_stem().and_then(|name| name.to_str()) {
                fixture_tools.insert(stem.to_string());
            }
        }

        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage_id = declared_stage_key(&stage_file)?.unwrap_or_else(|| {
                stage_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
            if !indexed_stages.contains(&stage_id) {
                errors.push(format!(
                    "{}: orphan stage file not referenced by index.yaml",
                    workspace.rel(&stage_file).display()
                ));
            }
        }

        let mut domain_tool_ids = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool_id = declared_tool_key(&tool_file)?.unwrap_or_else(|| {
                tool_file.file_stem().and_then(|name| name.to_str()).unwrap_or_default().to_string()
            });
            domain_tool_ids.insert(tool_id.clone());
            if !indexed_tools.contains(&tool_id)
                && !fixture_tools.contains(&tool_id)
                && !registry_tools_by_domain.get(&dom).is_some_and(|tools| tools.contains(&tool_id))
            {
                errors.push(format!(
                    "{}: orphan tool file not referenced by index.yaml, fixtures, or registry bindings",
                    workspace.rel(&tool_file).display()
                ));
            }
        }

        for registry_tool in registry_tools_by_domain.get(&dom).cloned().unwrap_or_default() {
            if !domain_tool_ids.contains(&registry_tool) && !external_tools.contains(&registry_tool)
            {
                errors.push(format!(
                    "domain/{dom}/tools: missing tool yaml for registry-bound tool '{registry_tool}' (or declare external tool policy)"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("orphan stage/tool: OK");
    }
    failure_block("orphan stage/tool check failed", errors)
}

pub(super) fn generate_inventory(
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    if args.len() > 2 {
        return Ok(DomainCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "Usage: cargo run -p bijux-dna-dev -- domain run generate-inventory -- [<json-path> [<markdown-path>]]\n".to_string(),
        });
    }
    let out_json = args
        .first()
        .map_or_else(|| workspace.path("artifacts/domain/inventory.json"), PathBuf::from);
    let out_md =
        args.get(1).map_or_else(|| workspace.path("artifacts/domain/inventory.md"), PathBuf::from);
    let rows = build_inventory_rows(workspace)?;
    write_utf8(&out_json, &render_inventory_json(&rows)?)?;
    write_utf8(&out_md, &render_inventory_markdown(&rows))?;
    Ok(DomainCommandOutcome::success(format!(
        "generated {}\ngenerated {}\n",
        out_json.display(),
        out_md.display()
    )))
}

pub(super) fn inventory_drift(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut domain_tools = BTreeSet::new();
    for domain in ["fastq", "bam"] {
        for tool_file in yaml_files(&workspace.path(&format!("domain/{domain}/tools")))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if matches!(status.as_str(), "production" | "supported") {
                if let Some(tool_id) = scalar_from_text(&text, "tool_id")? {
                    domain_tools.insert(tool_id);
                }
            }
        }
    }

    let mut domain_stages = BTreeSet::new();
    for domain in ["fastq", "bam"] {
        for stage_file in yaml_files(&workspace.path(&format!("domain/{domain}/stages")))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if matches!(status.as_str(), "production" | "supported") {
                if let Some(stage_id) = scalar_from_text(&text, "stage_id")? {
                    domain_stages.insert(stage_id);
                }
            }
        }
    }

    let registry_tools = cargo_registry_list_tools(workspace)?;
    let registry_stages = cargo_registry_list_stages(workspace)?;

    let tool_ref_re = regex(r#"ToolId::from_static\("([a-z0-9_\-]+)"\)"#)?;
    let stage_ref_re = regex(r#"StageId::from_static\("([a-z0-9._-]+)"\)"#)?;
    let mut code_tools = BTreeSet::new();
    let mut code_stages_raw = BTreeSet::new();
    let synthetic_core_test = format!("{}test", id_catalog::CORE_PREFIX);
    for entry in WalkDir::new(workspace.path("crates"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let text = read_utf8(entry.path()).unwrap_or_default();
        for captures in tool_ref_re.captures_iter(&text) {
            if let Some(value) = captures.get(1).map(|capture| capture.as_str()) {
                if !matches!(value, "tool" | "planner" | "unknown") {
                    code_tools.insert(value.to_string());
                }
            }
        }
        for captures in stage_ref_re.captures_iter(&text) {
            if let Some(value) = captures.get(1).map(|capture| capture.as_str()) {
                if value == synthetic_core_test
                    || value == "report.aggregate"
                    || value.starts_with("stage.")
                    || value == id_catalog::FASTQ_PREPROCESS
                {
                    continue;
                }
                code_stages_raw.insert(value.to_string());
            }
        }
    }
    let code_stages = code_stages_raw
        .into_iter()
        .filter(|stage_id| domain_stages.contains(stage_id))
        .collect::<BTreeSet<_>>();

    let stage_tools_re = regex(r"stage-tools ([a-z0-9._-]+) all")?;
    let mut make_stage_ids = BTreeSet::new();
    for file in WalkDir::new(workspace.path("makes"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let text = read_utf8(file.path()).unwrap_or_default();
        for captures in stage_tools_re.captures_iter(&text) {
            if let Some(stage_id) = captures.get(1).map(|value| value.as_str()) {
                make_stage_ids.insert(stage_id.to_string());
            }
        }
    }
    let makefile_text = read_utf8(&workspace.path("Makefile")).unwrap_or_default();
    for captures in stage_tools_re.captures_iter(&makefile_text) {
        if let Some(stage_id) = captures.get(1).map(|value| value.as_str()) {
            make_stage_ids.insert(stage_id.to_string());
        }
    }
    let mut make_tools = BTreeSet::new();
    for stage_id in make_stage_ids {
        make_tools.extend(cargo_registry_stage_tools(workspace, &stage_id)?);
    }

    let mut diffs = Vec::new();
    push_diff(&mut diffs, &domain_tools, &registry_tools, "domain tools missing from registry");
    push_diff(
        &mut diffs,
        &code_tools,
        &registry_tools,
        "code-referenced tools missing from registry",
    );
    push_diff(
        &mut diffs,
        &make_tools,
        &registry_tools,
        "make-referenced tools missing from registry",
    );
    push_diff(&mut diffs, &registry_tools, &domain_tools, "registry tools missing from domain");
    push_diff(
        &mut diffs,
        &domain_stages,
        &registry_stages,
        "domain stages missing from generated configs/ci/stages/stages.toml",
    );
    push_diff(
        &mut diffs,
        &registry_stages,
        &domain_stages,
        "generated configs/ci/stages/stages.toml stages missing from domain",
    );
    push_diff(
        &mut diffs,
        &code_stages,
        &registry_stages,
        "code-referenced stages missing from generated configs/ci/stages/stages.toml",
    );

    let mut stdout = String::new();
    if !diffs.is_empty() {
        stdout.push_str(&diffs.join(""));
    }
    stdout.push_str("--- inventory counts ---\n");
    stdout.push_str(&format!("domain:   {}\n", domain_tools.len()));
    stdout.push_str(&format!("registry: {}\n", registry_tools.len()));
    stdout.push_str(&format!("code:     {}\n", code_tools.len()));
    stdout.push_str(&format!("make:     {}\n", make_tools.len()));
    stdout.push_str(&format!("stages(domain): {}\n", domain_stages.len()));
    stdout.push_str(&format!("stages(registry): {}\n", registry_stages.len()));
    stdout.push_str(&format!("stages(code): {}\n", code_stages.len()));

    if diffs.is_empty() {
        stdout.push_str("domain-inventory-drift: OK\n");
        return Ok(DomainCommandOutcome::success(stdout));
    }

    Ok(DomainCommandOutcome {
        exit_code: 1,
        stdout,
        stderr: "domain-inventory-drift: mismatch detected\n".to_string(),
    })
}

fn push_diff(
    output: &mut Vec<String>,
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    title: &str,
) {
    let missing = left.difference(right).cloned().collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    let mut block = String::new();
    block.push_str("[DIFF] ");
    block.push_str(title);
    block.push('\n');
    for entry in missing {
        block.push_str("  - ");
        block.push_str(&entry);
        block.push('\n');
    }
    output.push(block);
}
