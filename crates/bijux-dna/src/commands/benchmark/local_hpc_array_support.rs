use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::bench::BenchmarkLocalHpcArrayResources;

pub(crate) type LocalHpcArrayResources = BenchmarkLocalHpcArrayResources;

pub(crate) fn manifest_path_for_script(surface: &str, script_path: &Path) -> Result<PathBuf> {
    let parent = script_path
        .parent()
        .ok_or_else(|| anyhow!("{surface} output `{}` has no parent", script_path.display()))?;
    let stem = script_path.file_stem().and_then(|value| value.to_str()).ok_or_else(|| {
        anyhow!("{surface} output `{}` has no valid file stem", script_path.display())
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
