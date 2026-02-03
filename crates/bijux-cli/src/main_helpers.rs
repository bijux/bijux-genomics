use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use bijux_api::load_profile;

use crate::cli::{AnalyzeReportArgs, Cli};
use bijux_api::normalize_run_base_dir;

pub(crate) fn render_report_bundle_html(report: &serde_json::Value) -> String {
    let pretty = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>bijux analyze report</title>
  <style>
    body {{
      font-family: system-ui, -apple-system, sans-serif;
      margin: 2rem;
      line-height: 1.4;
      background: #f7f7f9;
      color: #111;
    }}
    pre {{
      padding: 1rem;
      background: #fff;
      border-radius: 8px;
      overflow: auto;
      box-shadow: 0 1px 4px rgba(0,0,0,0.08);
    }}
  </style>
</head>
<body>
  <h1>bijux analyze report</h1>
  <pre>{pretty}</pre>
</body>
</html>"#
    )
}

pub(crate) fn normalize_fastq_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

pub(crate) fn resolve_report_inputs(args: &AnalyzeReportArgs) -> Result<(PathBuf, PathBuf)> {
    if let Some(facts_path) = args.facts_path.as_ref() {
        let base_dir = base_dir_from_facts(facts_path)?;
        return Ok((base_dir, facts_path.clone()));
    }

    if let Some(run_dir) = args.run_dir.as_ref() {
        let facts_path = facts_path_for_run_dir(run_dir)?;
        return Ok((run_dir.clone(), facts_path));
    }

    if let Some(sqlite_path) = args.sqlite.as_ref() {
        let run_dir = sqlite_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("sqlite path has no parent directory"))?;
        let facts_path = facts_path_for_run_dir(&run_dir)?;
        return Ok((run_dir, facts_path));
    }

    let run_id = args
        .run_id
        .as_ref()
        .ok_or_else(|| anyhow!("run_id is required when no run_dir or facts_path is provided"))?;
    let run_dir = args.search_root.join(run_id);
    let facts_path = facts_path_for_run_dir(&run_dir)?;
    Ok((run_dir, facts_path))
}

fn facts_path_for_run_dir(run_dir: &Path) -> Result<PathBuf> {
    let direct = run_dir.join("facts.jsonl");
    if direct.exists() {
        return Ok(direct);
    }
    let dashboard = run_dir.join("dashboard").join("facts.jsonl");
    if dashboard.exists() {
        return Ok(dashboard);
    }
    Err(anyhow!("facts.jsonl not found under {}", run_dir.display()))
}

fn base_dir_from_facts(facts_path: &Path) -> Result<PathBuf> {
    let Some(parent) = facts_path.parent() else {
        return Err(anyhow!("facts path has no parent directory"));
    };
    if parent.file_name().and_then(|name| name.to_str()) == Some("dashboard") {
        return parent
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("facts path dashboard has no parent"));
    }
    Ok(parent.to_path_buf())
}

pub(crate) fn load_profile_for_cli(cli: &Cli) -> Result<bijux_api::Profile> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let profile_path = cwd
        .join("configs")
        .join("profiles")
        .join(format!("{}.yaml", cli.profile));
    let mut profile = load_profile(&profile_path)
        .map_err(|err| anyhow!("failed to load profile {}: {err}", profile_path.display()))?;
    profile.run_base_dir = normalize_run_base_dir(&cwd, &profile.run_base_dir);
    Ok(profile)
}

pub(crate) fn ensure_profile_run_base_dir(
    stage: &bijux_api::StageId,
    tool: &bijux_api::ToolId,
    profile: &mut bijux_api::Profile,
) {
    let run_dir = bijux_api::run_dir(&profile.run_base_dir, &bijux_api::new_run_id(), stage, tool);
    if run_dir.starts_with(profile.run_base_dir.join("runs")) {
        let base = profile
            .run_base_dir
            .parent()
            .unwrap_or(&profile.run_base_dir);
        profile.run_base_dir = base.to_path_buf();
    }
}

pub(crate) fn qc_class_label(stage: &str) -> Option<&'static str> {
    match bijux_api::qc_class_for_stage(stage) {
        Some(bijux_api::QcClass::Structural) => Some("structural"),
        Some(bijux_api::QcClass::Statistical) => Some("statistical"),
        None => None,
    }
}
