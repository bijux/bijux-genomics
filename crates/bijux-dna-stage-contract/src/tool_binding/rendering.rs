use anyhow::{anyhow, bail, Result};

use super::{ToolBinding, ToolBindingRenderContext};

/// Renders argv from a binding without process execution or tool-specific branches.
///
/// # Errors
///
/// Returns an error for malformed placeholders or values missing from the render context.
pub fn render_tool_command(
    binding: &impl ToolBinding,
    context: &ToolBindingRenderContext,
) -> Result<Vec<String>> {
    let mut command = Vec::new();
    for token in binding.command_template() {
        if token.contains("${inputs:") {
            let name = token
                .strip_prefix("${inputs:")
                .and_then(|value| value.strip_suffix('}'))
                .filter(|value| !value.is_empty() && !value.contains("${") && !value.contains('}'))
                .ok_or_else(|| {
                    anyhow!("multi-path input projection must occupy one complete command token")
                })?;
            let values = context
                .input_sets
                .get(name)
                .ok_or_else(|| anyhow!("binding render context has no input set `{name}`"))?;
            if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
                bail!("binding render context input set `{name}` requires non-empty paths");
            }
            command.extend(values.iter().cloned());
        } else {
            command.push(render_token(token, context)?);
        }
    }
    Ok(command)
}

fn render_token(token: &str, context: &ToolBindingRenderContext) -> Result<String> {
    let mut rendered = String::new();
    let mut remaining = token;
    while let Some(start) = remaining.find("${") {
        rendered.push_str(&remaining[..start]);
        let placeholder = &remaining[start + 2..];
        let end = placeholder
            .find('}')
            .ok_or_else(|| anyhow!("unterminated placeholder in `{token}`"))?;
        let key = &placeholder[..end];
        rendered.push_str(resolve_placeholder(key, context)?);
        remaining = &placeholder[end + 1..];
    }
    rendered.push_str(remaining);
    Ok(rendered)
}

fn resolve_placeholder<'a>(key: &str, context: &'a ToolBindingRenderContext) -> Result<&'a str> {
    if key == "executable" {
        if context.executable.trim().is_empty() {
            bail!("binding render context has no executable");
        }
        return Ok(&context.executable);
    }
    let (kind, name) =
        key.split_once(':').ok_or_else(|| anyhow!("invalid binding placeholder `${{{key}}}`"))?;
    let values = match kind {
        "input" => &context.inputs,
        "parameter" => &context.parameters,
        "output" => &context.outputs,
        _ => bail!("unknown binding placeholder kind `{kind}`"),
    };
    values
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| anyhow!("binding render context has no {kind} `{name}`"))
}
