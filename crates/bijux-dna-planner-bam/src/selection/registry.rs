use std::cell::OnceCell;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Context, Result};
use bijux_dna_core::ids::ToolId;
use bijux_dna_domain_bam::BamStage;

/// A lazy successful registry observation bound to one explicit repository request.
///
/// Stability: additive planner contract. Legacy selection calls retain fresh reads.
#[derive(Debug)]
pub struct BamToolRegistryObservation {
    repo_root: PathBuf,
    registry: OnceCell<toml::Value>,
}

impl BamToolRegistryObservation {
    /// Capture a repository path without reading or admitting any files.
    #[must_use]
    pub fn new(repo_root: &Path) -> Self {
        Self { repo_root: repo_root.to_path_buf(), registry: OnceCell::new() }
    }

    /// Return the canonical candidate union from this request's admitted registry.
    ///
    /// The first successful demand validates the complete repository authority and
    /// registry. Later demands retain that observation; a new request observes changes.
    /// Failed admission remains retryable and never falls back to another root.
    ///
    /// # Errors
    /// Returns source-root, filesystem, TOML or registry-shape admission errors.
    pub fn allowed_tools_for_stage(&self, stage: BamStage) -> Result<Vec<ToolId>> {
        if self.registry.get().is_none() {
            ensure!(
                !self.repo_root.as_os_str().is_empty(),
                "BAM registry observation must identify a repository root"
            );
            let root =
                validate_root(&self.repo_root).context("validate BAM registry observation root")?;
            let path = bijux_dna_infra::configs_file(&root, "ci/registry/tool_registry.toml");
            let parsed = load_registry_toml(&path)?;
            ensure!(
                parsed.get("tools").and_then(toml::Value::as_array).is_some(),
                "BAM tool registry {} must declare tools",
                path.display()
            );
            self.registry
                .set(parsed)
                .map_err(|_| anyhow!("BAM registry observation was already admitted"))?;
        }
        let registry = self.registry.get().context("BAM registry observation was not admitted")?;
        Ok(super::tool_selection::canonical_tools_from_registry(stage, Some(registry)))
    }
}

pub(super) fn tool_registry_toml() -> Result<Option<toml::Value>> {
    let explicit = std::env::var_os("BIJUX_REPO_ROOT");
    let root = repository_root(explicit.as_deref())?;
    let path = bijux_dna_infra::configs_file(&root, "ci/registry/tool_registry.toml");
    if explicit.is_none() && !path.is_file() {
        return Ok(None);
    }
    let parsed = match load_registry_toml(&path) {
        Ok(parsed) => parsed,
        Err(_) if explicit.is_none() => return Ok(None),
        Err(error) => return Err(error),
    };
    ensure!(
        parsed.get("tools").and_then(toml::Value::as_array).is_some(),
        "BAM tool registry {} must declare tools",
        path.display()
    );
    Ok(Some(parsed))
}

fn load_registry_toml(path: &Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read BAM tool registry {}", path.display()))?;
    toml::from_str::<toml::Value>(&raw)
        .with_context(|| format!("parse BAM tool registry {}", path.display()))
}

fn repository_root(explicit: Option<&std::ffi::OsStr>) -> Result<PathBuf> {
    if let Some(root) = explicit {
        ensure!(!root.is_empty(), "BIJUX_REPO_ROOT must identify a repository root");
        return validate_root(Path::new(root)).context("validate BIJUX_REPO_ROOT");
    }
    let cwd = std::env::current_dir().context("resolve BAM registry current directory")?;
    for root in cwd.ancestors() {
        if is_repository_root(root) {
            return validate_root(root);
        }
    }
    let origin = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow!("BAM registry has no repository default"))?;
    validate_root(origin).context("validate unbound BAM registry repository default")
}

fn is_repository_root(root: &Path) -> bool {
    root.join("Cargo.toml").is_file()
        && root.join("Cargo.lock").is_file()
        && root.join("crates/bijux-dna/Cargo.toml").is_file()
        && bijux_dna_infra::configs_dir(root).is_dir()
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
