use std::path::{Component, Path};

use anyhow::{bail, Result};

use crate::catalog::CatalogFileEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogFileMaterializationArea {
    Raw,
    Derived,
}

impl CatalogFileMaterializationArea {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Derived => "derived",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogFileMaterializationPlan {
    pub area: CatalogFileMaterializationArea,
    pub relative_path: String,
}

/// Validate one catalog file's delivery contract and choose its immutable cache area.
///
/// Source downloads belong under `raw/`. Tool-derived artifacts belong under
/// `derived/` and must be admitted from checksum-verified bytes rather than
/// treated as network downloads.
///
/// # Errors
/// Returns an error when the catalog entry has an unsafe path or an internally
/// inconsistent source/derivation contract.
pub fn plan_panel_file_materialization(
    file: &CatalogFileEntry,
) -> Result<CatalogFileMaterializationPlan> {
    validate_relative_path(&file.path)?;
    validate_sha256(&file.checksum_sha256, "catalog file")?;
    if file.size_bytes == 0 {
        bail!("catalog file `{}` must declare a positive size", file.name);
    }

    let area = if let Some(derivation) = &file.derivation {
        if !file.url.trim().is_empty() {
            bail!("derived catalog file `{}` must not declare a download URL", file.name);
        }
        if derivation.source_file_name.trim().is_empty()
            || derivation.tool_id.trim().is_empty()
            || derivation.tool_version.trim().is_empty()
        {
            bail!("derived catalog file `{}` has incomplete provenance", file.name);
        }
        validate_sha256(&derivation.tool_artifact_sha256, "derivation tool artifact")?;
        if derivation.argv.first().map(String::as_str) != Some(derivation.tool_id.as_str()) {
            bail!(
                "derived catalog file `{}` argv must begin with tool `{}`",
                file.name,
                derivation.tool_id
            );
        }
        CatalogFileMaterializationArea::Derived
    } else {
        if file.url.trim().is_empty() {
            bail!("source catalog file `{}` must declare a download URL", file.name);
        }
        CatalogFileMaterializationArea::Raw
    };

    Ok(CatalogFileMaterializationPlan { area, relative_path: file.path.clone() })
}

fn validate_relative_path(raw: &str) -> Result<()> {
    let path = Path::new(raw);
    if raw.trim().is_empty()
        || path.is_absolute()
        || path.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("catalog file path must be a safe relative path: `{raw}`");
    }
    Ok(())
}

fn validate_sha256(raw: &str, label: &str) -> Result<()> {
    if raw.len() != 64
        || !raw.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} sha256 must be 64 lowercase hexadecimal characters");
    }
    Ok(())
}
