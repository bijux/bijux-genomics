use std::path::Path;

use regex::Regex;

use crate::resolve::EnvError;

/// Extract expected version from Dockerfile ARG lines for a given tool.
///
/// # Errors
/// Returns an error if the Dockerfile is missing or no version ARG is found.
pub(super) fn extract_version_from_dockerfile(
    dockerfile: &Path,
    tool: &str,
) -> Result<String, EnvError> {
    let tool = tool.trim();
    if tool.is_empty() {
        return Err(EnvError::Dockerfile("tool name is empty".to_string()));
    }
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
    let mut names = vec![format!("VERSION_{canonical}"), format!("{canonical}_VERSION")];
    if canonical == "TRIM_GALORE" {
        names.push("TRIM_GALORE".to_string());
    }
    names.sort();
    names.dedup();
    names
}
