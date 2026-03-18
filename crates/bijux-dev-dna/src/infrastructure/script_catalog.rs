use anyhow::{Context, Result};
use serde::Deserialize;

use crate::infrastructure::workspace::Workspace;

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
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(default)]
    pub network_allowed: bool,
    #[serde(default)]
    pub ci_allowed: bool,
}

pub(crate) fn load_supported_scripts(workspace: &Workspace) -> Result<Vec<SupportedScript>> {
    let spec_path = workspace.path("scripts/SUPPORTED.toml");
    let raw = std::fs::read_to_string(&spec_path)
        .with_context(|| format!("read {}", spec_path.display()))?;
    let parsed: SupportedScriptCatalog =
        toml::from_str(&raw).with_context(|| format!("parse {}", spec_path.display()))?;
    Ok(parsed.entries)
}
