use super::paths::path_relative_to_repo;
use crate::evidence::registry::smoke::VcfRegistrySmokeRecord;
use anyhow::{bail, Result};
use std::path::Path;
pub(super) fn resolve_container_smoke_wrapper(
    repo_root: &Path,
    registry_row: &VcfRegistrySmokeRecord,
) -> Result<(String, String, String, String)> {
    let dockerfile = repo_root.join(&registry_row.dockerfile);
    if !registry_row.dockerfile.trim().is_empty() && dockerfile.is_file() {
        return Ok((
            "docker_container_smoke".to_string(),
            "docker-arm64".to_string(),
            format!("bijux-dna env smoke docker-arm64 {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &dockerfile),
        ));
    }
    let apptainer_def = repo_root.join(&registry_row.apptainer_def);
    if !registry_row.apptainer_def.trim().is_empty() && apptainer_def.is_file() {
        return Ok((
            "apptainer_container_smoke".to_string(),
            "apptainer".to_string(),
            format!("bijux-dna env smoke apptainer {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &apptainer_def),
        ));
    }

    bail!(
        "VCF local-container smoke could not resolve a checked-in container wrapper for retained tool `{}` (dockerfile=`{}`, apptainer_def=`{}`)",
        registry_row.tool_id,
        registry_row.dockerfile,
        registry_row.apptainer_def
    );
}
pub(super) fn local_smoke_command_name(stage_id: &str) -> Result<String> {
    let Some(suffix) = stage_id.strip_prefix("vcf.") else {
        bail!("VCF local-container smoke expected a `vcf.*` stage id, found `{stage_id}`");
    };
    Ok(format!("run-vcf-{}-smoke", suffix.replace('_', "-")))
}
