use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use super::inventory::ImageInventoryScan;
use super::model::{
    ContainerDetachedImage, ContainerImageCandidate, ContainerImageInventoryPublication,
    ContainerImageInventorySummary, ContainerImageInventoryTool, ContainerImageState,
    ContainerStagedImage, ImageInventoryPublicationIdentity, RuntimeVerificationPublication,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3,
};
use super::tool_contract::{ContainerArtifactDelivery, ContainerProductSupport};
use super::validation::{
    validate_image_inventory_publication, validate_image_inventory_scan,
    validate_runtime_verification_publication,
};

/// Load and validate one complete image-inventory scan.
///
/// # Errors
/// Returns an error when the scan cannot be read, parsed, or admitted.
pub fn load_image_inventory_scan(path: &Path) -> Result<ImageInventoryScan> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read image-inventory scan {}", path.display()))?;
    let scan = serde_json::from_str(&raw)
        .with_context(|| format!("parse image-inventory scan {}", path.display()))?;
    validate_image_inventory_scan(&scan)
        .with_context(|| format!("validate image-inventory scan {}", path.display()))?;
    Ok(scan)
}

/// Load and validate one portable image-inventory publication.
///
/// # Errors
/// Returns an error when the publication cannot be read, parsed, or admitted.
pub fn load_image_inventory_publication(path: &Path) -> Result<ContainerImageInventoryPublication> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read image-inventory publication {}", path.display()))?;
    let schema_version = serde_json::from_str::<serde_json::Value>(&raw)
        .with_context(|| format!("parse image-inventory publication {}", path.display()))?
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let publication = match schema_version.as_str() {
        CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3 => serde_json::from_str(&raw)
            .with_context(|| format!("parse image-inventory publication {}", path.display()))?,
        CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1
        | CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2 => {
            normalize_legacy_inventory(serde_json::from_str(&raw).with_context(|| {
                format!("parse historical image-inventory publication {}", path.display())
            })?)
        }
        _ => anyhow::bail!("unsupported image-inventory publication schema: {schema_version}"),
    };
    validate_image_inventory_publication(&publication)
        .with_context(|| format!("validate image-inventory publication {}", path.display()))?;
    Ok(publication)
}

#[derive(Deserialize)]
struct LegacyImageInventoryPublication {
    #[serde(rename = "schema_version")]
    _schema_version: String,
    catalog_sha256: String,
    repository_commit: String,
    publication: ImageInventoryPublicationIdentity,
    summary: ContainerImageInventorySummary,
    tools: Vec<LegacyImageInventoryTool>,
    #[serde(default)]
    historical_images: Vec<ContainerDetachedImage>,
    #[serde(default, alias = "unclassified_images")]
    unknown_images: Vec<ContainerDetachedImage>,
    #[serde(default)]
    staged_images: Vec<ContainerStagedImage>,
}

#[derive(Deserialize)]
struct LegacyImageInventoryTool {
    tool: String,
    version: String,
    support_disposition: LegacySupportDisposition,
    apptainer_supported: bool,
    state: LegacyImageState,
    selected_image_sha256: Option<String>,
    #[serde(default)]
    candidates: Vec<ContainerImageCandidate>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacySupportDisposition {
    Retained,
    Unsupported,
    Retired,
    Excluded,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacyImageState {
    Current,
    Historical,
    Ambiguous,
    Missing,
    Unsupported,
}

fn normalize_legacy_inventory(
    legacy: LegacyImageInventoryPublication,
) -> ContainerImageInventoryPublication {
    let mut summary = legacy.summary;
    if let Some(count) = summary.states.remove("unsupported") {
        *summary.states.entry("not_applicable".to_string()).or_insert(0) += count;
    }
    ContainerImageInventoryPublication {
        schema_version: CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V3.to_string(),
        catalog_sha256: legacy.catalog_sha256,
        repository_commit: legacy.repository_commit,
        publication: legacy.publication,
        summary,
        tools: legacy.tools.into_iter().map(normalize_legacy_tool).collect(),
        historical_images: legacy.historical_images,
        unknown_images: legacy.unknown_images,
        staged_images: legacy.staged_images,
    }
}

fn normalize_legacy_tool(legacy: LegacyImageInventoryTool) -> ContainerImageInventoryTool {
    let (product_support, artifact_delivery, redistribution_permitted) = match legacy
        .support_disposition
    {
        LegacySupportDisposition::Retained => (
            ContainerProductSupport::Supported,
            ContainerArtifactDelivery::RedistributableImage,
            true,
        ),
        LegacySupportDisposition::Unsupported => {
            (ContainerProductSupport::Supported, ContainerArtifactDelivery::ExternalArtifact, false)
        }
        LegacySupportDisposition::Retired => {
            (ContainerProductSupport::Retired, ContainerArtifactDelivery::ExternalArtifact, false)
        }
        LegacySupportDisposition::Excluded => {
            (ContainerProductSupport::Excluded, ContainerArtifactDelivery::ExternalArtifact, false)
        }
    };
    let state = match legacy.state {
        LegacyImageState::Current => ContainerImageState::Current,
        LegacyImageState::Historical => ContainerImageState::Historical,
        LegacyImageState::Ambiguous => ContainerImageState::Ambiguous,
        LegacyImageState::Missing => ContainerImageState::Missing,
        LegacyImageState::Unsupported => ContainerImageState::NotApplicable,
    };
    ContainerImageInventoryTool {
        tool: legacy.tool,
        version: legacy.version,
        product_support,
        artifact_delivery,
        redistribution_permitted,
        apptainer_supported: legacy.apptainer_supported,
        state,
        selected_image_sha256: legacy.selected_image_sha256,
        candidates: legacy.candidates,
    }
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
