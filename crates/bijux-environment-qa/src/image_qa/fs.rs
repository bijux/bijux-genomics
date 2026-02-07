use std::path::PathBuf;

use anyhow::Result;
use uuid::Uuid;

pub(crate) fn temp_out_dir(stage: &str, tool: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir().join("bijux-image-qa").join(stage);
    bijux_infra::ensure_dir(&base)?;
    let path = base.join(format!("{tool}-{}", Uuid::new_v4()));
    bijux_infra::ensure_dir(&path)?;
    Ok(path)
}
