//! Environment catalog build helpers.
//!
//! Responsibilities: derive tool metadata from dockerfiles and curated defaults.
//! Invariants: no resolution side effects; outputs must be deterministic for the same inputs.

use std::path::Path;

use crate::resolve::EnvError;
use regex::Regex;

mod defaults;
mod models;

pub mod api {
    pub use crate::resolve::*;
}

pub use models::DockerToolSpec;

/// Builder entrypoint for environment definitions.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    #[must_use]
    pub fn default_docker_tools() -> Vec<DockerToolSpec> {
        defaults::default_docker_tools()
    }

    /// # Errors
    /// Returns an error if the dockerfile cannot be parsed.
    pub fn extract_version_from_dockerfile(
        dockerfile: &Path,
        tool: &str,
    ) -> Result<String, EnvError> {
        extract_version_from_dockerfile(dockerfile, tool)
    }
}

#[must_use]
pub fn default_docker_tools() -> Vec<DockerToolSpec> {
    defaults::default_docker_tools()
}

/// Extract expected version from Dockerfile ARG lines for a given tool.
///
/// # Errors
/// Returns an error if the Dockerfile is missing or no version ARG is found.
pub fn extract_version_from_dockerfile(dockerfile: &Path, tool: &str) -> Result<String, EnvError> {
    let content = std::fs::read_to_string(dockerfile)?;
    let pattern = version_arg_pattern(tool);
    let regex = Regex::new(&pattern)
        .map_err(|err| EnvError::Dockerfile(format!("invalid regex: {err}")))?;
    let caps = regex.captures(&content).ok_or_else(|| {
        EnvError::Dockerfile(format!(
            "no version ARG found for tool {tool} in {}",
            dockerfile.display()
        ))
    })?;
    Ok(caps
        .get(1)
        .ok_or_else(|| EnvError::Dockerfile("missing capture".to_string()))?
        .as_str()
        .trim()
        .trim_matches(|ch| matches!(ch, '"' | '\''))
        .to_string())
}

fn version_arg_pattern(tool: &str) -> String {
    let names = version_arg_names(tool)
        .into_iter()
        .map(|name| regex::escape(&name))
        .collect::<Vec<_>>()
        .join("|");
    format!(r"(?im)^\s*ARG\s+(?:{names})\s*=\s*(\S+)\s*$")
}

fn version_arg_names(tool: &str) -> Vec<String> {
    let canonical = tool.to_uppercase().replace('-', "_");
    let mut names = vec![
        format!("VERSION_{canonical}"),
        format!("{canonical}_VERSION"),
    ];
    if canonical == "TRIM_GALORE" {
        names.push("TRIM_GALORE".to_string());
    }
    names.sort();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_dockerfile(name: &str, contents: &[u8]) -> Result<std::path::PathBuf, EnvError> {
        let path = std::env::temp_dir().join(name);
        bijux_dna_infra::atomic_write_bytes(&path, contents).map_err(std::io::Error::other)?;
        Ok(path)
    }

    #[test]
    fn extract_version_from_dockerfile_parses() -> Result<(), EnvError> {
        let path = write_dockerfile(
            "bijux_test_fastp.Dockerfile",
            b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
        )?;
        let version = extract_version_from_dockerfile(&path, "fastp")?;
        assert_eq!(version, "0.23.4");
        let _ = bijux_dna_infra::remove_file(&path);
        Ok(())
    }

    #[test]
    fn extract_version_from_dockerfile_ignores_unrelated_arg_names() -> Result<(), EnvError> {
        let path = write_dockerfile(
            "bijux_test_fastqvalidator_missing_version.Dockerfile",
            b"FROM ubuntu:20.04\nARG BASE_VERSION=24.04\n",
        )?;
        let error = extract_version_from_dockerfile(&path, "fastqvalidator")
            .err()
            .ok_or_else(|| EnvError::Dockerfile("expected missing tool version".to_string()))?;
        assert!(error
            .to_string()
            .contains("no version ARG found for tool fastqvalidator"));
        let _ = bijux_dna_infra::remove_file(&path);
        Ok(())
    }

    #[test]
    fn extract_version_from_dockerfile_strips_optional_quotes() -> Result<(), EnvError> {
        let path = write_dockerfile(
            "bijux_test_fastp_quoted_version.Dockerfile",
            b"FROM ubuntu:20.04\nARG VERSION_FASTP=\"0.23.4\"\n",
        )?;
        let version = extract_version_from_dockerfile(&path, "fastp")?;
        assert_eq!(version, "0.23.4");
        let _ = bijux_dna_infra::remove_file(&path);
        Ok(())
    }
}
