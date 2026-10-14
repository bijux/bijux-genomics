use anyhow::{anyhow, bail, Result};

use super::{ToolBinding, ToolBindingRenderContext};

/// Renders argv from a binding without process execution or tool-specific branches.
///
/// Scalar values within single-quoted literals of the canonical shell command forms retain
/// their literal bytes, including apostrophes. Other argument tokens retain raw projection;
/// custom shell grammars are not interpreted here.
///
/// # Errors
///
/// Returns an error for malformed placeholders or values missing from the render context.
pub fn render_tool_command(
    binding: &impl ToolBinding,
    context: &ToolBindingRenderContext,
) -> Result<Vec<String>> {
    let mut command = Vec::new();
    let script_index = shell_script_index(binding.command_template());
    for (index, token) in binding.command_template().iter().enumerate() {
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
            let kind = if script_index == Some(index) {
                TokenKind::ShellScript
            } else {
                TokenKind::Argument
            };
            command.push(render_token(token, context, kind)?);
        }
    }
    Ok(command)
}

#[derive(Clone, Copy)]
enum TokenKind {
    Argument,
    ShellScript,
}

fn shell_script_index(template: &[String]) -> Option<usize> {
    match template {
        [shell, flag, _, ..]
            if (shell == "/bin/sh" && flag == "-c") || (shell == "/bin/bash" && flag == "-lc") =>
        {
            Some(2)
        }
        [shell, option, setting, flag, _, ..]
            if shell == "/bin/bash" && option == "-o" && setting == "pipefail" && flag == "-c" =>
        {
            Some(4)
        }
        _ => None,
    }
}

#[derive(Default)]
enum Quote {
    #[default]
    Unquoted,
    Single,
    Double,
}

#[derive(Default)]
struct ShellLiteral {
    quote: Quote,
    escaped: bool,
}

impl ShellLiteral {
    fn observe(&mut self, literal: &str) {
        for character in literal.chars() {
            if self.escaped {
                self.escaped = false;
                continue;
            }
            match (&self.quote, character) {
                (Quote::Unquoted, '\'') => self.quote = Quote::Single,
                (Quote::Single, '\'') | (Quote::Double, '"') => self.quote = Quote::Unquoted,
                (Quote::Unquoted, '"') => self.quote = Quote::Double,
                (Quote::Unquoted | Quote::Double, '\\') => self.escaped = true,
                _ => {}
            }
        }
    }
}

fn render_token(
    token: &str,
    context: &ToolBindingRenderContext,
    kind: TokenKind,
) -> Result<String> {
    let mut rendered = String::new();
    let mut remaining = token;
    let mut literal = ShellLiteral::default();
    while let Some(start) = remaining.find("${") {
        let prefix = &remaining[..start];
        rendered.push_str(prefix);
        literal.observe(prefix);
        let placeholder = &remaining[start + 2..];
        let end = placeholder
            .find('}')
            .ok_or_else(|| anyhow!("unterminated placeholder in `{token}`"))?;
        let key = &placeholder[..end];
        let value = resolve_placeholder(key, context)?;
        if matches!(kind, TokenKind::ShellScript) && matches!(literal.quote, Quote::Single) {
            // Leave the surrounding literal open while quoting only embedded apostrophes.
            // Projected values never become template syntax or change the quote observation.
            rendered.push_str(&value.replace('\'', "'\\''"));
        } else {
            rendered.push_str(value);
        }
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
