use std::collections::BTreeSet;
use std::path::{Component, Path};
use std::process::Command;

use anyhow::{bail, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::evidence::{BuildContextIdentity, BuildEvidenceStatus, ContainerBuildEvidence};

use super::catalog::validate_commit_identity;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApptainerRecipeIdentity {
    pub repository_commit: String,
    pub definition_path: String,
    pub definition_sha256: String,
    pub build_context: Vec<BuildContextIdentity>,
}

/// Resolve the repository's current commit to a full immutable identity.
///
/// # Errors
/// Returns an error when Git cannot resolve `HEAD` to a full commit identity.
pub fn repository_head_commit(repository_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .output()
        .context("resolve repository HEAD commit")?;
    let commit = String::from_utf8(output.stdout)
        .context("repository HEAD commit is not UTF-8")?
        .trim()
        .to_string();
    if !output.status.success() {
        bail!(
            "cannot resolve repository HEAD commit: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    validate_commit_identity(&commit)?;
    Ok(commit)
}

/// Resolve the repository's exact commit after proving that tracked and untracked inputs are clean.
///
/// # Errors
/// Returns an error when Git cannot inspect the repository, the worktree contains changes, or
/// `HEAD` is not a full immutable commit identity.
pub fn clean_repository_head_commit(repository_root: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["status", "--porcelain=v1", "--untracked-files=normal"])
        .output()
        .context("inspect repository worktree state")?;
    if !output.status.success() {
        bail!(
            "cannot inspect repository worktree state: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    if !output.stdout.is_empty() {
        bail!(
            "refusing container build from repository changes; commit every catalog, recipe, and build input first"
        );
    }
    repository_head_commit(repository_root)
}

/// List repository paths changed between one ancestor commit and the current `HEAD`.
///
/// The result is sorted, repository-relative, UTF-8, and rename detection is disabled so every
/// changed path remains explicit to callers enforcing source-freshness policy.
///
/// # Errors
/// Returns an error when the commit is malformed, is not an ancestor of `HEAD`, Git cannot inspect
/// the range, or a changed path is not a safe repository-relative UTF-8 path.
pub fn repository_changed_paths_since(
    repository_root: &Path,
    ancestor_commit: &str,
) -> Result<Vec<String>> {
    validate_commit_identity(ancestor_commit)?;
    let head = repository_head_commit(repository_root)?;
    if ancestor_commit == head {
        return Ok(Vec::new());
    }
    if !repository_commit_is_ancestor_of_head(repository_root, ancestor_commit)? {
        bail!("repository source commit is not an ancestor of HEAD");
    }
    let range = format!("{ancestor_commit}..{head}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args([
            "diff",
            "--name-only",
            "--no-ext-diff",
            "--no-renames",
            "--diff-filter=ACDMRTUXB",
            "-z",
            &range,
            "--",
        ])
        .output()
        .context("inspect repository paths changed since source commit")?;
    if !output.status.success() {
        bail!(
            "cannot inspect repository paths changed since source commit: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let mut paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path)
                .context("repository changed path is not UTF-8")?
                .to_string();
            validate_repository_path(&path, "repository changed path")?;
            Ok(path)
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Return whether one full commit identity is an ancestor of the repository's current `HEAD`.
///
/// # Errors
/// Returns an error when the commit is malformed or Git cannot inspect repository ancestry.
pub fn repository_commit_is_ancestor_of_head(repository_root: &Path, commit: &str) -> Result<bool> {
    let head = repository_head_commit(repository_root)?;
    repository_commit_is_ancestor(repository_root, commit, &head)
}

pub(crate) fn repository_commit_is_ancestor(
    repository_root: &Path,
    ancestor_commit: &str,
    descendant_commit: &str,
) -> Result<bool> {
    validate_commit_identity(ancestor_commit)?;
    validate_commit_identity(descendant_commit)?;
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["merge-base", "--is-ancestor", ancestor_commit, descendant_commit])
        .output()
        .context("verify repository source ancestry")?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => bail!(
            "cannot inspect repository source ancestry: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

/// Resolve an Apptainer definition and every repository-owned `%files` input at one exact commit.
///
/// # Errors
/// Returns an error when the commit or path is unsafe, Git cannot resolve an input, the definition
/// is not UTF-8, or `%files` contains an absolute, traversing, or duplicate source.
pub fn apptainer_recipe_identity_at_commit(
    repository_root: &Path,
    commit: &str,
    definition_path: &str,
) -> Result<ApptainerRecipeIdentity> {
    validate_commit_identity(commit)?;
    validate_repository_path(definition_path, "Apptainer definition")?;
    let definition = git_show(repository_root, commit, definition_path)?;
    let sources = definition_file_sources(&definition)?;
    let build_context = sources
        .into_iter()
        .map(|path| {
            Ok(BuildContextIdentity {
                git_object: git_object(repository_root, commit, &path)?,
                path,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ApptainerRecipeIdentity {
        repository_commit: commit.to_string(),
        definition_path: definition_path.to_string(),
        definition_sha256: bijux_dna_infra::hash_bytes_sha256(&definition),
        build_context,
    })
}

/// Return whether build evidence exactly matches a committed recipe identity.
#[must_use]
pub fn container_build_evidence_matches_recipe(
    evidence: &ContainerBuildEvidence,
    recipe: &ApptainerRecipeIdentity,
) -> bool {
    matches!(evidence.status, BuildEvidenceStatus::Built | BuildEvidenceStatus::Passed)
        && evidence.repository_commit == recipe.repository_commit
        && evidence.definition_sha256 == recipe.definition_sha256
        && evidence.build_context.as_deref().unwrap_or_default() == recipe.build_context
}

fn definition_file_sources(definition: &[u8]) -> Result<Vec<String>> {
    let definition =
        std::str::from_utf8(definition).context("Apptainer definition is not UTF-8")?;
    let mut in_files = false;
    let mut sources = Vec::new();
    let mut seen = BTreeSet::new();
    for raw_line in definition.lines() {
        let line = raw_line.trim();
        if line.starts_with('%') {
            in_files = line == "%files";
            continue;
        }
        if !in_files || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let source = line
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Apptainer %files row has no source"))?;
        validate_repository_path(source, "Apptainer %files source")?;
        if !seen.insert(source.to_string()) {
            bail!("Apptainer %files source is duplicated: {source}");
        }
        sources.push(source.to_string());
    }
    Ok(sources)
}

fn validate_repository_path(value: &str, label: &str) -> Result<()> {
    let path = Path::new(value);
    if value.trim().is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        bail!("{label} must remain repository-relative: {value}");
    }
    Ok(())
}

fn git_show(repository_root: &Path, commit: &str, path: &str) -> Result<Vec<u8>> {
    let object = format!("{commit}:{path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["show", "--no-ext-diff", &object])
        .output()
        .with_context(|| format!("read {path} at repository commit {commit}"))?;
    if !output.status.success() {
        bail!(
            "cannot read {path} at repository commit {commit}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

fn git_object(repository_root: &Path, commit: &str, path: &str) -> Result<String> {
    let object = format!("{commit}:{path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["rev-parse", "--verify", &object])
        .output()
        .with_context(|| format!("identify {path} at repository commit {commit}"))?;
    let identity = String::from_utf8(output.stdout)
        .context("Git object identity is not UTF-8")?
        .trim()
        .to_string();
    if !output.status.success()
        || identity.len() != 40
        || !identity.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!(
            "cannot identify {path} at repository commit {commit}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(identity)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::{definition_file_sources, repository_changed_paths_since, repository_head_commit};

    #[test]
    fn definition_sources_preserve_complete_order() -> anyhow::Result<()> {
        let definition =
            b"Bootstrap: docker\n\n%files\n  containers/a /a\n  containers/b /b\n\n%post\n  true\n";
        assert_eq!(definition_file_sources(definition)?, ["containers/a", "containers/b"]);
        Ok(())
    }

    #[test]
    fn definition_sources_reject_unsafe_and_duplicate_paths() {
        for definition in [
            b"%files\n/etc/passwd /passwd\n".as_slice(),
            b"%files\n../secret /secret\n".as_slice(),
            b"%files\ncontainers/a /a\ncontainers/a /b\n".as_slice(),
        ] {
            assert!(definition_file_sources(definition).is_err());
        }
    }

    #[test]
    fn changed_paths_require_ancestor_history_and_preserve_publication_paths() -> anyhow::Result<()>
    {
        let temporary = bijux_dna_infra::temp_dir("repository-changed-paths")?;
        let root = temporary.path();
        git(root, &["init", "--quiet"])?;
        git(root, &["config", "user.email", "tests@bijux.invalid"])?;
        git(root, &["config", "user.name", "Bijux Tests"])?;
        std::fs::write(root.join("authority.toml"), b"version = 1\n")?;
        git(root, &["add", "authority.toml"])?;
        git(root, &["commit", "--quiet", "-m", "initial authority"])?;
        let source = repository_head_commit(root)?;

        std::fs::create_dir_all(root.join("benchmarks/reports"))?;
        std::fs::write(root.join("benchmarks/reports/run-2004-bam-mapping-summary.json"), b"{}\n")?;
        git(root, &["add", "benchmarks/reports/run-2004-bam-mapping-summary.json"])?;
        git(root, &["commit", "--quiet", "-m", "publish report"])?;
        assert_eq!(
            repository_changed_paths_since(root, &source)?,
            ["benchmarks/reports/run-2004-bam-mapping-summary.json"]
        );

        git(root, &["checkout", "--quiet", "--orphan", "unrelated"])?;
        std::fs::write(root.join("authority.toml"), b"version = 2\n")?;
        git(root, &["add", "authority.toml"])?;
        git(root, &["commit", "--quiet", "-m", "unrelated authority"])?;
        assert!(repository_changed_paths_since(root, &source).is_err());
        Ok(())
    }

    fn git(root: &Path, arguments: &[&str]) -> anyhow::Result<()> {
        let output = Command::new("git").arg("-C").arg(root).args(arguments).output()?;
        anyhow::ensure!(
            output.status.success(),
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
