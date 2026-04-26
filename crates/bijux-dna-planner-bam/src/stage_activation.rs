use anyhow::{anyhow, Result};

fn stage_status(stage_id: &str) -> Option<String> {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent()?.parent()?;
    let path = bijux_dna_infra::configs_file(repo_root, "ci/stages/stages.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = raw.parse::<toml::Value>().ok()?;
    let entries = parsed.get("stages")?.as_array()?;
    entries.iter().find_map(|entry| {
        let id = entry.get("id").and_then(toml::Value::as_str)?;
        if id == stage_id {
            entry.get("status").and_then(toml::Value::as_str).map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

/// # Errors
/// Returns an error if the stage is outside the current planning scope.
pub fn enforce(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned") | Some("out_of_scope") if allow_planned => Ok(()),
        Some("planned") | Some("out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
}
