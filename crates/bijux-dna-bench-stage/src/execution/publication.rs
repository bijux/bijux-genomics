use super::shell::render_shell_command;
use crate::paths::{path_relative_to_repo, resolve_repo_pathbuf};
use crate::{
    validate_benchmark_local_stage_commands,
    BenchmarkLocalStageCommandArgvEntry as BenchLocalStageCommandArgvEntry,
    BenchmarkLocalStageCommandEntry, BenchmarkLocalStageCommandManifest,
    BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
};
use anyhow::{anyhow, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Publish a fully admitted script, argv stream and manifest for an explicit repository root.
///
/// # Errors
/// Refuses invalid commands, unresolvable roots and conflicting output kinds before writes.
pub fn publish_stage_commands(
    repo_root: &Path,
    output_path: &Path,
    commands: Vec<BenchmarkLocalStageCommandEntry>,
) -> Result<BenchmarkLocalStageCommandManifest> {
    let root = fs::canonicalize(repo_root)
        .with_context(|| format!("resolve repository {}", repo_root.display()))?;
    let script_path = resolve_repo_pathbuf(&root, output_path);
    let manifest_path = rendered_stage_commands_manifest_path(&script_path);
    let argv_path = rendered_stage_commands_argv_path(&script_path);
    let report = BenchmarkLocalStageCommandManifest {
        schema_version: BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3.to_string(),
        script_output_path: path_relative_to_repo(&root, &script_path),
        manifest_output_path: path_relative_to_repo(&root, &manifest_path),
        argv_output_path: path_relative_to_repo(&root, &argv_path),
        command_count: commands.len(),
        commands,
    };
    validate_benchmark_local_stage_commands(&report)?;
    for entry in &report.commands {
        anyhow::ensure!(
            entry.command == render_shell_command(&entry.argv),
            "stage command presentation disagrees with argv: {}",
            entry.stage_id
        );
    }
    for path in [&script_path, &manifest_path, &argv_path] {
        super::fixtures::admit_regular_output(path)?;
    }
    let script = render_script(&root, &script_path, &report.commands)?;
    let rows = report
        .commands
        .iter()
        .map(|entry| BenchLocalStageCommandArgvEntry {
            stage_id: entry.stage_id.clone(),
            tool_id: entry.tool_id.clone(),
            readiness_kind: entry.readiness_kind,
            argv: entry.argv.clone(),
        })
        .collect::<Vec<_>>();
    let argv = render_stage_command_argv_jsonl(&rows)?;
    bijux_dna_infra::atomic_write_bytes(&script_path, script.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&argv_path, argv.as_bytes())?;
    bijux_dna_infra::atomic_write_json(&manifest_path, &report)?;
    Ok(report)
}

fn render_script(
    root: &Path,
    script_path: &Path,
    commands: &[BenchmarkLocalStageCommandEntry],
) -> Result<String> {
    let parent = script_path.parent().ok_or_else(|| anyhow!("stage script has no parent"))?;
    // Internal scripts move with their repository; external scripts retain the supplied root.
    let anchor = match parent.strip_prefix(root) {
        Ok(relative)
            if relative.components().all(|c| matches!(c, std::path::Component::Normal(_))) =>
        {
            let levels = relative.components().count();
            let suffix = if levels == 0 {
                ".".to_string()
            } else {
                std::iter::repeat_n("..", levels).collect::<Vec<_>>().join("/")
            };
            format!("repo_root=\"$(cd -- \"$(dirname -- \"${{BASH_SOURCE[0]}}\")/{suffix}\" && pwd)\"\n")
        }
        _ => format!(
            "repo_root={}\n",
            render_shell_command(&[root
                .to_str()
                .ok_or_else(|| anyhow!("stage script repository path is not UTF-8"))?
                .to_string()])
        ),
    };
    let mut script = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    script.push_str(&anchor);
    script.push_str("cd -- \"$repo_root\"\n\n");
    for command in commands {
        script.push_str("# ");
        script.push_str(&command.stage_id.replace(['\n', '\r'], " "));
        script.push('\n');
        script.push_str(&render_shell_command(&command.argv));
        script.push('\n');
    }
    Ok(script)
}

fn rendered_stage_commands_manifest_path(script_output_path: &Path) -> PathBuf {
    let json_name = script_output_path.file_name().and_then(|name| name.to_str()).map_or_else(
        || "rendered-stage-commands.json".to_string(),
        |name| {
            if let Some(stem) = name.strip_suffix(".sh") {
                format!("{stem}.json")
            } else {
                format!("{name}.json")
            }
        },
    );
    script_output_path.with_file_name(json_name)
}

fn rendered_stage_commands_argv_path(script_output_path: &Path) -> PathBuf {
    let argv_name = script_output_path.file_name().and_then(|name| name.to_str()).map_or_else(
        || "rendered-stage-commands.argv.jsonl".to_string(),
        |name| {
            if let Some(stem) = name.strip_suffix(".sh") {
                format!("{stem}.argv.jsonl")
            } else {
                format!("{name}.argv.jsonl")
            }
        },
    );
    script_output_path.with_file_name(argv_name)
}

fn render_stage_command_argv_jsonl(rows: &[BenchLocalStageCommandArgvEntry]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let line =
            serde_json::to_string(row).context("serialize local stage command argv row to json")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}
