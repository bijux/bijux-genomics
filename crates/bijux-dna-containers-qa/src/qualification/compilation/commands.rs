//! Pure qualification command wrapping and output discovery conventions.
use crate::qualification::specification::QualificationSpecificationSourceV1;
use anyhow::{bail, Context, Result};
use bijux_dna_stage_contract::{OutputDiscoveryV1, ToolBinding};
use std::path::{Path, PathBuf};

pub(super) fn wrap_real_command(
    argv: &[String],
    source: &QualificationSpecificationSourceV1,
    binding: &impl ToolBinding,
) -> Result<Vec<String>> {
    let command = argv.iter().map(|token| shell_quote(token)).collect::<Vec<_>>().join(" ");
    let mut postconditions = Vec::new();
    for output in &source.outputs {
        let binding_output = binding
            .outputs()
            .iter()
            .find(|value| value.name == output.binding_output)
            .context("qualification output disappeared from its binding")?;
        if let OutputDiscoveryV1::Glob { pattern } = &binding_output.discovery {
            let expected = format!("${{output:{}}}*", output.binding_output);
            if pattern != &expected {
                bail!("qualification output uses an unsupported discovery pattern");
            }
            let parent = output.path.parent().unwrap_or_else(|| Path::new(""));
            let base = output
                .path
                .file_name()
                .and_then(|value| value.to_str())
                .context("qualification glob output has no UTF-8 basename")?;
            let search_root = if parent.as_os_str().is_empty() {
                PathBuf::from("/result")
            } else {
                Path::new("/result").join(parent)
            };
            let manifest = Path::new("/result").join(glob_manifest_path(&output.path));
            let partial_manifest = Path::new("/result/.qualification-glob-manifests")
                .join(format!("{}.txt.partial", output.binding_output));
            postconditions.push(format!(
                "mkdir -p {}; find {} -maxdepth 1 -mindepth 1 -name {} -print > {}; test -s {}; mv {} {}",
                shell_quote("/result/.qualification-glob-manifests"),
                shell_quote(&search_root.display().to_string()),
                shell_quote(&format!("{base}*")),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&partial_manifest.display().to_string()),
                shell_quote(&manifest.display().to_string()),
            ));
        }
    }
    let postconditions = postconditions.join("; ");
    let separator = if postconditions.is_empty() { "" } else { "; " };
    Ok(vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        format!(
            "set -eu; {command}; {postconditions}{separator}printf '%s\\n' '{{\"tool_completed\":1}}' > /result/qualification-metrics.json"
        ),
    ])
}

pub(super) fn glob_manifest_path(path: &Path) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(".glob-manifest.txt");
    PathBuf::from(value)
}

pub(super) fn split_catalog_command(value: &str, label: &str) -> Result<Vec<String>> {
    let argv = value.split_whitespace().map(ToString::to_string).collect::<Vec<_>>();
    if argv.is_empty() {
        bail!("catalog {label} command is empty");
    }
    Ok(argv)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
