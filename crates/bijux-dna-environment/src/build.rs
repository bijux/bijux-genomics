//! Environment catalog build helpers.
//!
//! Responsibilities: derive tool metadata from dockerfiles and curated defaults.
//! Invariants: no resolution side effects; outputs must be deterministic for the same inputs.

use std::path::Path;

use crate::resolve::EnvError;
use regex::Regex;

pub mod api {
    pub use crate::resolve::*;
}

#[derive(Debug, Clone)]
pub struct DockerToolSpec {
    pub name: String,
    pub executable: Option<String>,
    pub version_cmd: String,
    pub help_cmd: Option<String>,
    pub probe_cmd: Option<String>,
    pub probe_expected_exit: Vec<i32>,
}

/// Builder entrypoint for environment definitions.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentBuilder;

impl EnvironmentBuilder {
    #[must_use]
    pub fn default_docker_tools() -> Vec<DockerToolSpec> {
        default_docker_tools()
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

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn default_docker_tools() -> Vec<DockerToolSpec> {
    vec![
        DockerToolSpec {
            name: "adapterremoval".to_string(),
            executable: Some("adapterremoval".to_string()),
            version_cmd: "adapterremoval --version".to_string(),
            probe_cmd: Some("adapterremoval --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("adapterremoval --help".to_string()),
        },
        DockerToolSpec {
            name: "atropos".to_string(),
            executable: Some("atropos".to_string()),
            version_cmd: "atropos --version".to_string(),
            probe_cmd: Some("atropos --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("atropos --help".to_string()),
        },
        DockerToolSpec {
            name: "bbduk".to_string(),
            executable: Some("bbduk".to_string()),
            version_cmd: "bbduk -Xmx256m --version".to_string(),
            probe_cmd: Some("bbduk -Xmx256m --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("bbduk -Xmx256m --help".to_string()),
        },
        DockerToolSpec {
            name: "bowtie2".to_string(),
            executable: Some("bowtie2".to_string()),
            version_cmd: "bowtie2 --version".to_string(),
            probe_cmd: Some("bowtie2 --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("bowtie2 --help".to_string()),
        },
        DockerToolSpec {
            name: "cutadapt".to_string(),
            executable: Some("cutadapt".to_string()),
            version_cmd: "cutadapt --version".to_string(),
            probe_cmd: Some("cutadapt --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("cutadapt --help".to_string()),
        },
        DockerToolSpec {
            name: "fastp".to_string(),
            executable: Some("fastp".to_string()),
            version_cmd: "fastp --version".to_string(),
            probe_cmd: Some("fastp --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("fastp --help".to_string()),
        },
        DockerToolSpec {
            name: "fastqc".to_string(),
            executable: Some("fastqc".to_string()),
            version_cmd: "fastqc --version".to_string(),
            probe_cmd: Some("fastqc --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("fastqc --help".to_string()),
        },
        DockerToolSpec {
            name: "fastqvalidator".to_string(),
            executable: Some("fastqvalidator".to_string()),
            version_cmd: "fastqvalidator --version".to_string(),
            probe_cmd: Some("fastqvalidator --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("fastqvalidator --help".to_string()),
        },
        DockerToolSpec {
            name: "fqtools".to_string(),
            executable: Some("fqtools".to_string()),
            version_cmd: "fqtools --version".to_string(),
            probe_cmd: Some("fqtools --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("fqtools --help".to_string()),
        },
        DockerToolSpec {
            name: "kraken2".to_string(),
            executable: Some("kraken2".to_string()),
            version_cmd: "kraken2 --version".to_string(),
            probe_cmd: Some("kraken2 --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("kraken2 --help".to_string()),
        },
        DockerToolSpec {
            name: "multiqc".to_string(),
            executable: Some("multiqc".to_string()),
            version_cmd: "multiqc --version".to_string(),
            probe_cmd: Some("multiqc --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("multiqc --help".to_string()),
        },
        DockerToolSpec {
            name: "prinseq".to_string(),
            executable: Some("prinseq-lite.pl".to_string()),
            version_cmd: "prinseq-lite.pl --version".to_string(),
            probe_cmd: Some("prinseq-lite.pl --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("prinseq-lite.pl --help".to_string()),
        },
        DockerToolSpec {
            name: "samtools".to_string(),
            executable: Some("samtools".to_string()),
            version_cmd: "samtools --version".to_string(),
            probe_cmd: Some("samtools --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("samtools --help".to_string()),
        },
        DockerToolSpec {
            name: "sortmerna".to_string(),
            executable: Some("sortmerna".to_string()),
            version_cmd: "sortmerna --version".to_string(),
            probe_cmd: Some("sortmerna --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("sortmerna --help".to_string()),
        },
        DockerToolSpec {
            name: "seqpurge".to_string(),
            executable: Some("seqpurge".to_string()),
            version_cmd: "seqpurge --version".to_string(),
            probe_cmd: Some("seqpurge --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("seqpurge --help".to_string()),
        },
        DockerToolSpec {
            name: "bbmerge".to_string(),
            executable: Some("bbmerge".to_string()),
            version_cmd: "bbmerge --version".to_string(),
            probe_cmd: Some("bbmerge --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("bbmerge --help".to_string()),
        },
        DockerToolSpec {
            name: "rcorrector".to_string(),
            executable: Some("rcorrector".to_string()),
            version_cmd: "rcorrector --version".to_string(),
            probe_cmd: Some("rcorrector --version".to_string()),
            probe_expected_exit: vec![0],
            help_cmd: Some("rcorrector --help".to_string()),
        },
    ]
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
