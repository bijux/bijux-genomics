//! Owner: source selection and fresh acquisition of governed runtime-probe declarations.
use crate::BenchmarkRuntimeProbe;
use anyhow::{anyhow, ensure, Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Deserialize)]
struct GovernedRuntimeProbe {
    tool_id: String,
    #[serde(flatten)]
    probe: BenchmarkRuntimeProbe,
}
/// Load a fresh runtime projection for the requested tool.
/// # Errors
/// Returns identifier, file, declaration identity or decode failures.
pub fn load_runtime_probe(
    repo_root: &Path,
    domain: &str,
    tool_id: &str,
) -> Result<BenchmarkRuntimeProbe> {
    Ok(load_runtime_probe_with_source(repo_root, domain, tool_id)?.probe)
}

/// Current projection and exact selected source path, including a governed domain fallback.
pub struct LoadedRuntimeProbe {
    pub path: PathBuf,
    pub probe: BenchmarkRuntimeProbe,
}

/// Load the preferred domain first, then the existing canonical fallback order.
/// # Errors
/// Rejects unsafe identifiers and malformed or contradictory existing preferred files.
pub fn load_runtime_probe_with_source(
    repo_root: &Path,
    domain: &str,
    tool_id: &str,
) -> Result<LoadedRuntimeProbe> {
    ensure!(
        matches!(domain, "fastq" | "bam" | "vcf"),
        "unsupported runtime-probe domain `{domain}`"
    );
    ensure!(
        !tool_id.is_empty()
            && !matches!(tool_id, "." | "..")
            && tool_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric()
                    || matches!(byte, b'-' | b'_' | b'.' | b'+')),
        "runtime-probe tool identity must be a single canonical path component"
    );
    let mut probe_domains = vec![domain.to_string()];
    for fallback in ["fastq", "bam", "vcf"] {
        if !probe_domains.iter().any(|candidate| candidate == fallback) {
            probe_domains.push(fallback.to_string());
        }
    }

    let mut attempted_paths = Vec::new();
    for probe_domain in probe_domains {
        let yaml_path = repo_root
            .join("domain")
            .join(&probe_domain)
            .join("tools")
            .join(format!("{tool_id}.yaml"));
        attempted_paths.push(yaml_path.display().to_string());
        if !yaml_path.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&yaml_path)
            .with_context(|| format!("read {}", yaml_path.display()))?;
        let document: GovernedRuntimeProbe = bijux_dna_infra::formats::parse_yaml(&raw)
            .with_context(|| format!("parse {}", yaml_path.display()))?;
        ensure!(
            document.tool_id == tool_id,
            "runtime probe `{}` declares tool `{}` but was requested for `{}`",
            yaml_path.display(),
            document.tool_id,
            tool_id
        );
        return Ok(LoadedRuntimeProbe { path: yaml_path, probe: document.probe });
    }

    Err(anyhow!(
        "no governed runtime probe found for tool `{}`; looked in {}",
        tool_id,
        attempted_paths.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn probe_observation_preserves_preference_and_rejects_stale_or_foreign_identity() -> Result<()>
    {
        let directory = bijux_dna_testkit::temp::tempdir_for("runtime-probe-observation");
        let root = directory.path();
        let preferred = root.join("domain/bam/tools/tool.yaml");
        let fallback = root.join("domain/fastq/tools/tool.yaml");
        std::fs::create_dir_all(preferred.parent().ok_or_else(|| anyhow!("preferred parent"))?)?;
        std::fs::create_dir_all(fallback.parent().ok_or_else(|| anyhow!("fallback parent"))?)?;
        std::fs::write(
            &fallback,
            "tool_id: tool\ninstall_kind: host_binary\ncommand_template: [fallback]\n",
        )?;
        let loaded = load_runtime_probe_with_source(root, "bam", "tool")?;
        assert_eq!(loaded.path, fallback);
        assert_eq!(loaded.probe.command_entrypoint().as_deref(), Some("fallback"));
        std::fs::write(
            &preferred,
            "tool_id: tool\ninstall_kind: host_binary\ncommand_template: [preferred]\n",
        )?;
        assert_eq!(load_runtime_probe_with_source(root, "bam", "tool")?.path, preferred);
        assert_eq!(
            load_runtime_probe(root, "bam", "tool")?.command_entrypoint().as_deref(),
            Some("preferred")
        );
        std::fs::write(&preferred, "tool_id: other\ninstall_kind: host_binary\n")?;
        assert!(load_runtime_probe(root, "bam", "tool")
            .err()
            .is_some_and(|error| error.to_string().contains("declares tool `other`")));
        std::fs::write(&preferred, "tool_id: [\n")?;
        assert!(
            load_runtime_probe(root, "bam", "tool").is_err(),
            "malformed preferred input must not silently fall back"
        );
        std::fs::write(&preferred, "install_kind: host_binary\n")?;
        assert!(
            load_runtime_probe(root, "bam", "tool").is_err(),
            "missing declared tool identity must reject"
        );
        for tool in ["", "..", "../tool", "tool/child", "/absolute", "tool name"] {
            assert!(load_runtime_probe(root, "bam", tool).is_err());
        }
        assert!(load_runtime_probe(root, "../bam", "tool").is_err());
        std::fs::remove_file(&preferred)?;
        assert_eq!(load_runtime_probe_with_source(root, "bam", "tool")?.path, fallback);
        std::fs::remove_file(&fallback)?;
        assert!(load_runtime_probe(root, "bam", "tool").is_err());
        Ok(())
    }
}
