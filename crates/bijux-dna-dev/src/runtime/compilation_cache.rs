//! Exclusive mutable Cargo products, retained independently of disposable run artifacts.

use anyhow::{ensure, Context, Result};
use bijux_dna_infra::FileLock;
use std::path::{Path, PathBuf};

pub(crate) struct CompilationCache {
    target: PathBuf,
    _writer: FileLock,
}

impl CompilationCache {
    pub(crate) fn target_under(scratch: &Path) -> PathBuf {
        scratch.join("compile-cache/cargo")
    }

    /// Fail immediately on contention; independent producers must use separate scratch authorities.
    pub(crate) fn acquire(scratch: &Path) -> Result<Self> {
        let target = Self::target_under(scratch);
        let root = target.parent().context("cache target has no owner")?;
        bijux_dna_infra::ensure_dir(root)?;
        ensure!(
            !root.symlink_metadata()?.file_type().is_symlink(),
            "compilation cache owner must not be a symlink"
        );
        let writer = FileLock::try_acquire(&root.join("writer.lock"))?;
        bijux_dna_infra::ensure_dir(&target)?;
        ensure!(
            !target.symlink_metadata()?.file_type().is_symlink(),
            "compilation target must not be a symlink"
        );
        Ok(Self { target, _writer: writer })
    }

    pub(crate) fn target(&self) -> &Path {
        &self.target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compilation_cache_retains_products_and_excludes_concurrent_writers() -> Result<()> {
        let root = tempfile::tempdir()?;
        let lease = CompilationCache::acquire(root.path())?;
        let product = lease.target().join("retained.rlib");
        std::fs::write(&product, b"compiler-owned product")?;
        assert!(CompilationCache::acquire(root.path()).is_err());
        let independent = root.path().join("independent");
        let other = CompilationCache::acquire(&independent)?;
        assert_ne!(lease.target(), other.target());
        drop(lease);
        let reacquired = CompilationCache::acquire(root.path())?;
        assert_eq!(
            std::fs::read(reacquired.target().join("retained.rlib"))?,
            b"compiler-owned product"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn compilation_cache_rejects_redirected_output_ownership() -> Result<()> {
        let root = tempfile::tempdir()?;
        let external = root.path().join("external");
        std::fs::create_dir(&external)?;
        std::os::unix::fs::symlink(&external, root.path().join("compile-cache"))?;
        assert!(CompilationCache::acquire(root.path()).is_err());
        assert!(!external.join("writer.lock").exists());
        Ok(())
    }
    #[test]
    fn compilation_cache_native_cargo_reuses_and_invalidates_products() -> Result<()> {
        use crate::runtime::{process::ProcessRunner, workspace::Workspace};
        let root = tempfile::tempdir()?;
        let source = root.path().join("source");
        std::fs::create_dir_all(source.join("src"))?;
        std::fs::write(source.join("Cargo.toml"), "[package]\nname = \"cache-fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n")?;
        let input = source.join("src/main.rs");
        std::fs::write(&input, "fn main() { println!(\"original\"); }\n")?;
        let workspace = Workspace { root: source };
        let runner = ProcessRunner::new(&workspace);
        for (index, expected_fresh) in [false, true, false].into_iter().enumerate() {
            if index == 2 {
                std::fs::write(&input, "fn main() { println!(\"changed input\"); }\n")?;
            }
            let cache = CompilationCache::acquire(root.path())?;
            let output = runner.run_owned_with_env(
                "cargo",
                &[
                    "build".into(),
                    "--offline".into(),
                    "--jobs".into(),
                    "1".into(),
                    "--message-format=json".into(),
                ],
                &[
                    ("CARGO_TARGET_DIR".into(), cache.target().display().to_string()),
                    ("RUSTC_WRAPPER".into(), String::new()),
                ],
            )?;
            ensure!(
                output.status.success(),
                "native cache fixture build: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let artifacts = String::from_utf8(output.stdout)?
                .lines()
                .map(serde_json::from_str::<serde_json::Value>)
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let artifact = artifacts
                .iter()
                .find(|row| {
                    row["reason"] == "compiler-artifact" && row["target"]["name"] == "cache-fixture"
                })
                .context("native compiler artifact missing")?;
            assert_eq!(
                artifact["fresh"], expected_fresh,
                "native Cargo reuse differs at build {index}"
            );
            let executable =
                artifact["executable"].as_str().context("native executable missing")?;
            let execution = runner.run(&[executable])?;
            ensure!(execution.status.success(), "native cache fixture failed");
            assert_eq!(
                String::from_utf8(execution.stdout)?.trim(),
                if index == 2 { "changed input" } else { "original" }
            );
        }
        Ok(())
    }
}
