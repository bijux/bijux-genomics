use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::source_scan::collect_rs_files;

pub(super) struct GuardrailSources {
    pub(super) src_dir: PathBuf,
    pub(super) files: Vec<PathBuf>,
}

pub(super) fn collect_sources(crate_root: &Path) -> Result<GuardrailSources> {
    let src_dir = crate_root.join("src");
    let files = collect_rs_files(&src_dir)?;
    Ok(GuardrailSources { src_dir, files })
}
