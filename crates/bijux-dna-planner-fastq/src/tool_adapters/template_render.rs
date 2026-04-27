use anyhow::{anyhow, Result};

/// # Errors
/// Returns an error when the rendered command still contains unresolved
/// placeholder tokens after applying the provided bindings.
pub fn render_command_template(
    template: &[String],
    bindings: &[(&str, Option<String>)],
) -> Result<Vec<String>> {
    let normalized = bindings
        .iter()
        .map(|(key, value)| (format!("{{{{{key}}}}}"), value.clone()))
        .collect::<Vec<_>>();
    let mut rendered = Vec::with_capacity(template.len());
    for token in template {
        let mut value = token.clone();
        for (placeholder, replacement) in &normalized {
            if let Some(replacement) = replacement {
                value = value.replace(placeholder, replacement);
            }
        }
        if value.contains("{{") || value.contains("}}") {
            return Err(anyhow!("unresolved command template token: {value}"));
        }
        rendered.push(value);
    }
    Ok(rendered)
}
