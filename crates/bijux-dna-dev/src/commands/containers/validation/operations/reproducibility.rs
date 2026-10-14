#[cfg(unix)]
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Context, Result};

use super::{fs, run_program_with_env, sha256_hex, ContainerCommandOutcome, Workspace};

pub(super) struct CacheObservations {
    pub(super) same_cache_twice: bool,
    pub(super) clean_cache_match: bool,
    pub(super) purge_cache_match: bool,
    pub(super) outcome: ContainerCommandOutcome,
}

impl CacheObservations {
    #[must_use]
    pub(super) fn deterministic(&self) -> bool {
        self.same_cache_twice && self.clean_cache_match && self.purge_cache_match
    }
}

struct OwnedCaches {
    root: PathBuf,
    cache: PathBuf,
    temporary: PathBuf,
    #[cfg(unix)]
    identity: (u64, u64),
}

impl OwnedCaches {
    fn create(workspace: &Workspace, output: &Path, tool: &str) -> Result<Self> {
        let parent = workspace.admit_artifact_path(&output.join("build-observations"))?;
        fs::create_dir_all(&parent)
            .with_context(|| format!("create build observation parent {}", parent.display()))?;
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = parent.join(format!("{tool}-{}-{nonce}", std::process::id()));
        fs::create_dir(&root).with_context(|| {
            format!("create exclusive build observation root {}", root.display())
        })?;
        let root = root.canonicalize().context("resolve owned build observation root")?;
        let cache = root.join("cache");
        let temporary = root.join("temporary");
        fs::create_dir(&cache)?;
        fs::create_dir(&temporary)?;
        #[cfg(unix)]
        let metadata = fs::symlink_metadata(&root)?;
        Ok(Self {
            root,
            cache,
            temporary,
            #[cfg(unix)]
            identity: (metadata.dev(), metadata.ino()),
        })
    }

    fn environment(&self) -> Vec<(String, String)> {
        vec![
            ("APPTAINER_CACHEDIR".to_string(), self.cache.display().to_string()),
            ("APPTAINER_TMPDIR".to_string(), self.temporary.display().to_string()),
        ]
    }

    fn validate_owner(&self) -> Result<()> {
        let root = fs::symlink_metadata(&self.root).context("inspect invocation owner")?;
        ensure!(
            root.is_dir() && !root.file_type().is_symlink(),
            "invocation owner must remain a directory"
        );
        ensure!(
            self.root.canonicalize()? == self.root,
            "invocation owner ancestors must remain unaliased"
        );
        #[cfg(unix)]
        ensure!((root.dev(), root.ino()) == self.identity, "invocation directory identity changed");
        // Validate both owned roots before either removal. Never traverse a substituted link.
        for path in [&self.cache, &self.temporary] {
            ensure!(path.parent() == Some(self.root.as_path()), "cache root lost its owner");
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("inspect owned cache root {}", path.display()))?;
            ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "owned cache root must remain a directory: {}",
                path.display()
            );
        }
        Ok(())
    }

    fn purge(&self) -> Result<()> {
        self.validate_owner()?;
        for path in [&self.cache, &self.temporary] {
            fs::remove_dir_all(path)
                .with_context(|| format!("purge owned cache root {}", path.display()))?;
            fs::create_dir(path)
                .with_context(|| format!("recreate owned cache root {}", path.display()))?;
        }
        Ok(())
    }
}

fn build(
    workspace: &Workspace,
    definition: &Path,
    output: &Path,
    environment: &[(String, String)],
) -> Result<ContainerCommandOutcome> {
    run_program_with_env(
        workspace,
        "apptainer",
        &[
            "build".to_string(),
            "--force".to_string(),
            output.display().to_string(),
            definition.display().to_string(),
        ],
        environment,
    )
}

fn image_hash(path: &Path) -> Result<String> {
    Ok(sha256_hex(&fs::read(path).with_context(|| format!("read {}", path.display()))?))
}

pub(super) fn observe_cache_scenarios(
    workspace: &Workspace,
    tool: &str,
    output: &Path,
) -> Result<CacheObservations> {
    let definition = workspace.path(&format!("containers/apptainer/{tool}.def"));
    ensure!(definition.is_file(), "sampled Apptainer definition missing: {}", definition.display());
    let owned = OwnedCaches::create(workspace, output, tool)?;
    let environment = owned.environment();
    let paths = ["baseline.sif", "same-cache.sif", "clean-cache.sif", "purged-cache.sif"]
        .map(|name| owned.root.join(name));
    let mut observation = CacheObservations {
        same_cache_twice: false,
        clean_cache_match: false,
        purge_cache_match: false,
        outcome: ContainerCommandOutcome::success(String::new()),
    };
    let mut hashes = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        if index == 2 {
            owned.validate_owner()?;
            let outcome = run_program_with_env(
                workspace,
                "apptainer",
                &["cache".to_string(), "clean".to_string(), "-f".to_string()],
                &environment,
            )?;
            if !outcome.is_success() {
                observation.outcome = outcome;
                return Ok(observation);
            }
        } else if index == 3 {
            owned.purge()?;
        }
        owned.validate_owner()?;
        let outcome = build(workspace, &definition, path, &environment)?;
        if !outcome.is_success() {
            observation.outcome = outcome;
            return Ok(observation);
        }
        owned.validate_owner()?;
        hashes.push(image_hash(path)?);
        match index {
            1 => observation.same_cache_twice = hashes[0] == hashes[1],
            2 => observation.clean_cache_match = hashes[0] == hashes[2],
            3 => observation.purge_cache_match = hashes[0] == hashes[3],
            _ => {}
        }
    }
    observation.outcome = if observation.deterministic() {
        ContainerCommandOutcome::success(format!("apptainer rebuild repro: OK ({tool})\n"))
    } else {
        ContainerCommandOutcome::failure(format!(
            "apptainer rebuild repro: SIF hash mismatch for {tool}\n- baseline: {}\n- same cache: {}\n- clean cache: {}\n- purged cache: {}\n",
            hashes[0], hashes[1], hashes[2], hashes[3]
        ))
    };
    Ok(observation)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn owned_cache_purge_preserves_foreign_and_caller_roots() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let workspace = Workspace { root: directory.path().to_path_buf() };
        let caller_cache = workspace.path("artifacts/caller-cache");
        let foreign = workspace.path("foreign-cache");
        fs::create_dir_all(&caller_cache)?;
        fs::create_dir_all(&foreign)?;
        fs::write(caller_cache.join("retained"), b"caller-owned")?;
        fs::write(foreign.join("retained"), b"foreign-owned")?;
        let owned =
            OwnedCaches::create(&workspace, &workspace.path("artifacts/observations"), "fixture")?;
        fs::write(owned.cache.join("discard"), b"owned-cache")?;
        fs::write(owned.temporary.join("discard"), b"owned-temporary")?;
        owned.purge()?;
        assert!(!owned.cache.join("discard").exists());
        assert!(!owned.temporary.join("discard").exists());
        assert_eq!(fs::read(caller_cache.join("retained"))?, b"caller-owned");
        assert_eq!(fs::read(foreign.join("retained"))?, b"foreign-owned");

        fs::write(owned.cache.join("retained-before-refusal"), b"do-not-remove")?;
        fs::remove_dir(&owned.temporary)?;
        std::os::unix::fs::symlink(&foreign, &owned.temporary)?;
        assert!(
            owned.purge().is_err(),
            "a substituted owned root must refuse before either removal"
        );
        assert_eq!(fs::read(owned.cache.join("retained-before-refusal"))?, b"do-not-remove");
        assert_eq!(fs::read(foreign.join("retained"))?, b"foreign-owned");
        assert_eq!(fs::read(caller_cache.join("retained"))?, b"caller-owned");

        let foreign_owner = workspace.path("foreign-owner");
        for name in ["cache", "temporary"] {
            fs::create_dir_all(foreign_owner.join(name))?;
            fs::write(foreign_owner.join(name).join("retained"), b"foreign-child")?;
        }
        let redirected = OwnedCaches::create(
            &workspace,
            &workspace.path("artifacts/observations"),
            "redirected",
        )?;
        fs::rename(&redirected.root, redirected.root.with_file_name("retained-original-root"))?;
        std::os::unix::fs::symlink(&foreign_owner, &redirected.root)?;
        assert!(
            redirected.purge().is_err(),
            "replacing the invocation owner with a symlink must refuse"
        );
        for name in ["cache", "temporary"] {
            assert_eq!(fs::read(foreign_owner.join(name).join("retained"))?, b"foreign-child");
        }

        let replaced =
            OwnedCaches::create(&workspace, &workspace.path("artifacts/observations"), "replaced")?;
        fs::rename(&replaced.root, replaced.root.with_file_name("retained-regular-root"))?;
        for path in [&replaced.cache, &replaced.temporary] {
            fs::create_dir_all(path)?;
            fs::write(path.join("retained"), b"replacement-child")?;
        }
        assert!(
            replaced.purge().is_err(),
            "a different regular directory at the same path is not the acquired invocation owner"
        );
        for path in [&replaced.cache, &replaced.temporary] {
            assert_eq!(fs::read(path.join("retained"))?, b"replacement-child");
        }
        Ok(())
    }
}
