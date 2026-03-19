use anyhow::{Context, Result};
use serde::Deserialize;

use crate::runtime::workspace::Workspace;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupportedScriptCatalog {
    #[serde(rename = "script")]
    pub entries: Vec<SupportedScript>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SupportedScript {
    #[serde(rename = "id")]
    pub _id: String,
    pub path: String,
}

pub(crate) fn load_supported_scripts(workspace: &Workspace) -> Result<Vec<SupportedScript>> {
    let spec_path = workspace.path(&["scr", "ipts/SUPPORTED.toml"].concat());
    if !spec_path.is_file() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("read {}", spec_path.display()))?;
    let parsed: SupportedScriptCatalog =
        toml::from_str(&raw).with_context(|| format!("parse {}", spec_path.display()))?;
    Ok(parsed.entries)
}
