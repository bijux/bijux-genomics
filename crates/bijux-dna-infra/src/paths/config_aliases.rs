pub(super) fn remap_runtime_profile(relative: &str) -> Option<String> {
    if let Some(profile) = relative.strip_prefix("runtime/profile.") {
        if let Some(profile) = profile.strip_suffix(".toml") {
            return Some(format!("runtime/profiles/{profile}.toml"));
        }
    }
    if let Some(profile) = relative.strip_prefix("runtime/profile_") {
        if let Some(profile) = profile.strip_suffix(".toml") {
            return Some(format!("runtime/profiles/{profile}.toml"));
        }
    }
    None
}

pub(super) fn remap_ci_registry(relative: &str) -> Option<&'static str> {
    match relative {
        "ci/tool_registry.toml" => Some("ci/registry/tool_registry.toml"),
        "ci/tool_registry_experimental.toml" => Some("ci/registry/tool_registry_experimental.toml"),
        "ci/tool_registry_vcf.toml" => Some("ci/registry/tool_registry_vcf.toml"),
        "ci/tool_registry.lock.sha256" | "ci/tool_registry_lock.sha256" => {
            Some("ci/registry/tool_registry_lock.sha256")
        }
        "ci/domains.toml" => Some("ci/registry/domains.toml"),
        "ci/stages.toml" => Some("ci/stages/stages.toml"),
        "ci/stages_vcf.toml" => Some("ci/stages/stages_vcf.toml"),
        "ci/required_tools.toml" => Some("ci/tools/required_tools.toml"),
        "ci/required_tools_vcf.toml" => Some("ci/tools/required_tools_vcf.toml"),
        "ci/images.toml" => Some("ci/tools/images.toml"),
        "ci/param_registry.toml" => Some("ci/params/param_registry.toml"),
        "ci/param_registry_vcf.toml" => Some("ci/params/param_registry_vcf.toml"),
        _ => None,
    }
}
