use anyhow::Context;
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::Path;
pub(in crate::bam::assignment) fn load_governed_toml<T>(path: &Path, label: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {label} {}", path.display()))
}
