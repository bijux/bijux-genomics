//! Archive production/reuse and readonly executable custody.

use crate::application::ci::admit_compiled_executable_paths;
use crate::application::ci::NextestReuseInputs;
use crate::commands::ops::tooling::ci::sharded::contract::FullShard;
use crate::commands::ops::tooling::ci::sharded::storage::artifact_root_environment;
use crate::commands::ops::tooling::ci::sharded::storage::FullArchiveRoots;
use crate::runtime::compiled_products::ExecutableSeal;
use crate::runtime::owned_artifacts::remove_owned_tree as remove_full_shard_ephemera;
use anyhow::Context;
use anyhow::Result;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[cfg(test)]
use crate::commands::ops::tooling::ci::sharded::storage::FullShardStorageRoots;

pub(in super::super) struct ArchivedExecutableSeal {
    pub(in super::super) extracted: ExecutableSeal,
    pub(in super::super) producer: ExecutableSeal,
}

pub(in super::super) fn full_archive_args(archive_file: &Path) -> Vec<String> {
    vec![
        "nextest".to_string(),
        "archive".to_string(),
        "--workspace".to_string(),
        "--all-targets".to_string(),
        "--all-features".to_string(),
        "--locked".to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--archive-file".to_string(),
        archive_file.display().to_string(),
    ]
}

pub(in super::super) fn full_inventory_args(archive: &FullArchiveRoots) -> Vec<String> {
    vec![
        "nextest".to_string(),
        "list".to_string(),
        "--archive-file".to_string(),
        archive.archive_file.display().to_string(),
        "--extract-to".to_string(),
        archive.archive_extract.display().to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--message-format".to_string(),
        "json".to_string(),
    ]
}

pub(in super::super) fn full_archive_extraction_args(archive: &FullArchiveRoots) -> Vec<String> {
    let mut args = full_inventory_args(archive);
    args.extend(["--list-type".to_string(), "binaries-only".to_string()]);
    args
}

pub(in super::super) fn full_shard_args(
    shard: &FullShard,
    reuse: &NextestReuseInputs,
    workspace_remap: &Path,
    test_threads: usize,
) -> Result<Vec<String>> {
    anyhow::ensure!(test_threads > 0, "full-shard case threads must be positive");
    let mut args = vec![
        "nextest".to_string(),
        "run".to_string(),
        "--message-format".to_string(),
        "libtest-json-plus".to_string(),
        "--message-format-version".to_string(),
        "0.1".to_string(),
        "--no-tests".to_string(),
        "fail".to_string(),
        "--config-file".to_string(),
        "configs/rust/nextest.toml".to_string(),
        "--profile".to_string(),
        "full".to_string(),
        "--run-ignored".to_string(),
        "all".to_string(),
        "--retries".to_string(),
        "0".to_string(),
        "--partition".to_string(),
        shard.partition.clone(),
        "--test-threads".to_string(),
        test_threads.to_string(),
    ];
    args.extend(reuse.arguments(workspace_remap)?);
    Ok(args)
}

pub(in super::super) fn archive_reuse_inputs(archive_extract: &Path) -> Result<NextestReuseInputs> {
    let target = archive_extract.join("target");
    NextestReuseInputs::new(
        target.join("nextest/cargo-metadata.json"),
        target.join("nextest/binaries-metadata.json"),
        target,
    )
}

pub(in super::super) fn capture_archive_executable_seal(
    archive: &FullArchiveRoots,
    source: &Path,
) -> Result<ArchivedExecutableSeal> {
    let extracted_target = archive.archive_extract.join("target");
    let cargo_metadata = extracted_target.join("nextest/cargo-metadata.json");
    let binary_metadata = extracted_target.join("nextest/binaries-metadata.json");
    let paths = admit_compiled_executable_paths(
        &fs::read(&cargo_metadata)?,
        &fs::read(&binary_metadata)?,
        source,
        &archive.cargo_target,
    )?;
    let producer = ExecutableSeal::capture(&paths)?;
    let original = producer.digest_manifest();
    let mut extracted = paths
        .iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&archive.cargo_target)
                .context("archived executable escaped its producer target")?;
            let extracted = extracted_target.join(relative);
            anyhow::ensure!(
                bijux_dna_infra::hash_file_sha256(&extracted)? == original[path],
                "extracted executable differs from its source-bound producer: {}",
                extracted.display()
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let producer = path.metadata()?;
                let consumer = extracted.metadata()?;
                anyhow::ensure!(
                    producer.dev() != consumer.dev() || producer.ino() != consumer.ino(),
                    "extracted executable aliases its mutable compilation product: {}",
                    extracted.display()
                );
            }
            Ok(extracted)
        })
        .collect::<Result<BTreeSet<_>>>()?;
    extracted.extend([cargo_metadata, binary_metadata]);
    Ok(ArchivedExecutableSeal { extracted: ExecutableSeal::capture(&extracted)?, producer })
}

pub(in super::super) fn prepare_full_archive_roots(roots: &FullArchiveRoots) -> Result<()> {
    bijux_dna_infra::ensure_dir(&roots.artifact_root)?;
    bijux_dna_infra::ensure_dir(&roots.cargo_target)?;
    for stale in [&roots.archive_file, &roots.build_log] {
        if stale.exists() {
            std::fs::remove_file(stale)
                .with_context(|| format!("remove stale full shard artifact {}", stale.display()))?;
        }
    }
    Ok(())
}

pub(in super::super) fn set_full_archive_readonly(path: &Path) -> Result<()> {
    let mut permissions = std::fs::metadata(path)
        .with_context(|| format!("read full shard archive metadata {}", path.display()))?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)
        .with_context(|| format!("make full shard archive read-only {}", path.display()))
}

pub(in super::super) fn prepare_full_archive_extraction(path: &Path) -> Result<()> {
    remove_full_shard_ephemera(path)?;
    bijux_dna_infra::ensure_dir(path)?;
    Ok(())
}

pub(in super::super) fn seal_full_archive_extraction(path: &Path) -> Result<()> {
    let metadata = path
        .symlink_metadata()
        .with_context(|| format!("read extracted archive root {}", path.display()))?;
    anyhow::ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "full archive extraction must be an owned directory: {}",
        path.display()
    );
    for entry in walkdir::WalkDir::new(path).follow_links(false).contents_first(true) {
        let entry =
            entry.with_context(|| format!("inspect extracted archive {}", path.display()))?;
        if !entry.file_type().is_symlink() {
            set_full_archive_readonly(entry.path())?;
        }
    }
    Ok(())
}

pub(in super::super) fn full_archive_env(
    roots: &FullArchiveRoots,
    build_jobs: usize,
) -> Result<Vec<(String, String)>> {
    bijux_dna_infra::ensure_dir(&roots.cargo_target)?;
    let mut environment = artifact_root_environment(&roots.artifact_root);
    environment.extend([
        ("CARGO_TARGET_DIR".to_string(), roots.cargo_target.display().to_string()),
        ("CARGO_BUILD_JOBS".to_string(), build_jobs.to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ]);
    Ok(environment)
}

#[cfg(test)]
mod tests {
    use super::full_archive_args;
    use super::full_shard_args;
    use super::set_full_archive_readonly;
    use crate::commands::ops::tooling::ci::sharded::contract::contract;
    use crate::commands::ops::tooling::ci::sharded::storage::full_archive_roots;
    use crate::commands::ops::tooling::ci::sharded::storage::resolve_full_shard_storage_roots;
    use crate::commands::ops::tooling::ci::sharded::storage::validate_full_archive_roots;
    use crate::commands::ops::tooling::ci::sharded::storage::FullArchiveRoots;
    use crate::runtime::workspace::Workspace;
    use anyhow::Context;

    #[test]
    fn full_shards_build_once_and_execute_only_from_the_archive() -> anyhow::Result<()> {
        let workspace = Workspace::resolve()?;
        let contract = contract();
        let storage = resolve_full_shard_storage_roots(&workspace, &contract, None)?;
        let archive = full_archive_roots(&storage, &contract)?;
        validate_full_archive_roots(&storage, &archive)?;

        let build_args = full_archive_args(&archive.archive_file);
        for required in ["archive", "--workspace", "--all-targets", "--all-features", "--locked"] {
            assert!(build_args.iter().any(|arg| arg == required));
        }
        assert_eq!(build_args.iter().filter(|arg| arg.as_str() == "--archive-file").count(), 1);

        let inventory = super::full_inventory_args(&archive);
        assert!(inventory.windows(2).any(|pair| pair[0] == "--extract-to"
            && pair[1] == archive.archive_extract.display().to_string()));
        assert!(!inventory.iter().any(|arg| arg == "--workspace" || arg == "--all-targets"));
        let extraction = super::full_archive_extraction_args(&archive);
        assert!(extraction
            .windows(2)
            .any(|pair| pair[0] == "--list-type" && pair[1] == "binaries-only"));
        let native_inventory = crate::runtime::compiled_products::inventory_arguments(
            &super::archive_reuse_inputs(&archive.archive_extract)?,
            &storage.scratch_root.join("producer"),
        )?;
        assert!(native_inventory.iter().any(|arg| arg == "--cargo-metadata"));
        assert!(native_inventory
            .windows(2)
            .any(|pair| pair[0] == "--run-ignored" && pair[1] == "all"));
        assert!(!native_inventory.iter().any(|arg| [
            "--archive-file",
            "--extract-to",
            "--workspace",
            "--all-targets"
        ]
        .contains(&arg.as_str())));

        for shard in &contract.shards {
            let remap = storage.scratch_root.join("workspaces").join(&shard.id);
            let args = full_shard_args(
                shard,
                &super::archive_reuse_inputs(&archive.archive_extract)?,
                &remap,
                3,
            )?;
            assert!(args.windows(2).any(|pair| pair[0] == "--test-threads" && pair[1] == "3"));
            for forbidden in [
                "--workspace",
                "--all-targets",
                "--all-features",
                "--locked",
                "--archive-file",
                "--extract-to",
            ] {
                assert!(!args.iter().any(|arg| arg == forbidden));
            }
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--cargo-metadata"
                    && pair[1]
                        == archive
                            .archive_extract
                            .join("target/nextest/cargo-metadata.json")
                            .display()
                            .to_string()
            }));
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--target-dir-remap"
                    && pair[1] == archive.archive_extract.join("target").display().to_string()
            }));
            assert!(args.windows(2).any(|pair| {
                pair[0] == "--workspace-remap" && pair[1] == remap.display().to_string()
            }));
        }
        Ok(())
    }

    #[test]
    fn shared_archive_preparation_creates_an_empty_extraction_destination() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("archive/extracted");
        super::prepare_full_archive_extraction(&archive)?;
        assert!(archive.is_dir());
        std::fs::write(archive.join("stale"), b"old generated artifact")?;
        super::seal_full_archive_extraction(&archive)?;
        super::prepare_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::read_dir(&archive)?.count(), 0);
        Ok(())
    }

    #[test]
    fn shared_archive_sealing_preserves_bytes_and_owned_cleanup() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let archive = temp.path().join("extracted");
        let member = archive.join("target/nextest/binaries-metadata.json");
        std::fs::create_dir_all(member.parent().context("member parent")?)?;
        std::fs::write(&member, b"governed archive metadata")?;
        super::seal_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::read(&member)?, b"governed archive metadata");
        assert!(std::fs::metadata(&member)?.permissions().readonly());
        assert!(std::fs::metadata(&archive)?.permissions().readonly());
        super::remove_full_shard_ephemera(&archive)?;
        assert!(!archive.exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn archive_executable_seal_rejects_changed_materialization_with_unchanged_archive(
    ) -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir()?;
        let root = temp.path().canonicalize()?;
        let archive = FullArchiveRoots {
            artifact_root: root.join("archive"),
            archive_extract: root.join("extracted"),
            cargo_target: root.join("producer-target"),
            archive_file: root.join("archive.tar.zst"),
            build_log: root.join("build.log"),
        };
        let source = root.join("producer-source");
        let target = archive.archive_extract.join("target");
        let metadata = target.join("nextest");
        let binary = target.join("debug/deps/owner");
        std::fs::create_dir_all(&metadata)?;
        std::fs::create_dir_all(binary.parent().context("binary parent")?)?;
        std::fs::write(&binary, b"original executable")?;
        let producer_binary = archive.cargo_target.join("debug/deps/owner");
        std::fs::create_dir_all(producer_binary.parent().context("producer binary parent")?)?;
        std::fs::write(&producer_binary, b"original executable")?;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        std::fs::write(
            metadata.join("cargo-metadata.json"),
            serde_json::to_vec(&serde_json::json!({
                "workspace_root": source, "target_directory": archive.cargo_target,
            }))?,
        )?;
        std::fs::write(
            metadata.join("binaries-metadata.json"),
            serde_json::to_vec(&serde_json::json!({
                "rust-build-meta": {"target-directory": archive.cargo_target, "non-test-binaries": {}},
                "rust-binaries": {"owner": {"binary-id": "owner", "binary-path": archive.cargo_target.join("debug/deps/owner")}},
            }))?,
        )?;
        std::fs::write(&archive.archive_file, b"unchanged source-bound archive")?;
        let archive_sha256 = bijux_dna_infra::hash_file_sha256(&archive.archive_file)?;
        super::seal_full_archive_extraction(&archive.archive_extract)?;
        let seal = super::capture_archive_executable_seal(&archive, &source)?;
        seal.extracted.require_readonly()?;
        assert!(seal.extracted.verify().unchanged);
        assert!(seal.producer.verify().unchanged);
        let original_permissions = std::fs::metadata(&binary)?.permissions();
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        std::fs::write(&binary, b"modified executable")?;
        std::fs::set_permissions(&binary, original_permissions)?;
        assert_eq!(bijux_dna_infra::hash_file_sha256(&archive.archive_file)?, archive_sha256);
        let observed = seal.extracted.verify();
        assert!(!observed.unchanged);
        assert_eq!(observed.receipt["changed_inputs"][0], binary.display().to_string());
        assert!(seal.producer.verify().unchanged);
        std::fs::write(&producer_binary, b"changed retained producer")?;
        assert!(!seal.producer.verify().unchanged);
        let cargo_metadata = std::fs::read(metadata.join("cargo-metadata.json"))?;
        let binary_metadata = std::fs::read(metadata.join("binaries-metadata.json"))?;
        super::remove_full_shard_ephemera(&archive.archive_extract)?;
        std::fs::create_dir_all(&metadata)?;
        std::fs::create_dir_all(binary.parent().context("binary parent")?)?;
        std::fs::write(metadata.join("cargo-metadata.json"), cargo_metadata)?;
        std::fs::write(metadata.join("binaries-metadata.json"), binary_metadata)?;
        std::fs::hard_link(&producer_binary, &binary)?;
        super::seal_full_archive_extraction(&archive.archive_extract)?;
        assert!(super::capture_archive_executable_seal(&archive, &source).is_err());
        super::remove_full_shard_ephemera(&archive.archive_extract)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn shared_archive_sealing_never_follows_external_symlinks() -> anyhow::Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let temp = tempfile::tempdir()?;
        let outside = temp.path().join("outside");
        let archive = temp.path().join("extracted");
        std::fs::create_dir(&outside)?;
        std::fs::create_dir(&archive)?;
        let file = outside.join("writable");
        std::fs::write(&file, b"outside authority")?;
        let mode = std::fs::metadata(&file)?.permissions().mode();
        symlink(&outside, archive.join("linked"))?;
        super::seal_full_archive_extraction(&archive)?;
        assert_eq!(std::fs::metadata(&file)?.permissions().mode(), mode);
        assert_eq!(std::fs::read(&file)?, b"outside authority");
        assert!(super::seal_full_archive_extraction(&archive.join("linked")).is_err());
        super::remove_full_shard_ephemera(&archive)?;
        assert!(file.is_file());
        Ok(())
    }

    #[test]
    fn full_shard_archive_is_read_only_during_partition_execution() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let artifact_root = root.join("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let archive_root = tempfile::tempdir_in(&artifact_root)?;
        let archive = archive_root.path().join("workspace-tests.tar.zst");
        std::fs::write(&archive, b"checksum-bound archive")?;

        set_full_archive_readonly(&archive)?;

        assert!(std::fs::metadata(&archive)?.permissions().readonly());
        Ok(())
    }

    #[test]
    fn full_shard_archive_output_root_is_prepared_independently_of_the_cache() -> anyhow::Result<()>
    {
        let root = tempfile::tempdir()?;
        let storage = super::FullShardStorageRoots {
            retained_report_root: root.path().join("reports"),
            scratch_root: root.path().join("scratch"),
        };
        let roots = full_archive_roots(&storage, &contract())?;
        super::prepare_full_archive_roots(&roots)?;
        assert!(!roots.cargo_target.starts_with(&roots.artifact_root));
        assert!(roots.cargo_target.is_dir());
        assert!(roots.artifact_root.is_dir());
        std::fs::write(&roots.archive_file, b"archive output")?;
        Ok(())
    }
}
