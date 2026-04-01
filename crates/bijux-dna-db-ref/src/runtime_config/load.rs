use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

pub(crate) fn load_toml<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<T>(&raw).with_context(|| format!("parse {}", path.display()))
}
