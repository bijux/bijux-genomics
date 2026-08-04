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
    use super::definition_file_sources;

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
}
