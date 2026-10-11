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
    binding.command_template().iter().map(|token| render_token(token, context)).collect()
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
