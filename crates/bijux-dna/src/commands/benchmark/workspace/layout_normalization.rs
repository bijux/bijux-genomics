use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::BenchmarkWorkspaceConfig;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceEntrySummary {
    exists: bool,
    kind: String,
    file_count: u64,
    total_size_bytes: u64,
    mtime: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorkspaceConvergenceAction {
    pub(crate) action: String,
    pub(crate) entry_name: String,
    source: String,
    target: String,
    legacy_summary: WorkspaceEntrySummary,
    canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorkspaceConvergencePlan {
    canonical_root: String,
    legacy_root: String,
    pub(crate) actions: Vec<WorkspaceConvergenceAction>,
    removable_legacy_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    legacy_root_removed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkspaceStageNormalizationReport {
    stage_id: String,
    action: String,
    source: String,
    target: String,
    legacy_summary: WorkspaceEntrySummary,
    canonical_summary: WorkspaceEntrySummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorkspaceNormalizationReport {
    corpus_id: String,
    corpus_dir_name: String,
    canonical_corpus_root: String,
    legacy_corpus_root: String,
    archive_stage_ids: Vec<String>,
    cache_stage_ids: Vec<String>,
    shared_stage_ids: Vec<String>,
    archive_only_stage_ids: Vec<String>,
    cache_only_stage_ids: Vec<String>,
    mode: String,
    pub(crate) status: String,
    stage_reports: Vec<WorkspaceStageNormalizationReport>,
    pub(crate) moved_stage_ids: Vec<String>,
    pub(crate) removed_duplicate_stage_ids: Vec<String>,
    pub(crate) manual_review_stage_ids: Vec<String>,
}

pub(crate) fn normalize_workspace_layout_report(
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    corpus_dir_name: &str,
    confirm: bool,
) -> Result<WorkspaceNormalizationReport> {
    let local_results_root = workspace_local_results_root(workspace)?;
    let local_cache_mirror_root = workspace_local_cache_mirror_root(workspace)?;
    let legacy_corpus_root = local_results_root.join(corpus_dir_name);
    let canonical_corpus_root = local_cache_mirror_root.join("results").join(corpus_dir_name);

    let archive_stage_ids = stage_directory_names(&legacy_corpus_root);
    let cache_stage_ids = stage_directory_names(&canonical_corpus_root);
    let archive_set = archive_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let cache_set = cache_stage_ids.iter().cloned().collect::<BTreeSet<_>>();
    let shared_stage_ids = archive_set.intersection(&cache_set).cloned().collect::<Vec<_>>();
    let archive_only_stage_ids = archive_set.difference(&cache_set).cloned().collect::<Vec<_>>();
    let cache_only_stage_ids = cache_set.difference(&archive_set).cloned().collect::<Vec<_>>();

    let plan = plan_root_convergence(&canonical_corpus_root, &legacy_corpus_root)?;
    let convergence_report = if confirm { apply_root_convergence(&plan)? } else { plan };

    if confirm && legacy_corpus_root.exists() {
        remove_empty_parents(&legacy_corpus_root, &local_results_root)?;
    }

    let stage_reports = convergence_report
        .actions
        .iter()
        .map(|action| WorkspaceStageNormalizationReport {
            stage_id: action.entry_name.clone(),
            action: action.action.clone(),
            source: action.source.clone(),
            target: action.target.clone(),
            legacy_summary: action.legacy_summary.clone(),
            canonical_summary: action.canonical_summary.clone(),
            status: action.status.clone(),
        })
        .collect::<Vec<_>>();
    let moved_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "move-legacy-entry")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();
    let removed_duplicate_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "remove-legacy-duplicate")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();
    let manual_review_stage_ids = stage_reports
        .iter()
        .filter(|report| report.action == "manual-review-required")
        .map(|report| report.stage_id.clone())
        .collect::<Vec<_>>();

    Ok(WorkspaceNormalizationReport {
        corpus_id: corpus_id.to_string(),
        corpus_dir_name: corpus_dir_name.to_string(),
        canonical_corpus_root: canonical_corpus_root.display().to_string(),
        legacy_corpus_root: legacy_corpus_root.display().to_string(),
        archive_stage_ids,
        cache_stage_ids,
        shared_stage_ids,
        archive_only_stage_ids,
        cache_only_stage_ids,
        mode: if confirm { "confirm" } else { "dry-run" }.to_string(),
        status: if manual_review_stage_ids.is_empty() { "clear" } else { "needs-review" }
            .to_string(),
        stage_reports,
        moved_stage_ids,
        removed_duplicate_stage_ids,
        manual_review_stage_ids,
    })
}

fn workspace_local_results_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.results_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.results_root"))
}

fn workspace_local_cache_mirror_root(workspace: &BenchmarkWorkspaceConfig) -> Result<PathBuf> {
    workspace
        .local
        .as_ref()
        .and_then(|row| row.cache_mirror_root.as_deref())
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("workspace config is missing local.cache_mirror_root"))
}

fn stage_directory_names(root: &Path) -> Vec<String> {
    if !root.is_dir() {
        return Vec::new();
    }
    let mut entries = root
        .read_dir()
        .ok()
        .into_iter()
        .flat_map(|iter| iter.filter_map(Result::ok))
        .filter_map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => Some(entry.file_name().to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn remove_empty_parents(root: &Path, stop_at: &Path) -> Result<()> {
    let mut current = root.to_path_buf();
    while current != stop_at && current.exists() {
        match fs::remove_dir(&current) {
            Ok(()) => {
                current = current
                    .parent()
                    .ok_or_else(|| anyhow!("missing parent for {}", root.display()))?
                    .to_path_buf();
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => break,
            Err(err) => {
                if current.exists()
                    && fs::read_dir(&current)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false)
                {
                    break;
                }
                return Err(err).with_context(|| format!("remove dir {}", current.display()));
            }
        }
    }
    Ok(())
}

pub(crate) fn plan_root_convergence(
    canonical_root: &Path,
    legacy_root: &Path,
) -> Result<WorkspaceConvergencePlan> {
    let mut actions = Vec::new();
    if legacy_root.exists() {
        let mut legacy_entries = legacy_root
            .read_dir()
            .with_context(|| format!("read {}", legacy_root.display()))?
            .filter_map(|entry| entry.ok().map(|item| item.path()))
            .collect::<Vec<_>>();
        legacy_entries.sort();

        for legacy_entry in legacy_entries {
            let entry_name = legacy_entry
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| anyhow!("invalid legacy entry {}", legacy_entry.display()))?
                .to_string();
            let canonical_entry = canonical_root.join(&entry_name);
            let legacy_summary = workspace_entry_summary(&legacy_entry)?;
            let canonical_summary = workspace_entry_summary(&canonical_entry)?;
            let action = if !canonical_entry.exists() {
                "move-legacy-entry"
            } else if canonical_summary.file_count >= legacy_summary.file_count
                && canonical_summary.total_size_bytes >= legacy_summary.total_size_bytes
                && canonical_summary.mtime.unwrap_or(0) >= legacy_summary.mtime.unwrap_or(0)
            {
                "remove-legacy-duplicate"
            } else {
                "manual-review-required"
            };
            actions.push(WorkspaceConvergenceAction {
                action: action.to_string(),
                entry_name,
                source: legacy_entry.display().to_string(),
                target: canonical_entry.display().to_string(),
                legacy_summary,
                canonical_summary,
                status: None,
            });
        }
    }

    Ok(WorkspaceConvergencePlan {
        canonical_root: canonical_root.display().to_string(),
        legacy_root: legacy_root.display().to_string(),
        removable_legacy_root: actions
            .iter()
            .all(|action| action.action != "manual-review-required"),
        actions,
        legacy_root_removed: None,
    })
}

fn workspace_entry_summary(path: &Path) -> Result<WorkspaceEntrySummary> {
    if !path.exists() {
        return Ok(WorkspaceEntrySummary {
            exists: false,
            kind: "missing".to_string(),
            file_count: 0,
            total_size_bytes: 0,
            mtime: None,
        });
    }

    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    if metadata.is_file() {
        return Ok(WorkspaceEntrySummary {
            exists: true,
            kind: "file".to_string(),
            file_count: 1,
            total_size_bytes: metadata.len(),
            mtime: modified_secs(&metadata),
        });
    }

    let mut file_count = 0u64;
    let mut total_size_bytes = 0u64;
    let mut newest_mtime = 0u64;
    accumulate_workspace_entry_summary(
        path,
        &mut file_count,
        &mut total_size_bytes,
        &mut newest_mtime,
    )?;

    Ok(WorkspaceEntrySummary {
        exists: true,
        kind: "directory".to_string(),
        file_count,
        total_size_bytes,
        mtime: Some(newest_mtime),
    })
}

fn modified_secs(metadata: &fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn accumulate_workspace_entry_summary(
    path: &Path,
    file_count: &mut u64,
    total_size_bytes: &mut u64,
    newest_mtime: &mut u64,
) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry.with_context(|| format!("read {}", path.display()))?;
        let child_path = entry.path();
        let child_metadata =
            fs::metadata(&child_path).with_context(|| format!("stat {}", child_path.display()))?;
        if let Some(mtime) = modified_secs(&child_metadata) {
            *newest_mtime = (*newest_mtime).max(mtime);
        }
        if child_metadata.is_dir() {
            accumulate_workspace_entry_summary(
                &child_path,
                file_count,
                total_size_bytes,
                newest_mtime,
            )?;
            continue;
        }
        if child_metadata.is_file() {
            *file_count += 1;
            *total_size_bytes += child_metadata.len();
        }
    }
    Ok(())
}

fn apply_root_convergence(plan: &WorkspaceConvergencePlan) -> Result<WorkspaceConvergencePlan> {
    let mut applied_actions = Vec::new();
    for action in &plan.actions {
        let source = PathBuf::from(&action.source);
        let target = PathBuf::from(&action.target);
        let status = match action.action.as_str() {
            "move-legacy-entry" => {
                move_workspace_path(&source, &target)?;
                "applied"
            }
            "remove-legacy-duplicate" => {
                remove_workspace_path(&source)?;
                "applied"
            }
            "manual-review-required" => "pending-manual-review",
            other => return Err(anyhow!("unsupported convergence action `{other}`")),
        };
        applied_actions.push(WorkspaceConvergenceAction {
            status: Some(status.to_string()),
            ..action.clone()
        });
    }

    let legacy_root = PathBuf::from(&plan.legacy_root);
    let legacy_root_removed = if plan.removable_legacy_root && legacy_root.is_dir() {
        let mut entries =
            legacy_root.read_dir().with_context(|| format!("read {}", legacy_root.display()))?;
        if entries.next().is_none() {
            fs::remove_dir(&legacy_root)
                .with_context(|| format!("remove dir {}", legacy_root.display()))?;
            true
        } else {
            false
        }
    } else {
        false
    };

    Ok(WorkspaceConvergencePlan {
        actions: applied_actions,
        legacy_root_removed: Some(legacy_root_removed),
        ..plan.clone()
    })
}

fn move_workspace_path(source: &Path, target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::rename(source, target)
        .with_context(|| format!("move {} -> {}", source.display(), target.display()))
}

fn remove_workspace_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("remove dir {}", path.display()))
    } else {
        fs::remove_file(path).with_context(|| format!("remove file {}", path.display()))
    }
}
