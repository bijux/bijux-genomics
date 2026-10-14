use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context, Result};

pub(crate) fn workspace_root() -> Result<PathBuf> {
    if let Some(root) = std::env::var_os("BIJUX_REPO_ROOT") {
        ensure!(!root.is_empty(), "BIJUX_REPO_ROOT must identify a repository root");
        return validate_root(Path::new(&root)).context("validate BIJUX_REPO_ROOT");
    }
    let cwd = std::env::current_dir().context("resolve reference provider current directory")?;
    for root in cwd.ancestors() {
        if is_repository_root(root) {
            return validate_root(root);
        }
    }
    let origin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow!("reference provider has no repository default"))?;
    validate_root(origin).context("validate unbound reference provider repository default")
}

fn is_repository_root(root: &Path) -> bool {
    root.join("Cargo.toml").is_file()
        && root.join("Cargo.lock").is_file()
        && root.join("crates/bijux-dna/Cargo.toml").is_file()
        && root.join("configs").is_dir()
}

fn validate_root(root: &Path) -> Result<PathBuf> {
    ensure!(is_repository_root(root), "not a Bijux Genomics repository root: {}", root.display());
    let workspace_path = root.join("Cargo.toml");
    let package_path = root.join("crates/bijux-dna/Cargo.toml");
    let workspace: toml::Value = toml::from_str(&std::fs::read_to_string(&workspace_path)?)
        .with_context(|| format!("parse {}", workspace_path.display()))?;
    let package: toml::Value = toml::from_str(&std::fs::read_to_string(&package_path)?)
        .with_context(|| format!("parse {}", package_path.display()))?;
    ensure!(
        workspace.get("workspace").and_then(toml::Value::as_table).is_some(),
        "repository root must declare a Cargo workspace"
    );
    ensure!(
        package.get("package").and_then(|value| value.get("name")).and_then(toml::Value::as_str)
            == Some("bijux-dna"),
        "repository root must contain the bijux-dna package"
    );
    root.canonicalize().with_context(|| format!("resolve repository root {}", root.display()))
}
