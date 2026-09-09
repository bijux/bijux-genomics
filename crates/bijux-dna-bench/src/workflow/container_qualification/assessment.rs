//! Owner: benchmark qualification composition.
//! Container qualification assessment from admitted evidence.
use super::decisions::{artifact_delivery, not_applicable, product_support};
use super::external::external_artifact_assessment;
use super::image::redistributable_image_dimensions;
use super::QualificationContext;
use anyhow::Result;
use bijux_dna_containers::evidence::{
    ContainerCatalogTool, ContainerImageInventoryTool, RunEvidenceRecord,
};
use bijux_dna_policies::{
    ArtifactAvailability, ArtifactDelivery, ProductSupport, PublicationDisposition,
    QualificationAssessment, REQUIRED_EVIDENCE_DIMENSIONS,
};

pub(super) fn derive_tool_assessment(
    tool: &ContainerCatalogTool,
    inventory: Option<&ContainerImageInventoryTool>,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
) -> Result<QualificationAssessment> {
    if !tool.is_product_supported() {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| (dimension, not_applicable(tool)))
            .collect();
        return QualificationAssessment::v3(
            tool.id.clone(),
            product_support(tool.product_support),
            artifact_delivery(tool.artifact_delivery),
            tool.redistribution_permitted,
            ArtifactAvailability::Unavailable,
            PublicationDisposition::Current,
            dimensions,
        );
    }

    if !tool.is_redistributable_image() {
        return external_artifact_assessment(tool, runs, context);
    }

    let inventory = inventory.ok_or_else(|| {
        anyhow::anyhow!("qualification inventory omits retained tool {}", tool.id)
    })?;
    let image_sha256 = inventory
        .selected_image_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("qualification inventory has no image for {}", tool.id))?;
    let dimensions = redistributable_image_dimensions(tool, runs, context, image_sha256);
    QualificationAssessment::v3(
        tool.id.clone(),
        ProductSupport::Supported,
        ArtifactDelivery::RedistributableImage,
        tool.redistribution_permitted,
        ArtifactAvailability::Admitted,
        PublicationDisposition::Current,
        dimensions,
    )
}
