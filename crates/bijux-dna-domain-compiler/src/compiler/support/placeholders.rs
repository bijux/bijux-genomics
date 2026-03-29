use super::*;

fn has_placeholder_token(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("todo") || lower.contains("tbd") || lower.contains("placeholder")
}

pub(crate) fn has_supported_placeholder_forbidden_token(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    has_placeholder_token(raw) || lower.contains("sha256:dummy") || lower.contains("0.0.0")
}

pub(crate) fn placeholders_allowed(status: &str) -> bool {
    status == "planned"
}

pub(crate) fn ensure_no_placeholders_in_active_config(name: &str, rendered: &str) -> Result<()> {
    if has_supported_placeholder_forbidden_token(rendered) {
        bail!(
            "generated {name} contains placeholder token (todo/tbd/placeholder/sha256:dummy/0.0.0)"
        );
    }
    Ok(())
}
