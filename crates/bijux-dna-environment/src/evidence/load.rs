use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{ContainerImageInventoryPublication, RuntimeVerificationPublication};
use super::validation::{
    validate_image_inventory_publication, validate_runtime_verification_publication,
};

/// Load and validate one portable image-inventory publication.
///
/// # Errors
/// Returns an error when the publication cannot be read, parsed, or admitted.
pub fn load_image_inventory_publication(path: &Path) -> Result<ContainerImageInventoryPublication> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read image-inventory publication {}", path.display()))?;
    let publication = serde_json::from_str(&raw)
        .with_context(|| format!("parse image-inventory publication {}", path.display()))?;
    validate_image_inventory_publication(&publication)
        .with_context(|| format!("validate image-inventory publication {}", path.display()))?;
    Ok(publication)
}

/// Load and validate one portable runtime-verification publication.
///
/// # Errors
/// Returns an error when the publication cannot be read, parsed, or admitted.
pub fn load_runtime_verification_publication(
    path: &Path,
) -> Result<RuntimeVerificationPublication> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read runtime-verification publication {}", path.display()))?;
    let publication = serde_json::from_str(&raw)
        .with_context(|| format!("parse runtime-verification publication {}", path.display()))?;
    validate_runtime_verification_publication(&publication)
        .with_context(|| format!("validate runtime-verification publication {}", path.display()))?;
    Ok(publication)
}
