use anyhow::{anyhow, Result};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolVersionCheck {
    pub tool: String,
    pub version: String,
    pub help_ok: bool,
}

/// # Errors
/// Returns an error when version output does not satisfy expected regex.
pub fn verify_tool_wrapper(
    tool: &str,
    version_output: &str,
    help_output: &str,
    expected_version_regex: &str,
) -> Result<ToolVersionCheck> {
    let regex = Regex::new(expected_version_regex)
        .map_err(|err| anyhow!("invalid regex for tool {tool}: {err}"))?;
    if !regex.is_match(version_output) {
        return Err(anyhow!(
            "tool {tool} version output does not match regex `{expected_version_regex}`"
        ));
    }
    let help_ok = help_output.contains("Usage")
        || help_output.contains("usage")
        || help_output.contains("Options")
        || help_output.contains("help");
    if !help_ok {
        return Err(anyhow!("tool {tool} help output missing usage markers"));
    }
    Ok(ToolVersionCheck {
        tool: tool.to_string(),
        version: version_output
            .lines()
            .next()
            .unwrap_or(version_output)
            .trim()
            .to_string(),
        help_ok,
    })
}
