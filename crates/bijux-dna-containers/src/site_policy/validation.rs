use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use regex::Regex;

use super::model::{ContainerGovernance, CONTAINER_GOVERNANCE_SCHEMA_V1};

/// Load and structurally validate a container site-governance contract.
///
/// # Errors
/// Returns an error when the file cannot be read, YAML decoding fails, or policy validation fails.
pub fn load_container_governance(path: &Path) -> Result<ContainerGovernance> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read container governance {}", path.display()))?;
    let governance: ContainerGovernance =
        serde_yaml::from_str(&raw).context("parse container governance YAML")?;
    validate_container_governance(&governance)?;
    Ok(governance)
}

/// Validate the durable structure and invariants of a container site-governance contract.
///
/// # Errors
/// Returns an error for unsupported schemas, invalid roots, malformed site or Slurm policy, or
/// non-deterministic preflight lists.
pub fn validate_container_governance(governance: &ContainerGovernance) -> Result<()> {
    ensure!(
        governance.schema_version == CONTAINER_GOVERNANCE_SCHEMA_V1,
        "unsupported container governance schema"
    );
    ensure!(!governance.site.id.trim().is_empty(), "site id is empty");
    Regex::new(&governance.site.hostname_regex).context("compile governed hostname regex")?;

    let paths = &governance.paths;
    for (label, path) in [
        ("study_root", &paths.study_root),
        ("containers_root", &paths.containers_root),
        ("data_root", &paths.data_root),
        ("results_root", &paths.results_root),
        ("finalization_root", &paths.finalization_root),
        ("active_repository", &paths.active_repository),
        ("frozen_repositories", &paths.frozen_repositories),
    ] {
        ensure!(path.is_absolute(), "paths.{label} must be absolute");
    }
    for (label, path) in [
        ("containers_root", &paths.containers_root),
        ("data_root", &paths.data_root),
        ("results_root", &paths.results_root),
        ("finalization_root", &paths.finalization_root),
    ] {
        ensure!(
            path.starts_with(&paths.study_root),
            "paths.{label} must remain under paths.study_root"
        );
    }
    ensure!(
        paths.active_repository != paths.frozen_repositories,
        "active and frozen repository roots must differ"
    );
    ensure!(
        matches!(governance.runtime.apptainer_architecture.as_str(), "linux/amd64" | "linux/arm64"),
        "unsupported Apptainer architecture"
    );

    let slurm = &governance.slurm;
    ensure!(!slurm.account.trim().is_empty(), "Slurm account is empty");
    ensure_sorted_unique_nonempty(&slurm.partitions, "Slurm partitions")?;
    ensure_nonempty(&slurm.quota_command, "quota command")?;
    ensure!(
        slurm.scratch_environment.is_some() ^ slurm.scratch_root.is_some(),
        "exactly one Slurm scratch authority must be configured"
    );
    if let Some(name) = &slurm.scratch_environment {
        ensure!(
            Regex::new(r"^[A-Z][A-Z0-9_]*$")?.is_match(name),
            "Slurm scratch environment is invalid"
        );
    }
    if let Some(root) = &slurm.scratch_root {
        ensure!(root.is_absolute(), "Slurm scratch root must be absolute");
    }

    let preflight = &governance.preflight;
    ensure!(preflight.minimum_available_bytes > 0, "minimum available bytes must be positive");
    ensure!(preflight.minimum_available_inodes > 0, "minimum available inodes must be positive");
    ensure_sorted_unique_nonempty(&preflight.required_commands, "required commands")?;
    ensure_sorted_unique_nonempty(&preflight.required_contracts, "required contracts")?;
    Ok(())
}

/// Validate a governance contract and admit one host against its complete hostname policy.
///
/// # Errors
/// Returns an error when policy validation fails, the hostname expression is invalid, or the host
/// is outside the governed site.
pub fn validate_container_governance_host(
    governance: &ContainerGovernance,
    hostname: &str,
) -> Result<()> {
    validate_container_governance(governance)?;
    let pattern = Regex::new(&format!(r"\A(?:{})\z", governance.site.hostname_regex))?;
    ensure!(
        pattern.is_match(hostname),
        "host {hostname:?} is outside governed site {:?}",
        governance.site.id
    );
    Ok(())
}

fn ensure_nonempty(values: &[String], label: &str) -> Result<()> {
    ensure!(!values.is_empty(), "{label} is empty");
    ensure!(values.iter().all(|value| !value.trim().is_empty()), "{label} contains an empty value");
    Ok(())
}

fn ensure_sorted_unique_nonempty(values: &[String], label: &str) -> Result<()> {
    ensure_nonempty(values, label)?;
    let normalized =
        values.iter().cloned().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    ensure!(values == normalized, "{label} must be sorted and unique");
    Ok(())
}

pub(super) fn safe_checkout_path(path: &Path) -> Result<PathBuf> {
    ensure!(!path.as_os_str().is_empty(), "source freeze path is empty");
    ensure!(!path.is_absolute(), "source freeze path is absolute: {}", path.display());
    ensure!(
        path.components().all(|component| matches!(component, std::path::Component::Normal(_))),
        "source freeze path is unsafe: {}",
        path.display()
    );
    Ok(path.to_path_buf())
}
