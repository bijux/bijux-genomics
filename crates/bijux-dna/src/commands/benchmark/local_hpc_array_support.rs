use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcArrayResources {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) scratch_gb: u32,
}

pub(crate) fn time_limit_to_seconds(surface: &str, value: &str) -> Result<u64> {
    let parts = value.split(':').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(anyhow!(
            "{surface} encountered malformed time limit `{value}`"
        ));
    }
    let hours = parts[0].parse::<u64>().with_context(|| format!("parse hours from `{value}`"))?;
    let minutes =
        parts[1].parse::<u64>().with_context(|| format!("parse minutes from `{value}`"))?;
    let seconds =
        parts[2].parse::<u64>().with_context(|| format!("parse seconds from `{value}`"))?;
    Ok(hours * 3600 + minutes * 60 + seconds)
}

pub(crate) fn manifest_path_for_script(surface: &str, script_path: &Path) -> Result<PathBuf> {
    let parent = script_path
        .parent()
        .ok_or_else(|| anyhow!("{surface} output `{}` has no parent", script_path.display()))?;
    let stem = script_path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
        anyhow!(
            "{surface} output `{}` has no valid file stem",
            script_path.display()
        )
    })?;
    Ok(parent.join(format!("{stem}-manifest.json")))
}

pub(crate) fn shell_quote(value: &str) -> String {
    if value.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/' | ':' | '+' | '%')
    }) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
