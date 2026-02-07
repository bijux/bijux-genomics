use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use crate::commands::cli::AnalyzeReportArgs;

#[must_use]
pub(crate) fn normalize_fastq_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

#[must_use]
pub(crate) fn qc_class_label(stage: &str) -> Option<&'static str> {
    match bijux_api::v1::api::bench::qc_class_for_stage(stage) {
        Some(bijux_api::v1::api::bench::QcClass::Structural) => Some("structural"),
        Some(bijux_api::v1::api::bench::QcClass::Statistical) => Some("statistical"),
        None => None,
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
