//! Owner: bijux-dna-bench
//! Checksum-backed, reversible convergence of benchmark workspace roots.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkWorkspaceConfig, WorkspaceConvergenceAction, WorkspaceConvergencePlan,
    WorkspaceEntrySummary, WorkspaceNormalizationReport, WorkspaceStageNormalizationReport,
};

const MOVE_LEGACY_ENTRY: &str = "move-legacy-entry";
const QUARANTINE_LEGACY_DUPLICATE: &str = "quarantine-legacy-duplicate";
const MANUAL_REVIEW_REQUIRED: &str = "manual-review-required";

/// Plan or apply convergence of one configured benchmark corpus workspace.
///
/// Existing canonical and legacy entries are treated as duplicates only when their kinds and
/// content SHA-256 digests match. Confirmed duplicates are moved to deterministic same-filesystem
/// quarantine rather than deleted.
///
/// # Errors
/// Returns an error when required roots are absent, inventory hashing fails, or a requested apply
/// cannot be completed and verified atomically.
pub fn normalize_benchmark_workspace_layout_report(
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

    let plan = plan_benchmark_workspace_convergence(&canonical_corpus_root, &legacy_corpus_root)?;
    let convergence_plan =
        if confirm { apply_benchmark_workspace_convergence(&plan)? } else { plan };

    if confirm && legacy_corpus_root.exists() {
        remove_empty_parents(&legacy_corpus_root, &local_results_root)?;
    }

    let stage_reports = stage_reports(&convergence_plan);
    let moved_stage_ids = stage_ids_for_action(&stage_reports, MOVE_LEGACY_ENTRY);
    let quarantined_duplicate_stage_ids =
        stage_ids_for_action(&stage_reports, QUARANTINE_LEGACY_DUPLICATE);
    let manual_review_stage_ids = stage_ids_for_action(&stage_reports, MANUAL_REVIEW_REQUIRED);

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
        convergence_plan: Some(convergence_plan),
        stage_reports,
        moved_stage_ids,
        quarantined_duplicate_stage_ids,
        manual_review_stage_ids,
    })
}

/// Build a deterministic convergence plan for canonical and legacy benchmark roots.
///
/// # Errors
/// Returns an error when the legacy inventory cannot be read or content cannot be hashed.
pub fn plan_benchmark_workspace_convergence(
    canonical_root: &Path,
    legacy_root: &Path,
) -> Result<WorkspaceConvergencePlan> {
    let quarantine_root = benchmark_workspace_quarantine_root(legacy_root)?;
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
            let (action, target) = convergence_action(
                &entry_name,
                &canonical_entry,
                &quarantine_root,
                &legacy_summary,
                &canonical_summary,
            )?;
            actions.push(WorkspaceConvergenceAction {
                action: action.to_string(),
                entry_name,
                source: legacy_entry.display().to_string(),
                target: target.display().to_string(),
                legacy_summary,
                canonical_summary,
                status: None,
            });
        }
    }

    Ok(WorkspaceConvergencePlan {
        canonical_root: canonical_root.display().to_string(),
        legacy_root: legacy_root.display().to_string(),
        removable_legacy_root: actions.iter().all(|action| action.action != MANUAL_REVIEW_REQUIRED),
        actions,
        legacy_root_removed: None,
    })
}

/// Apply an immutable convergence plan using verified same-filesystem renames.
///
/// The operation is idempotent: an already-moved target is accepted only when its digest still
/// matches the planned source digest.
///
/// # Errors
/// Returns an error when paths conflict, bytes drift, or an atomic rename is unavailable.
pub fn apply_benchmark_workspace_convergence(
    plan: &WorkspaceConvergencePlan,
) -> Result<WorkspaceConvergencePlan> {
    let mut applied_actions = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        validate_convergence_action(plan, action)?;
        let status = match action.action.as_str() {
            MOVE_LEGACY_ENTRY | QUARANTINE_LEGACY_DUPLICATE => {
                apply_verified_move(action)?;
                "applied"
            }
            MANUAL_REVIEW_REQUIRED => "pending-manual-review",
            other => bail!("unsupported convergence action `{other}`"),
        };
        applied_actions.push(WorkspaceConvergenceAction {
            status: Some(status.to_string()),
            ..action.clone()
        });
    }

    let legacy_root = PathBuf::from(&plan.legacy_root);
    let legacy_root_removed = remove_empty_legacy_root(plan, &legacy_root)?;
    Ok(WorkspaceConvergencePlan {
        actions: applied_actions,
        legacy_root_removed: Some(legacy_root_removed),
        ..plan.clone()
    })
}

/// Roll back every applied move in reverse order after revalidating its checksum.
///
/// # Errors
/// Returns an error when an action is not in an applied or already-restored state, bytes drifted,
/// or an atomic same-filesystem rename cannot restore the source.
pub fn rollback_benchmark_workspace_convergence(
    plan: &WorkspaceConvergencePlan,
) -> Result<WorkspaceConvergencePlan> {
    let mut actions = plan.actions.clone();
    for action in actions.iter_mut().rev() {
        validate_convergence_action(plan, action)?;
        match action.action.as_str() {
            MOVE_LEGACY_ENTRY | QUARANTINE_LEGACY_DUPLICATE => {
                rollback_verified_move(action)?;
                action.status = Some("rolled-back".to_string());
            }
            MANUAL_REVIEW_REQUIRED => {
                action.status = Some("pending-manual-review".to_string());
            }
            other => bail!("unsupported convergence action `{other}`"),
        }
    }
    Ok(WorkspaceConvergencePlan { actions, legacy_root_removed: Some(false), ..plan.clone() })
}

fn validate_convergence_action(
    plan: &WorkspaceConvergencePlan,
    action: &WorkspaceConvergenceAction,
) -> Result<()> {
    if action.entry_name.is_empty()
        || Path::new(&action.entry_name).components().count() != 1
        || action.entry_name == "."
        || action.entry_name == ".."
    {
        bail!("invalid workspace convergence entry name `{}`", action.entry_name);
    }
    let canonical_root = Path::new(&plan.canonical_root);
    let legacy_root = Path::new(&plan.legacy_root);
    let expected_source = legacy_root.join(&action.entry_name);
    let expected_target = match action.action.as_str() {
        MOVE_LEGACY_ENTRY => {
            if action.canonical_summary.exists {
                bail!("move action unexpectedly has an existing canonical summary");
            }
            canonical_root.join(&action.entry_name)
        }
        QUARANTINE_LEGACY_DUPLICATE => {
            if action.legacy_summary.kind != action.canonical_summary.kind
                || action.legacy_summary.content_sha256.is_none()
                || action.legacy_summary.content_sha256 != action.canonical_summary.content_sha256
            {
                bail!("quarantine action does not describe byte-identical entries");
            }
            let digest = action
                .legacy_summary
                .content_sha256
                .as_deref()
                .context("quarantine action has no content digest")?;
            benchmark_workspace_quarantine_root(legacy_root)?.join(&action.entry_name).join(digest)
        }
        MANUAL_REVIEW_REQUIRED => canonical_root.join(&action.entry_name),
        other => bail!("unsupported convergence action `{other}`"),
    };
    if Path::new(&action.source) != expected_source {
        bail!("workspace convergence source does not match the governed legacy root");
    }
    if Path::new(&action.target) != expected_target {
        bail!("workspace convergence target does not match the governed action path");
    }
    if !action.legacy_summary.exists || action.legacy_summary.content_sha256.is_none() {
        bail!("workspace convergence action has no admitted legacy content identity");
    }
    Ok(())
}

/// Roll back the applied convergence plan retained by a normalization report.
///
/// # Errors
/// Returns an error when the report predates embedded convergence plans or when checksum-backed
/// rollback cannot restore every applied move.
pub fn rollback_benchmark_workspace_layout_report(
    report: &WorkspaceNormalizationReport,
) -> Result<WorkspaceNormalizationReport> {
    let plan = report
        .convergence_plan
        .as_ref()
        .context("workspace normalization report has no convergence plan to roll back")?;
    let rolled_back = rollback_benchmark_workspace_convergence(plan)?;
    let mut restored = report.clone();
    restored.mode = "rollback".to_string();
    restored.stage_reports = stage_reports(&rolled_back);
    restored.convergence_plan = Some(rolled_back);
    Ok(restored)
}

fn convergence_action(
    entry_name: &str,
    canonical_entry: &Path,
    quarantine_root: &Path,
    legacy: &WorkspaceEntrySummary,
    canonical: &WorkspaceEntrySummary,
) -> Result<(&'static str, PathBuf)> {
    if !canonical.exists {
        return Ok((MOVE_LEGACY_ENTRY, canonical_entry.to_path_buf()));
    }
    let identical = legacy.kind == canonical.kind
        && legacy.content_sha256.is_some()
        && legacy.content_sha256 == canonical.content_sha256;
    if identical {
        let digest = legacy
            .content_sha256
            .as_deref()
            .context("identical workspace entry is missing its content digest")?;
        return Ok((QUARANTINE_LEGACY_DUPLICATE, quarantine_root.join(entry_name).join(digest)));
    }
    Ok((MANUAL_REVIEW_REQUIRED, canonical_entry.to_path_buf()))
}

fn benchmark_workspace_quarantine_root(legacy_root: &Path) -> Result<PathBuf> {
    let parent = legacy_root
        .parent()
        .ok_or_else(|| anyhow!("legacy root has no parent: {}", legacy_root.display()))?;
    let corpus = legacy_root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow!("legacy root has no valid corpus name: {}", legacy_root.display())
        })?;
    Ok(parent.join(".quarantine/workspace-layout").join(corpus))
}

fn workspace_entry_summary(path: &Path) -> Result<WorkspaceEntrySummary> {
    if !path.exists() {
        return Ok(WorkspaceEntrySummary {
            exists: false,
            kind: "missing".to_string(),
            file_count: 0,
            total_size_bytes: 0,
            mtime: None,
            content_sha256: None,
        });
    }

    let metadata = fs::metadata(path).with_context(|| format!("stat {}", path.display()))?;
    let content_sha256 = Some(
        bijux_dna_infra::hash_path_sha256(path)
            .with_context(|| format!("hash {}", path.display()))?,
    );
    if metadata.is_file() {
        return Ok(WorkspaceEntrySummary {
            exists: true,
            kind: "file".to_string(),
            file_count: 1,
            total_size_bytes: metadata.len(),
            mtime: modified_secs(&metadata),
            content_sha256,
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
        content_sha256,
    })
}

fn apply_verified_move(action: &WorkspaceConvergenceAction) -> Result<()> {
    let source = Path::new(&action.source);
    let target = Path::new(&action.target);
    match (source.exists(), target.exists()) {
        (true, false) => {
            verify_path_digest(source, &action.legacy_summary)?;
            move_workspace_path(source, target)?;
            verify_path_digest(target, &action.legacy_summary)
        }
        (false, true) => verify_path_digest(target, &action.legacy_summary),
        (true, true) => bail!(
            "workspace convergence source and target both exist: {} -> {}",
            source.display(),
            target.display()
        ),
        (false, false) => bail!(
            "workspace convergence source and target are both missing: {} -> {}",
            source.display(),
            target.display()
        ),
    }
}

fn rollback_verified_move(action: &WorkspaceConvergenceAction) -> Result<()> {
    let source = Path::new(&action.source);
    let target = Path::new(&action.target);
    match (source.exists(), target.exists()) {
        (false, true) => {
            verify_path_digest(target, &action.legacy_summary)?;
            move_workspace_path(target, source)?;
            verify_path_digest(source, &action.legacy_summary)
        }
        (true, false) => verify_path_digest(source, &action.legacy_summary),
        (true, true) => bail!(
            "workspace rollback source and target both exist: {} <- {}",
            source.display(),
            target.display()
        ),
        (false, false) => bail!(
            "workspace rollback source and target are both missing: {} <- {}",
            source.display(),
            target.display()
        ),
    }
}

fn verify_path_digest(path: &Path, expected: &WorkspaceEntrySummary) -> Result<()> {
    let expected_digest = expected
        .content_sha256
        .as_deref()
        .context("workspace convergence action has no source digest")?;
    let actual = bijux_dna_infra::hash_path_sha256(path)
        .with_context(|| format!("hash {}", path.display()))?;
    if actual != expected_digest {
        bail!(
            "workspace content digest changed for {}: expected {}, observed {}",
            path.display(),
            expected_digest,
            actual
        );
    }
    Ok(())
}

fn move_workspace_path(source: &Path, target: &Path) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("workspace move target has no parent: {}", target.display()))?;
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))?;
    if !bijux_dna_infra::same_filesystem(source, parent).with_context(|| {
        format!("compare filesystems for {} and {}", source.display(), parent.display())
    })? {
        bail!(
            "workspace convergence requires a same-filesystem rename: {} -> {}",
            source.display(),
            target.display()
        );
    }
    bijux_dna_infra::rename(source, target)
        .with_context(|| format!("move {} -> {}", source.display(), target.display()))
}

fn remove_empty_legacy_root(plan: &WorkspaceConvergencePlan, legacy_root: &Path) -> Result<bool> {
    if !plan.removable_legacy_root || !legacy_root.is_dir() {
        return Ok(false);
    }
    let mut entries =
        legacy_root.read_dir().with_context(|| format!("read {}", legacy_root.display()))?;
    if entries.next().is_some() {
        return Ok(false);
    }
    fs::remove_dir(legacy_root)
        .with_context(|| format!("remove empty dir {}", legacy_root.display()))?;
    Ok(true)
}

fn stage_reports(plan: &WorkspaceConvergencePlan) -> Vec<WorkspaceStageNormalizationReport> {
    plan.actions
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
        .collect()
}

fn stage_ids_for_action(
    reports: &[WorkspaceStageNormalizationReport],
    action: &str,
) -> Vec<String> {
    reports
        .iter()
        .filter(|report| report.action == action)
        .map(|report| report.stage_id.clone())
        .collect()
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
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => {
                if current.exists()
                    && fs::read_dir(&current)
                        .map(|mut entries| entries.next().is_some())
                        .unwrap_or(false)
                {
                    break;
                }
                return Err(error).with_context(|| format!("remove dir {}", current.display()));
            }
        }
    }
    Ok(())
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
        } else if child_metadata.is_file() {
            *file_count += 1;
            *total_size_bytes += child_metadata.len();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        apply_benchmark_workspace_convergence, normalize_benchmark_workspace_layout_report,
        plan_benchmark_workspace_convergence, rollback_benchmark_workspace_convergence,
        rollback_benchmark_workspace_layout_report, MANUAL_REVIEW_REQUIRED, MOVE_LEGACY_ENTRY,
        QUARANTINE_LEGACY_DUPLICATE,
    };
    use bijux_dna_bench_model::{BenchmarkWorkspaceConfig, BenchmarkWorkspaceLocal};
    use std::path::Path;

    fn write(path: &Path, content: &str) -> anyhow::Result<()> {
        bijux_dna_infra::ensure_dir(path.parent().expect("fixture path parent"))?;
        bijux_dna_infra::write_bytes(path, content.as_bytes())?;
        Ok(())
    }

    #[test]
    fn differing_content_never_qualifies_as_a_duplicate() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical");
        let legacy = temp.path().join("legacy");
        write(&canonical.join("stage/result.txt"), "new")?;
        write(&legacy.join("stage/result.txt"), "old")?;

        let plan = plan_benchmark_workspace_convergence(&canonical, &legacy)?;
        assert_eq!(plan.actions[0].action, MANUAL_REVIEW_REQUIRED);
        assert_ne!(
            plan.actions[0].legacy_summary.content_sha256,
            plan.actions[0].canonical_summary.content_sha256
        );
        Ok(())
    }

    #[test]
    fn unique_and_identical_entries_apply_and_rollback_without_byte_loss() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("cache/results/corpus");
        let legacy = temp.path().join("archive/corpus");
        write(&canonical.join("shared/result.txt"), "same")?;
        write(&legacy.join("shared/result.txt"), "same")?;
        write(&legacy.join("unique/result.txt"), "unique")?;

        let plan = plan_benchmark_workspace_convergence(&canonical, &legacy)?;
        assert_eq!(plan.actions[0].action, QUARANTINE_LEGACY_DUPLICATE);
        assert_eq!(plan.actions[1].action, MOVE_LEGACY_ENTRY);
        let applied = apply_benchmark_workspace_convergence(&plan)?;
        assert!(!legacy.exists());
        assert!(Path::new(&applied.actions[0].target).is_dir());
        assert_eq!(std::fs::read_to_string(canonical.join("unique/result.txt"))?, "unique");

        let rolled_back = rollback_benchmark_workspace_convergence(&applied)?;
        assert_eq!(std::fs::read_to_string(legacy.join("shared/result.txt"))?, "same");
        assert_eq!(std::fs::read_to_string(legacy.join("unique/result.txt"))?, "unique");
        assert!(rolled_back
            .actions
            .iter()
            .all(|action| { action.status.as_deref() == Some("rolled-back") }));
        Ok(())
    }

    #[test]
    fn applied_report_is_a_complete_rollback_receipt() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let results = temp.path().join("archive");
        let cache = temp.path().join("cache");
        write(&results.join("corpus/shared/result.txt"), "same")?;
        write(&cache.join("results/corpus/shared/result.txt"), "same")?;
        let workspace = BenchmarkWorkspaceConfig {
            local: Some(BenchmarkWorkspaceLocal {
                results_root: Some(results.display().to_string()),
                cache_mirror_root: Some(cache.display().to_string()),
                ..BenchmarkWorkspaceLocal::default()
            }),
            ..BenchmarkWorkspaceConfig::default()
        };
        let applied =
            normalize_benchmark_workspace_layout_report(&workspace, "corpus", "corpus", true)?;

        let restored = rollback_benchmark_workspace_layout_report(&applied)?;
        assert_eq!(restored.mode, "rollback");
        assert_eq!(std::fs::read_to_string(results.join("corpus/shared/result.txt"))?, "same");
        assert_eq!(restored.stage_reports[0].status.as_deref(), Some("rolled-back"));
        Ok(())
    }

    #[test]
    fn apply_refuses_a_tampered_target_before_moving_bytes() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical");
        let legacy = temp.path().join("legacy");
        write(&legacy.join("stage/result.txt"), "evidence")?;
        let mut plan = plan_benchmark_workspace_convergence(&canonical, &legacy)?;
        plan.actions[0].target = temp.path().join("outside/stage").display().to_string();

        assert!(apply_benchmark_workspace_convergence(&plan).is_err());
        assert_eq!(std::fs::read_to_string(legacy.join("stage/result.txt"))?, "evidence");
        assert!(!temp.path().join("outside").exists());
        Ok(())
    }

    #[test]
    fn normalization_report_retains_the_reversible_plan() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let results = temp.path().join("archive");
        let cache = temp.path().join("cache");
        write(&results.join("corpus/shared/result.txt"), "same")?;
        write(&cache.join("results/corpus/shared/result.txt"), "same")?;
        let workspace = BenchmarkWorkspaceConfig {
            local: Some(BenchmarkWorkspaceLocal {
                results_root: Some(results.display().to_string()),
                cache_mirror_root: Some(cache.display().to_string()),
                ..BenchmarkWorkspaceLocal::default()
            }),
            ..BenchmarkWorkspaceConfig::default()
        };

        let report =
            normalize_benchmark_workspace_layout_report(&workspace, "corpus", "corpus", true)?;
        assert_eq!(report.quarantined_duplicate_stage_ids, vec!["shared"]);
        assert!(report.convergence_plan.is_some());
        assert_eq!(report.status, "clear");
        Ok(())
    }
}
