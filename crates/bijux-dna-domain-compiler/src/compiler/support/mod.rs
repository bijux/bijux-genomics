use super::*;

mod placeholders;
mod render;
mod repository;
mod status;
mod tooling;

pub(super) use placeholders::{
    ensure_no_placeholders_in_active_config, has_supported_placeholder_forbidden_token,
    placeholders_allowed,
};
pub(super) use render::{encode_f64_map, encode_threshold_map, generated_header, toml_array};
pub(super) use repository::{domain_content_hash, git_head_commit};
pub(super) use status::{
    ensure_status, is_tool_meaningful_in_domain, is_umbrella_stage, scope_active,
};
pub(super) use tooling::{
    default_healthcheck_cmd, default_version_regex, infer_tool_role, parse_container_ref,
    parse_version_from_recipe, read_text_if_exists, required_tool_roles_for_stage,
    resolve_tool_citation, resolve_tool_upstream, resolve_upstream_pin, tool_pin_override,
    tool_version_override, validate_tool_output_subset,
};

pub(super) fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn is_unspecified(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unspecified")
}
