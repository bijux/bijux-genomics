use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};

#[test]
fn engine_has_no_runner_dependency() -> Result<()> {
    let root = workspace_root()?;
    require_no_runner_dependency(&root, &metadata_at(&root)?)
}

fn workspace_root() -> Result<PathBuf> {
    let explicit = std::env::var_os("BIJUX_REPO_ROOT");
    let origin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("engine crate has no repository parent")?;
    resolve_workspace_root(explicit.as_deref(), origin)
}

fn resolve_workspace_root(explicit: Option<&OsStr>, origin: &Path) -> Result<PathBuf> {
    let root = explicit.map_or(origin, Path::new);
    ensure!(
        root.is_absolute()
            && !root
                .components()
                .any(|part| matches!(part, Component::CurDir | Component::ParentDir)),
        "engine metadata requires a normalized absolute repository root"
    );
    ensure!(root.is_dir() && root.canonicalize()? == root, "engine repository root is redirected");
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/bijux-dna/Cargo.toml",
        "crates/bijux-dna-engine/Cargo.toml",
        "crates/bijux-dna-runner/Cargo.toml",
    ] {
        let path = root.join(relative);
        ensure!(
            path.is_file() && path.canonicalize()? == path,
            "engine repository input is missing or redirected: {}",
            path.display()
        );
    }
    let configs = root.join("configs");
    ensure!(
        configs.is_dir() && configs.canonicalize()? == configs,
        "engine repository configurations are missing or redirected"
    );
    Ok(root.to_owned())
}

fn metadata_at(root: &Path) -> Result<Metadata> {
    let root = resolve_workspace_root(Some(root.as_os_str()), root)?;
    let runtime = root.join("artifacts/tests/engine-metadata-runtime");
    std::fs::create_dir_all(&runtime)?;
    let mut command = MetadataCommand::new();
    command
        .manifest_path(root.join("Cargo.toml"))
        .current_dir(&root)
        .other_options(vec!["--locked".to_owned()])
        .env("CARGO_TARGET_DIR", runtime.join("target"))
        .env("TMPDIR", &runtime)
        .env("TMP", &runtime)
        .env("TEMP", &runtime);
    // Preserve the caller's tool provider while keeping this command's scratch private.
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        for (name, default) in [("CARGO_HOME", ".cargo"), ("RUSTUP_HOME", ".rustup")] {
            command.env(
                name,
                std::env::var_os(name).map_or_else(|| home.join(default), PathBuf::from),
            );
        }
    }
    command.env("HOME", &runtime);
    command.exec().context("load metadata for the selected engine repository")
}

fn workspace_package<'a>(metadata: &'a Metadata, root: &Path, name: &str) -> Result<&'a Package> {
    let expected = root.join("crates").join(name).join("Cargo.toml");
    let package = metadata
        .packages
        .iter()
        .find(|package| package.name == name)
        .with_context(|| format!("{name} missing from selected repository"))?;
    ensure!(
        package.manifest_path.as_std_path() == expected
            && metadata.workspace_members.contains(&package.id),
        "{name} package identity does not belong to the selected repository"
    );
    Ok(package)
}

fn require_no_runner_dependency(root: &Path, metadata: &Metadata) -> Result<()> {
    ensure!(
        metadata.workspace_root.as_std_path() == root,
        "metadata workspace identity differs from the selected repository"
    );
    workspace_package(metadata, root, "bijux-dna")?;
    let engine = workspace_package(metadata, root, "bijux-dna-engine")?;
    let runner = workspace_package(metadata, root, "bijux-dna-runner")?;
    let resolve = metadata.resolve.as_ref().context("resolve graph missing")?;
    let node =
        resolve.nodes.iter().find(|node| node.id == engine.id).context("engine node missing")?;
    let has_edge = node.deps.iter().any(|dependency| dependency.pkg == runner.id);
    ensure!(!has_edge, "bijux-dna-engine must not depend on bijux-dna-runner");
    Ok(())
}

fn private_workspace(runner_dependency: bool) -> Result<tempfile::TempDir> {
    let scratch = workspace_root()?.join("artifacts/tests/engine-workspace-identity");
    std::fs::create_dir_all(&scratch)?;
    let owner = tempfile::Builder::new().prefix("workspace-").tempdir_in(&scratch)?;
    let root = owner.path();
    std::fs::create_dir(root.join("configs"))?;
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"crates/bijux-dna\", \"crates/bijux-dna-engine\", \"crates/bijux-dna-runner\"]\n",
    )?;
    let mut lock = "version = 4\n".to_owned();
    for name in ["bijux-dna", "bijux-dna-engine", "bijux-dna-runner"] {
        let package = root.join("crates").join(name);
        std::fs::create_dir_all(package.join("src"))?;
        let dependency = if runner_dependency && name == "bijux-dna-engine" {
            "\n[dependencies]\nbijux-dna-runner = { path = \"../bijux-dna-runner\" }\n"
        } else {
            ""
        };
        std::fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n{dependency}"),
        )?;
        std::fs::write(
            package.join("src/lib.rs"),
            "pub const OWNER: &str = \"metadata fixture\";\n",
        )?;
        write!(lock, "\n[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\n")?;
        if !dependency.is_empty() {
            lock.push_str("dependencies = [\"bijux-dna-runner\"]\n");
        }
    }
    std::fs::write(root.join("Cargo.lock"), lock)?;
    Ok(owner)
}

#[test]
fn metadata_dependency_checks_use_explicit_workspace_identity() -> Result<()> {
    let allowed = private_workspace(false)?;
    let forbidden = private_workspace(true)?;
    let allowed_root = allowed.path().canonicalize()?;
    let forbidden_root = forbidden.path().canonicalize()?;
    let allowed_metadata = metadata_at(&allowed_root)?;
    let forbidden_metadata = metadata_at(&forbidden_root)?;
    require_no_runner_dependency(&allowed_root, &allowed_metadata)?;
    assert!(require_no_runner_dependency(&forbidden_root, &forbidden_metadata).is_err());
    assert!(require_no_runner_dependency(&forbidden_root, &allowed_metadata).is_err());
    assert!(require_no_runner_dependency(&allowed_root, &forbidden_metadata).is_err());
    let mut redirected_package = allowed_metadata.clone();
    redirected_package
        .packages
        .iter_mut()
        .find(|package| package.name == "bijux-dna-engine")
        .context("fixture engine package")?
        .manifest_path =
        workspace_package(&forbidden_metadata, &forbidden_root, "bijux-dna-engine")?
            .manifest_path
            .clone();
    assert!(require_no_runner_dependency(&allowed_root, &redirected_package).is_err());
    Ok(())
}

#[test]
fn explicit_invalid_workspace_roots_do_not_fall_back() -> Result<()> {
    let valid = private_workspace(false)?;
    let root = valid.path().canonicalize()?;
    assert_eq!(resolve_workspace_root(None, &root)?, root);
    for invalid in [
        PathBuf::new(),
        PathBuf::from("relative"),
        root.join("absent"),
        root.join("Cargo.toml"),
        root.join("configs/.."),
    ] {
        assert!(resolve_workspace_root(Some(invalid.as_os_str()), &root).is_err());
    }
    let unrelated = root.join("artifacts/unrelated-workspace");
    std::fs::create_dir_all(unrelated.join("configs"))?;
    std::fs::write(unrelated.join("Cargo.toml"), "[workspace]\n")?;
    std::fs::write(unrelated.join("Cargo.lock"), "version = 4\n")?;
    assert!(resolve_workspace_root(Some(unrelated.as_os_str()), &root).is_err());
    #[cfg(unix)]
    {
        let redirected = root.join("artifacts/redirected-workspace");
        std::fs::create_dir_all(redirected.parent().context("redirect parent")?)?;
        std::os::unix::fs::symlink(&root, &redirected)?;
        assert!(resolve_workspace_root(Some(redirected.as_os_str()), &root).is_err());
    }
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"unrelated\"\nversion = \"0.1.0\"\n",
    )?;
    assert!(metadata_at(&root).is_err());
    Ok(())
}
