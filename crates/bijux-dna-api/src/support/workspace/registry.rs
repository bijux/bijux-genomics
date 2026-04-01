use std::path::PathBuf;

use anyhow::{anyhow, Result};

fn workspace_root() -> Result<PathBuf> {
    crate::support::workspace::resolve_repo_root()
}

pub fn workspace_domain_dir() -> Result<PathBuf> {
    Ok(workspace_root()?.join("domain"))
}

pub fn load_registry(
    source_path: &std::path::Path,
) -> Result<bijux_dna_core::contract::ToolRegistry> {
    let registry_path = if source_path.is_dir()
        && source_path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "domain")
    {
        bijux_dna_infra::configs_file(
            source_path.parent().unwrap_or(source_path),
            "ci/registry/tool_registry.toml",
        )
    } else {
        source_path.to_path_buf()
    };
    bijux_dna_runtime::manifests::load_manifests(&registry_path)
        .map_err(|err| anyhow!("manifest validation failed: {err}"))
}

pub fn load_workspace_registry() -> Result<bijux_dna_core::contract::ToolRegistry> {
    load_registry(&workspace_domain_dir()?)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{load_workspace_registry, workspace_domain_dir};

    #[test]
    fn workspace_domain_dir_resolves_repo_domain_tree() {
        let domain_dir = workspace_domain_dir().expect("workspace domain dir");
        assert!(
            domain_dir.join("fastq").is_dir(),
            "workspace domain dir must resolve the repo domain tree"
        );
        load_workspace_registry().unwrap_or_else(|err| {
            panic!("workspace registry should load from resolved domain dir: {err}")
        });
    }
}
