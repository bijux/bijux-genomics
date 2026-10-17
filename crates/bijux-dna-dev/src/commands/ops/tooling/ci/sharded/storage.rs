//! Source workspaces, disjoint output authorities and cleanup.

use crate::commands::ops::tooling::artifact_root_path;
use crate::commands::ops::tooling::ci::ci_output_path;
use crate::commands::ops::tooling::ci::sharded::contract::FullShard;
use crate::commands::ops::tooling::ci::sharded::contract::FullShardContract;
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::owned_artifacts::owned_children;
use crate::runtime::owned_artifacts::remove_empty_owned_directory;
use crate::runtime::owned_artifacts::remove_owned_trees;
use crate::runtime::source_snapshot::TrackedSourceSnapshot;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

pub(in super::super) const FULL_SHARD_SCRATCH_ROOT_ENV: &str = "BIJUX_CI_FULL_SHARD_SCRATCH_ROOT";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct FullShardRoots {
    pub(in super::super) artifact_root: PathBuf,
    pub(in super::super) cargo_target: PathBuf,
    pub(in super::super) test_target: PathBuf,
    pub(in super::super) coverage_target: PathBuf,
    pub(in super::super) test_tmp: PathBuf,
    pub(in super::super) coverage_tmp: PathBuf,
    pub(in super::super) test_profraw: PathBuf,
    pub(in super::super) coverage_profraw: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct FullArchiveRoots {
    pub(in super::super) artifact_root: PathBuf,
    pub(in super::super) archive_extract: PathBuf,
    pub(in super::super) cargo_target: PathBuf,
    pub(in super::super) archive_file: PathBuf,
    pub(in super::super) build_log: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct FullShardStorageRoots {
    pub(in super::super) retained_report_root: PathBuf,
    pub(in super::super) scratch_root: PathBuf,
}

pub(in super::super) struct FullShardEphemera {
    pub(in super::super) scratch_workspaces: PathBuf,
    pub(in super::super) archive_root: PathBuf,
    pub(in super::super) shard_artifact_roots: Vec<PathBuf>,
    pub(in super::super) armed: bool,
}

impl FullShardEphemera {
    pub(super) fn new(
        scratch_root: &Path,
        archive_roots: &FullArchiveRoots,
        shard_roots: &[FullShardRoots],
    ) -> Self {
        Self {
            scratch_workspaces: scratch_root.join("workspaces"),
            archive_root: archive_roots.artifact_root.clone(),
            shard_artifact_roots: shard_roots
                .iter()
                .map(|roots| roots.artifact_root.clone())
                .collect(),
            armed: true,
        }
    }

    pub(super) fn retain_bounded_evidence(&mut self) -> Result<()> {
        let mut paths = self.shard_artifact_roots.clone();
        paths.push(self.archive_root.clone());
        let children = owned_children(&self.scratch_workspaces);
        let mut failures = Vec::new();
        match children {
            Ok(children) => paths.extend(children),
            Err(error) => failures.push(format!("{error:#}")),
        }
        if let Err(error) = remove_owned_trees(&paths) {
            failures.push(format!("{error:#}"));
        }
        if let Err(error) = remove_empty_owned_directory(&self.scratch_workspaces) {
            failures.push(format!("{error:#}"));
        }
        anyhow::ensure!(failures.is_empty(), "full shard cleanup failed: {}", failures.join("; "));
        self.armed = false;
        Ok(())
    }
}

impl Drop for FullShardEphemera {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.retain_bounded_evidence();
        }
    }
}

pub(in super::super) fn observe_full_shard_cleanup(
    receipt: &mut serde_json::Value,
    ephemera: &mut FullShardEphemera,
) -> Result<()> {
    let cleanup_started = Instant::now();
    let cleanup = ephemera.retain_bounded_evidence();
    // A terminal receipt must describe the last attempt, without a hidden Drop retry.
    ephemera.armed = false;
    receipt["cleanup"] = serde_json::json!({
        "completed": cleanup.is_ok(),
        "duration_milliseconds": cleanup_started.elapsed().as_millis(),
        "error": cleanup.as_ref().err().map(|error| format!("{error:#}")),
    });
    receipt["evidence_retention"]["archive_retained"] =
        serde_json::json!(ephemera.archive_root.symlink_metadata().is_ok());
    let compilation_target = receipt["evidence_retention"]["compilation_cache_target"]
        .as_str()
        .map(PathBuf::from)
        .unwrap_or_else(|| ephemera.archive_root.join("build"));
    receipt["evidence_retention"]["build_targets_retained"] =
        serde_json::json!(compilation_target.exists());
    receipt["evidence_retention"]["isolated_workspaces_retained"] =
        serde_json::json!(ephemera.scratch_workspaces.symlink_metadata().is_ok());
    receipt["evidence_retention"]["shard_runtime_artifacts_retained"] = serde_json::json!(ephemera
        .shard_artifact_roots
        .iter()
        .any(|root| root.symlink_metadata().is_ok()));
    cleanup
}

#[cfg(test)]
pub(in super::super) fn finish_full_shard_receipt(
    path: &Path,
    receipt: serde_json::Value,
    started: Instant,
    ephemera: &mut FullShardEphemera,
) -> Result<()> {
    crate::runtime::gate_receipt::run_reported(
        path,
        receipt,
        started,
        ephemera,
        |_, _| Ok(()),
        |ephemera, receipt| observe_full_shard_cleanup(receipt, ephemera),
    )
}

pub(in super::super) fn materialize_full_shard_workspace(
    snapshot: &TrackedSourceSnapshot,
    report_root: &Path,
    identity: &str,
) -> Result<PathBuf> {
    anyhow::ensure!(
        Path::new(identity).components().count() == 1
            && matches!(Path::new(identity).components().next(), Some(Component::Normal(_))),
        "shard workspace identity must be one directory name"
    );
    let shard_workspace = report_root.join("workspaces").join(identity);
    if shard_workspace.symlink_metadata().is_ok() {
        std::fs::remove_dir_all(&shard_workspace).with_context(|| {
            format!("remove stale shard workspace {}", shard_workspace.display())
        })?;
    }
    let shard_parent = shard_workspace.parent().context("shard workspace has no parent")?;
    bijux_dna_infra::ensure_dir(shard_parent)?;
    snapshot.materialize(&shard_workspace)?;
    Ok(shard_workspace)
}

pub(in super::super) fn full_shard_roots(
    storage: &FullShardStorageRoots,
    contract: &FullShardContract,
    shard: &FullShard,
) -> Result<FullShardRoots> {
    let artifact_root = full_shard_scratch_path(
        &storage.scratch_root,
        &contract.report_root,
        &contract.shard_artifact_root,
    )?
    .join(&shard.id);
    Ok(FullShardRoots {
        cargo_target: isolated_build_tree(&artifact_root, "cargo"),
        test_target: isolated_build_tree(&artifact_root, "test"),
        coverage_target: isolated_build_tree(&artifact_root, "coverage"),
        test_tmp: artifact_root.join("tmp/test"),
        coverage_tmp: artifact_root.join("tmp/coverage"),
        test_profraw: artifact_root.join("coverage/profraw-test"),
        coverage_profraw: artifact_root.join("coverage/profraw-coverage"),
        artifact_root,
    })
}

pub(in super::super) fn full_archive_roots(
    storage: &FullShardStorageRoots,
    contract: &FullShardContract,
) -> Result<FullArchiveRoots> {
    let artifact_root = full_shard_scratch_path(
        &storage.scratch_root,
        &contract.report_root,
        &contract.archive_root,
    )?;
    Ok(FullArchiveRoots {
        archive_extract: artifact_root.join("extracted"),
        cargo_target: CompilationCache::target_under(&storage.scratch_root),
        archive_file: artifact_root.join("workspace-tests.tar.zst"),
        build_log: storage.retained_report_root.join("archive-build.log"),
        artifact_root,
    })
}

pub(in super::super) fn isolated_build_tree(artifact_root: &Path, consumer: &str) -> PathBuf {
    artifact_root.join("build").join(consumer)
}

pub(in super::super) fn resolve_full_shard_storage_roots(
    workspace: &Workspace,
    contract: &FullShardContract,
    configured_scratch_root: Option<PathBuf>,
) -> Result<FullShardStorageRoots> {
    let retained_report_root = ci_output_path(workspace, &contract.report_root)?;
    let scratch_root = configured_scratch_root.map_or_else(
        || artifact_root_path(workspace).map(|root| root.join("ci/full-shards-scratch")),
        Ok,
    )?;
    validate_absolute_normalized_path(&scratch_root, FULL_SHARD_SCRATCH_ROOT_ENV)?;
    if paths_overlap(&scratch_root, &retained_report_root) {
        anyhow::bail!(
            "full shard scratch root and retained report root overlap: {} and {}",
            scratch_root.display(),
            retained_report_root.display()
        );
    }
    Ok(FullShardStorageRoots { retained_report_root, scratch_root })
}

pub(in super::super) fn full_shard_scratch_path(
    scratch_root: &Path,
    configured_report_root: &str,
    configured_path: &str,
) -> Result<PathBuf> {
    let suffix = Path::new(configured_path)
        .strip_prefix(configured_report_root)
        .context("full shard scratch path is outside its declared report root")?;
    if suffix.as_os_str().is_empty() {
        anyhow::bail!("full shard scratch path must be a child of its declared report root");
    }
    Ok(scratch_root.join(suffix))
}

pub(in super::super) fn validate_absolute_normalized_path(
    path: &Path,
    authority: &str,
) -> Result<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        anyhow::bail!("{authority} must be absolute and normalized");
    }
    Ok(())
}

pub(in super::super) fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

pub(in super::super) fn validate_full_archive_roots(
    storage: &FullShardStorageRoots,
    roots: &FullArchiveRoots,
) -> Result<()> {
    if roots
        .scratch_all()
        .into_iter()
        .any(|root| !root.is_absolute() || !root.starts_with(&storage.scratch_root))
    {
        anyhow::bail!("full shard archive root is outside scratch authority");
    }
    if !roots.build_log.is_absolute() || !roots.build_log.starts_with(&storage.retained_report_root)
    {
        anyhow::bail!("full shard archive log is outside retained report authority");
    }
    if !roots.archive_file.starts_with(&roots.artifact_root)
        || roots.cargo_target != CompilationCache::target_under(&storage.scratch_root)
        || paths_overlap(&roots.cargo_target, &roots.artifact_root)
        || !roots.archive_extract.starts_with(&roots.artifact_root)
        || paths_overlap(&roots.archive_extract, &roots.cargo_target)
        || paths_overlap(&roots.archive_extract, &roots.archive_file)
    {
        anyhow::bail!("full shard archive paths escape the archive scratch root");
    }
    Ok(())
}

pub(in super::super) fn artifact_root_environment(root: &Path) -> Vec<(String, String)> {
    ["ARTIFACT_ROOT", "ISO_ROOT", "ARTIFACT_DIR"]
        .into_iter()
        .map(|key| (key.to_string(), root.display().to_string()))
        .collect()
}

pub(in super::super) fn validate_full_shard_root_set(
    storage: &FullShardStorageRoots,
    shard_roots: &[FullShardRoots],
) -> Result<()> {
    for roots in shard_roots {
        for root in roots.all() {
            if !root.is_absolute() || !root.starts_with(&storage.scratch_root) {
                anyhow::bail!(
                    "full shard output root is outside scratch authority: {}",
                    root.display()
                );
            }
        }
        let mut distinct = roots.all().into_iter().collect::<BTreeSet<_>>();
        distinct.remove(roots.artifact_root.as_path());
        if distinct.len() != roots.all().len() - 1 {
            anyhow::bail!("full shard runtime roots are not distinct");
        }
    }
    for (index, left) in shard_roots.iter().enumerate() {
        for right in shard_roots.iter().skip(index + 1) {
            if left
                .all()
                .into_iter()
                .any(|left_root| right.all().into_iter().any(|right_root| left_root == right_root))
            {
                anyhow::bail!("full shard output roots overlap");
            }
        }
    }
    Ok(())
}

pub(in super::super) fn full_shard_env(
    roots: &FullShardRoots,
    shard_workspace: &Path,
) -> Result<Vec<(String, String)>> {
    for root in roots.all() {
        bijux_dna_infra::ensure_dir(root)?;
    }
    let workspace = Workspace { root: shard_workspace.to_path_buf() };
    let outputs = workspace.admit_artifact_path(&workspace.path("artifacts"))?;
    bijux_dna_infra::ensure_dir(&outputs)?;
    let mut environment = vec![("NEXTEST_EXPERIMENTAL_LIBTEST_JSON".to_string(), "1".to_string())];
    environment.extend(artifact_root_environment(&outputs));
    environment.extend([
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("TEST_TARGET_DIR".to_string(), roots.test_target.display().to_string()),
        ("COV_TARGET_DIR".to_string(), roots.coverage_target.display().to_string()),
        ("TEST_TMP_DIR".to_string(), roots.test_tmp.display().to_string()),
        ("COV_TMP_DIR".to_string(), roots.coverage_tmp.display().to_string()),
        ("TEST_PROFRAW_DIR".to_string(), roots.test_profraw.display().to_string()),
        ("COV_PROFRAW_DIR".to_string(), roots.coverage_profraw.display().to_string()),
        ("BIJUX_REPO_ROOT".to_string(), shard_workspace.display().to_string()),
        (
            "BIJUX_BENCHMARK_ROOT".to_string(),
            shard_workspace.join("benchmarks").display().to_string(),
        ),
        // Archived snapshot assertions retain their compile-time source paths. Shards
        // therefore fail with a rendered diff but never write pending source files.
        ("INSTA_UPDATE".to_string(), "no".to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ]);
    Ok(environment)
}

pub(in super::super) fn full_shard_root_receipt(
    roots: &FullShardRoots,
    archive: &FullArchiveRoots,
    shard_workspace: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "repository_root": shard_workspace.display().to_string(),
        "benchmark_root": shard_workspace.join("benchmarks").display().to_string(),
        "snapshot_source_authority": "archive_compile_time_metadata",
        "snapshot_update_policy": "no",
        "artifact_root": shard_workspace.join("artifacts").display().to_string(),
        "iso_root": shard_workspace.join("artifacts").display().to_string(),
        "executor_scratch_root": roots.artifact_root.display().to_string(),
        "archive_extract_dir": archive.archive_extract.display().to_string(),
        "archive_extraction_ownership": "shared_read_only",
        "cargo_target_dir": roots.cargo_target.display().to_string(),
        "test_target_dir": roots.test_target.display().to_string(),
        "coverage_target_dir": roots.coverage_target.display().to_string(),
        "test_tmp_dir": roots.test_tmp.display().to_string(),
        "coverage_tmp_dir": roots.coverage_tmp.display().to_string(),
        "test_profraw_dir": roots.test_profraw.display().to_string(),
        "coverage_profraw_dir": roots.coverage_profraw.display().to_string(),
    })
}

pub(in super::super) fn full_shard_storage_receipt(
    storage: &FullShardStorageRoots,
) -> serde_json::Value {
    serde_json::json!({
        "retained_report_root": storage.retained_report_root.display().to_string(),
        "scratch_root": storage.scratch_root.display().to_string(),
        "scratch_root_environment": FULL_SHARD_SCRATCH_ROOT_ENV,
        "scratch_retained": true,
        "retention_scope": "exclusively_owned_compilation_cache",
        "compilation_cache_target": CompilationCache::target_under(&storage.scratch_root),
    })
}

impl FullShardRoots {
    fn all(&self) -> [&Path; 8] {
        [
            &self.artifact_root,
            &self.cargo_target,
            &self.test_target,
            &self.coverage_target,
            &self.test_tmp,
            &self.coverage_tmp,
            &self.test_profraw,
            &self.coverage_profraw,
        ]
    }
}

impl FullArchiveRoots {
    fn scratch_all(&self) -> [&Path; 4] {
        [&self.artifact_root, &self.cargo_target, &self.archive_file, &self.archive_extract]
    }
}

#[cfg(test)]
mod tests {
    use super::full_archive_roots;
    use super::full_shard_env;
    use super::full_shard_root_receipt;
    use super::full_shard_roots;
    use super::full_shard_storage_receipt;
    use super::isolated_build_tree;
    use super::materialize_full_shard_workspace;
    use super::resolve_full_shard_storage_roots;
    use super::validate_full_archive_roots;
    use super::validate_full_shard_root_set;
    use super::FullArchiveRoots;
    use super::FullShardEphemera;
    use super::FullShardRoots;
    use super::FULL_SHARD_SCRATCH_ROOT_ENV;
    use crate::commands::ops::tooling::artifact_root_path;
    use crate::commands::ops::tooling::ci::sharded::contract::contract;
    use crate::commands::ops::tooling::ci::sharded::contract::FullShard;
    use crate::runtime::owned_artifacts::remove_owned_tree as remove_full_shard_ephemera;
    use crate::runtime::workspace::Workspace;

    #[test]
    fn full_shard_runtime_roots_are_absolute_complete_disjoint_and_read_only() -> anyhow::Result<()>
    {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let storage = resolve_full_shard_storage_roots(&workspace, &contract, None)?;
        let roots = contract
            .shards
            .iter()
            .map(|shard| full_shard_roots(&storage, &contract, shard))
            .collect::<anyhow::Result<Vec<_>>>()?;
        validate_full_shard_root_set(&storage, &roots)?;

        let shard_workspace = storage.scratch_root.join("workspaces/hash-1-of-5");
        std::fs::create_dir_all(&shard_workspace)?;
        let first_env = full_shard_env(&roots[0], &shard_workspace)?
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        for name in [
            "ARTIFACT_ROOT",
            "ISO_ROOT",
            "ARTIFACT_DIR",
            "CARGO_TARGET_DIR",
            "TEST_TARGET_DIR",
            "COV_TARGET_DIR",
            "TEST_TMP_DIR",
            "COV_TMP_DIR",
            "TEST_PROFRAW_DIR",
            "COV_PROFRAW_DIR",
        ] {
            let value = first_env
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("missing shard environment variable {name}"))?;
            let path = std::path::Path::new(value);
            assert!(path.is_absolute());
            assert!(path.starts_with(&storage.scratch_root));
        }
        assert_ne!(first_env["CARGO_TARGET_DIR"], first_env["TEST_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TARGET_DIR"], first_env["COV_TARGET_DIR"]);
        assert_ne!(first_env["TEST_TMP_DIR"], first_env["COV_TMP_DIR"]);
        assert_ne!(first_env["TEST_PROFRAW_DIR"], first_env["COV_PROFRAW_DIR"]);
        assert_eq!(first_env["BIJUX_REPO_ROOT"], shard_workspace.display().to_string());
        assert_eq!(
            first_env["BIJUX_BENCHMARK_ROOT"],
            shard_workspace.join("benchmarks").display().to_string()
        );
        let consumer = Workspace { root: shard_workspace.clone() };
        let outputs = std::path::Path::new(&first_env["ARTIFACT_ROOT"]);
        assert_eq!(consumer.admit_artifact_path(outputs)?, consumer.path("artifacts"));
        assert_eq!(first_env["ISO_ROOT"], first_env["ARTIFACT_ROOT"]);
        assert_eq!(first_env["ARTIFACT_DIR"], first_env["ARTIFACT_ROOT"]);
        assert!(consumer.admit_artifact_path(&roots[0].artifact_root).is_err());
        assert_eq!(first_env["INSTA_UPDATE"], "no");
        assert!(!first_env.contains_key("INSTA_WORKSPACE_ROOT"));
        let archive = full_archive_roots(&storage, &contract)?;
        let receipt = full_shard_root_receipt(&roots[0], &archive, &shard_workspace);
        assert_eq!(receipt["repository_root"], first_env["BIJUX_REPO_ROOT"]);
        assert_eq!(receipt["benchmark_root"], first_env["BIJUX_BENCHMARK_ROOT"]);
        assert_eq!(receipt["snapshot_source_authority"], "archive_compile_time_metadata");
        assert_eq!(receipt["snapshot_update_policy"], first_env["INSTA_UPDATE"]);
        assert_eq!(receipt["artifact_root"], first_env["ARTIFACT_ROOT"]);
        assert_eq!(receipt["iso_root"], first_env["ISO_ROOT"]);
        assert_eq!(receipt["archive_extract_dir"], archive.archive_extract.display().to_string());
        assert_eq!(receipt["cargo_target_dir"], first_env["CARGO_TARGET_DIR"]);
        assert_eq!(receipt["test_target_dir"], first_env["TEST_TARGET_DIR"]);
        assert_eq!(receipt["coverage_target_dir"], first_env["COV_TARGET_DIR"]);
        assert_eq!(receipt["test_tmp_dir"], first_env["TEST_TMP_DIR"]);
        assert_eq!(receipt["coverage_tmp_dir"], first_env["COV_TMP_DIR"]);
        assert_eq!(receipt["test_profraw_dir"], first_env["TEST_PROFRAW_DIR"]);
        assert_eq!(receipt["coverage_profraw_dir"], first_env["COV_PROFRAW_DIR"]);
        let storage_receipt = full_shard_storage_receipt(&storage);
        assert_eq!(storage_receipt["scratch_root"], storage.scratch_root.display().to_string());
        assert_eq!(
            storage_receipt["retained_report_root"],
            storage.retained_report_root.display().to_string()
        );
        assert_eq!(storage_receipt["scratch_root_environment"], FULL_SHARD_SCRATCH_ROOT_ENV);
        Ok(())
    }

    #[test]
    fn full_shard_scratch_authority_is_external_disjoint_and_fail_closed() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let external = std::path::PathBuf::from("/local/slurmtmp.1234/bijux-full-shards");
        let storage =
            resolve_full_shard_storage_roots(&workspace, &contract, Some(external.clone()))?;

        assert_eq!(storage.scratch_root, external);
        assert!(storage.retained_report_root.starts_with(artifact_root_path(&workspace)?));
        assert!(!storage.scratch_root.starts_with(&storage.retained_report_root));
        assert!(!storage.retained_report_root.starts_with(&storage.scratch_root));

        let archive = full_archive_roots(&storage, &contract)?;
        validate_full_archive_roots(&storage, &archive)?;
        assert!(archive.artifact_root.starts_with(&storage.scratch_root));
        assert!(archive.build_log.starts_with(&storage.retained_report_root));

        assert!(resolve_full_shard_storage_roots(
            &workspace,
            &contract,
            Some(std::path::PathBuf::from("relative/scratch")),
        )
        .is_err());
        assert!(resolve_full_shard_storage_roots(
            &workspace,
            &contract,
            Some(storage.retained_report_root.join("scratch")),
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn full_shards_retain_only_bounded_logs_and_receipts() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let report_root = root.path().join("retained/full-shards");
        let scratch_root = root.path().join("scratch/full-shards");
        let archive_root = scratch_root.join("archive");
        let shard_root = scratch_root.join("shards/hash-1-of-5");
        std::fs::create_dir_all(scratch_root.join("workspaces/hash-1-of-5"))?;
        std::fs::create_dir_all(isolated_build_tree(&archive_root, "cargo"))?;
        std::fs::create_dir_all(&shard_root)?;
        std::fs::create_dir_all(&report_root)?;
        std::fs::write(report_root.join("hash-1-of-5.log"), b"test failure\n")?;
        std::fs::write(report_root.join("full-gate-timing.json"), b"{}\n")?;
        std::fs::write(report_root.join("archive-build.log"), b"archive build\n")?;
        std::fs::write(archive_root.join("workspace-tests.tar.zst"), b"archive\n")?;
        std::fs::write(
            isolated_build_tree(&archive_root, "cargo").join("output"),
            b"cargo output\n",
        )?;
        std::fs::write(shard_root.join("output"), b"shard output\n")?;
        let extracted = shard_root.join("nextest/archive/read-only");
        std::fs::create_dir_all(&extracted)?;
        let extracted_file = extracted.join("test-binary");
        std::fs::write(&extracted_file, b"nextest archive member\n")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            std::fs::set_permissions(&extracted_file, std::fs::Permissions::from_mode(0o400))?;
            std::fs::set_permissions(&extracted, std::fs::Permissions::from_mode(0o500))?;
            std::fs::set_permissions(
                shard_root.join("nextest/archive"),
                std::fs::Permissions::from_mode(0o500),
            )?;
            std::fs::set_permissions(
                shard_root.join("nextest"),
                std::fs::Permissions::from_mode(0o500),
            )?;
            std::fs::set_permissions(&shard_root, std::fs::Permissions::from_mode(0o500))?;
        }

        let archive_roots = FullArchiveRoots {
            artifact_root: archive_root.clone(),
            archive_extract: archive_root.join("extracted"),
            cargo_target: isolated_build_tree(&archive_root, "cargo"),
            archive_file: archive_root.join("workspace-tests.tar.zst"),
            build_log: report_root.join("archive-build.log"),
        };
        let shard_roots = vec![FullShardRoots {
            artifact_root: shard_root.clone(),
            cargo_target: isolated_build_tree(&shard_root, "cargo"),
            test_target: isolated_build_tree(&shard_root, "test"),
            coverage_target: isolated_build_tree(&shard_root, "coverage"),
            test_tmp: shard_root.join("tmp/test"),
            coverage_tmp: shard_root.join("tmp/coverage"),
            test_profraw: shard_root.join("coverage/profraw-test"),
            coverage_profraw: shard_root.join("coverage/profraw-coverage"),
        }];
        let mut ephemera = FullShardEphemera::new(&scratch_root, &archive_roots, &shard_roots);
        ephemera.retain_bounded_evidence()?;

        assert!(!scratch_root.join("workspaces").exists());
        assert!(!archive_root.exists());
        assert!(!shard_root.exists());
        assert!(report_root.join("archive-build.log").is_file());
        assert!(report_root.join("hash-1-of-5.log").is_file());
        assert!(report_root.join("full-gate-timing.json").is_file());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_ephemera_removes_nested_symlinks_without_following() -> anyhow::Result<()> {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir()?;
        let ephemeral = root.path().join("ephemeral");
        let retained = root.path().join("retained");
        std::fs::create_dir_all(&ephemeral)?;
        std::fs::write(&retained, b"retained evidence\n")?;
        symlink(&retained, ephemeral.join("escape"))?;

        remove_full_shard_ephemera(&ephemeral)?;

        assert_eq!(std::fs::read(&retained)?, b"retained evidence\n");
        assert!(!ephemeral.exists());
        Ok(())
    }

    #[test]
    fn full_shard_workspace_is_an_isolated_tracked_source_clone() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let artifact_root = workspace.path("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let report_root = tempfile::tempdir_in(&artifact_root)?;
        let shard =
            FullShard { id: "isolation-proof".to_string(), partition: "hash:1/5".to_string() };

        let snapshot = crate::runtime::source_snapshot::TrackedSourceSnapshot::capture(&workspace)?;
        let isolated = materialize_full_shard_workspace(&snapshot, report_root.path(), &shard.id)?;
        assert!(isolated.join(".git").is_dir());
        assert!(isolated.join("artifacts").is_dir());
        let root_manifest = std::fs::read(workspace.path("Cargo.toml"))?;
        assert_eq!(std::fs::read(isolated.join("Cargo.toml"))?, root_manifest);
        std::fs::write(isolated.join("Cargo.toml"), b"isolated mutation\n")?;
        assert_eq!(std::fs::read(workspace.path("Cargo.toml"))?, root_manifest);
        Ok(())
    }

    #[test]
    fn full_shard_receipt_includes_owned_cleanup_in_terminal_timing() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let archive = root.path().join("archive");
        std::fs::create_dir_all(archive.join("build"))?;
        std::fs::write(archive.join("build/product"), b"owned compilation output")?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: archive.clone(),
            shard_artifact_roots: vec![],
            armed: true,
        };
        let started = std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(25))
            .ok_or_else(|| anyhow::anyhow!("cannot construct elapsed timer"))?;
        let path = root.path().join("timing.json");
        super::finish_full_shard_receipt(
            &path,
            serde_json::json!({"state":"complete","all_passed":true,"wall_time_milliseconds":0}),
            started,
            &mut ephemera,
        )?;
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert!(!archive.exists());
        assert!(!ephemera.armed);
        assert_eq!(receipt["all_passed"], true);
        assert_eq!(receipt["cleanup"]["completed"], true);
        assert_eq!(receipt["evidence_retention"]["build_targets_retained"], false);
        assert!(receipt["wall_time_milliseconds"].as_u64().is_some_and(|elapsed| elapsed >= 25));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_receipt_records_cleanup_failure_and_preserves_external_targets(
    ) -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let external = root.path().join("retained");
        std::fs::create_dir_all(&external)?;
        std::fs::write(external.join("evidence"), b"retained")?;
        let archive = root.path().join("archive");
        std::os::unix::fs::symlink(&external, &archive)?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: archive,
            shard_artifact_roots: vec![],
            armed: true,
        };
        let path = root.path().join("timing.json");
        assert!(super::finish_full_shard_receipt(
            &path,
            serde_json::json!({"state":"complete","all_passed":true}),
            std::time::Instant::now(),
            &mut ephemera
        )
        .is_err());
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(receipt["all_passed"], false);
        assert_eq!(receipt["cleanup"]["completed"], false);
        assert_eq!(receipt["evidence_retention"]["archive_retained"], true);
        assert!(!ephemera.armed);
        assert_eq!(std::fs::read(external.join("evidence"))?, b"retained");
        Ok(())
    }

    #[test]
    fn full_shard_cleanup_retains_exclusively_owned_compilation_products() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let lease = super::CompilationCache::acquire(root.path())?;
        let product = lease.target().join("compiled-product");
        std::fs::write(&product, b"reusable")?;
        let mut ephemera = FullShardEphemera {
            scratch_workspaces: root.path().join("workspaces"),
            archive_root: root.path().join("archive"),
            shard_artifact_roots: vec![root.path().join("shards/hash-1-of-5")],
            armed: true,
        };
        std::fs::create_dir_all(&ephemera.archive_root)?;
        let report = root.path().join("timing.json");
        super::finish_full_shard_receipt(
            &report,
            serde_json::json!({"all_passed":true,"evidence_retention":{"compilation_cache_target":lease.target()}}),
            std::time::Instant::now(),
            &mut ephemera,
        )?;
        assert!(!ephemera.archive_root.exists());
        assert_eq!(std::fs::read(product)?, b"reusable");
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report)?)?;
        assert_eq!(receipt["evidence_retention"]["build_targets_retained"], true);
        assert_eq!(receipt["all_passed"], true);
        Ok(())
    }
}
