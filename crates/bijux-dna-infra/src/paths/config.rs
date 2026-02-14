use std::path::{Path, PathBuf};

#[must_use]
pub fn configs_dir(root: &Path) -> PathBuf {
    root.join("configs")
}

#[must_use]
pub fn configs_file(root: &Path, relative: &str) -> PathBuf {
    let normalized = if let Some(profile) = relative.strip_prefix("runtime/profile.") {
        if let Some(profile) = profile.strip_suffix(".toml") {
            return configs_dir(root).join(format!("runtime/profiles/{profile}.toml"));
        }
        relative
    } else if let Some(profile) = relative.strip_prefix("runtime/profile_") {
        if let Some(profile) = profile.strip_suffix(".toml") {
            return configs_dir(root).join(format!("runtime/profiles/{profile}.toml"));
        }
        relative
    } else {
        match relative {
            "ci/tool_registry.toml" => "ci/registry/tool_registry.toml",
            "ci/tool_registry_experimental.toml" => "ci/registry/tool_registry_experimental.toml",
            "ci/tool_registry_vcf.toml" => "ci/registry/tool_registry_vcf.toml",
            "ci/tool_registry.lock.sha256" | "ci/tool_registry_lock.sha256" => {
                "ci/registry/tool_registry_lock.sha256"
            }
            "ci/domains.toml" => "ci/registry/domains.toml",
            "ci/stages.toml" => "ci/stages/stages.toml",
            "ci/stages_vcf.toml" => "ci/stages/stages_vcf.toml",
            "ci/required_tools.toml" => "ci/tools/required_tools.toml",
            "ci/required_tools_vcf.toml" => "ci/tools/required_tools_vcf.toml",
            "ci/images.toml" => "ci/tools/images.toml",
            "ci/param_registry.toml" => "ci/params/param_registry.toml",
            "ci/param_registry_vcf.toml" => "ci/params/param_registry_vcf.toml",
            _ => relative,
        }
    };
    configs_dir(root).join(normalized)
}
