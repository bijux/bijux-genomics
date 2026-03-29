use std::collections::BTreeSet;

use anyhow::Result;
use toml::Value as TomlValue;
use walkdir::WalkDir;

use super::support::*;
use super::toml_stages;
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

fn planner_stage_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut stage_ids = BTreeSet::new();
    for rel in [
        "configs/ci/stages/stages.toml",
        "configs/ci/stages/stages_vcf.toml",
        "configs/ci/stages/stages_vcf_downstream.toml",
    ] {
        for row in toml_stages(&workspace.path(rel))? {
            if let Some(stage_id) = row
                .as_table()
                .and_then(|table| table.get("id"))
                .and_then(TomlValue::as_str)
            {
                stage_ids.insert(stage_id.trim().to_string());
            }
        }
    }
    Ok(stage_ids)
}

pub(super) fn check_planner_fixture_coverage(
    workspace: &Workspace,
) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for stage_id in planner_stage_ids(workspace)? {
        let domain = stage_id.split('.').next().unwrap_or_default();
        let fixture_dir = workspace.path(&format!("domain/{domain}/fixtures/{stage_id}"));
        let has_files = fixture_dir.is_dir()
            && WalkDir::new(&fixture_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .any(|entry| entry.file_type().is_file());
        if !has_files {
            errors.push(format!(
                "{} missing fixture files for planner stage '{stage_id}'",
                workspace.rel(&fixture_dir).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("planner fixture coverage: OK");
    }
    failure_block("planner fixture coverage check failed", errors)
}

pub(super) fn check_planner_stage_coverage(
    workspace: &Workspace,
) -> Result<DomainCommandOutcome> {
    let planner_stage_ids = planner_stage_ids(workspace)?;
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
                continue;
            };
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if status != "supported" {
                continue;
            }
            if !planner_stage_ids.contains(&stage_id) {
                errors.push(format!(
                    "{}: supported stage '{stage_id}' missing planner coverage in configs/ci/stages/*.toml",
                    workspace.rel(&stage_file).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("planner stage coverage: OK");
    }
    failure_block("planner stage coverage check failed", errors)
}
