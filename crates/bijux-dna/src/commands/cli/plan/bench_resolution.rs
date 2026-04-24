use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::commands::cli::env::registry_tools_for_stage;

pub(super) fn resolve_bench_tools(stage: &str, raw_tools: &[String]) -> Result<Vec<String>> {
    let mut normalized = raw_tools
        .iter()
        .map(|tool| tool.trim().to_lowercase())
        .filter(|tool| !tool.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();

    let mode = if normalized.is_empty() {
        "auto"
    } else if normalized.len() == 1 && normalized[0] == "all" {
        "all"
    } else if normalized.len() == 1 && normalized[0] == "auto" {
        "auto"
    } else {
        if normalized.iter().any(|value| value == "auto" || value == "all") {
            return Err(anyhow!("--tools accepts either `auto`, `all`, or an explicit CSV list"));
        }
        "csv"
    };

    let registry_path = resolve_registry_path()?;
    let all_tools = registry_tools_for_stage(&registry_path, stage, None, "all")?;
    if all_tools.is_empty() {
        return Err(anyhow!("no compatible tools found for stage `{stage}`"));
    }
    let mut selected = match mode {
        "auto" => registry_tools_for_stage(&registry_path, stage, None, "primary")?,
        "all" => all_tools.clone(),
        _ => normalized,
    };
    selected.sort();
    selected.dedup();

    if selected.is_empty() {
        return Err(anyhow!("resolved empty tool set for stage `{stage}`"));
    }
    for tool in &selected {
        if !all_tools.contains(tool) {
            return Err(anyhow!("tool `{tool}` is not compatible with stage `{stage}`"));
        }
    }
    Ok(selected)
}

fn resolve_registry_path() -> Result<PathBuf> {
    let cwd = std::env::current_dir().map_err(|err| anyhow!("resolve cwd: {err}"))?;
    let cwd_registry = bijux_dna_infra::configs_file(&cwd, "ci/registry/tool_registry.toml");
    if cwd_registry.exists() {
        return Ok(cwd_registry);
    }

    let workspace_registry = bijux_dna_infra::configs_file(
        crate::commands::support::workspace_root::resolve_repo_root()?.as_path(),
        "ci/registry/tool_registry.toml",
    );
    if workspace_registry.exists() {
        return Ok(workspace_registry);
    }

    Ok(cwd_registry)
}

pub(super) fn bench_tools_resolved_implicitly(raw_tools: &[String]) -> bool {
    let mut normalized = raw_tools
        .iter()
        .map(|tool| tool.trim().to_lowercase())
        .filter(|tool| !tool.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized.is_empty()
        || (normalized.len() == 1 && matches!(normalized[0].as_str(), "auto" | "all"))
}

pub(super) fn normalize_validate_failure_flags(
    strict: bool,
    validation_mode: Option<&str>,
) -> Result<(bool, Option<String>)> {
    let Some(validation_mode) = validation_mode else {
        return Ok((strict, None));
    };
    let normalized = validation_mode.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "strict" => Ok((true, Some(normalized))),
        "report_only" => {
            if strict {
                Err(anyhow!(
                    "--strict conflicts with --validation-mode report_only for fastq.validate_reads"
                ))
            } else {
                Ok((false, Some(normalized)))
            }
        }
        _ => Ok((strict, Some(validation_mode.to_string()))),
    }
}
