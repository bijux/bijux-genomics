fn tool_supports_polyx(tool_id: &str) -> bool {
    matches!(tool_id, "fastp")
}

#[must_use]
pub fn polyx_unsupported_warning(
    tool_id: &str,
    polyx_bank: Option<&serde_json::Value>,
    explicit: bool,
) -> Option<String> {
    if explicit && polyx_bank.is_some() && !tool_supports_polyx(tool_id) {
        return Some(format!(
            "warning: polyx preset requested but tool '{tool_id}' does not advertise polyX support"
        ));
    }
    None
}
