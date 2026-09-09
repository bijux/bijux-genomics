//! Container qualification decisions from admitted evidence.
use super::QualificationContext;
use bijux_dna_containers::evidence::ContainerCatalogTool;
use bijux_dna_policies::{
    ArtifactAvailability, ArtifactDelivery, EvidenceDecision, ProductSupport,
    SECURITY_EVIDENCE_RULE_V1, SUPPORT_DIMENSIONS_RULE_V2,
};
use std::collections::BTreeMap;

pub(super) fn product_support(
    support: bijux_dna_containers::evidence::ContainerProductSupport,
) -> ProductSupport {
    match support {
        bijux_dna_containers::evidence::ContainerProductSupport::Supported => {
            ProductSupport::Supported
        }
        bijux_dna_containers::evidence::ContainerProductSupport::Retired => ProductSupport::Retired,
        bijux_dna_containers::evidence::ContainerProductSupport::Excluded => {
            ProductSupport::Excluded
        }
    }
}

pub(super) fn artifact_delivery(
    delivery: bijux_dna_containers::evidence::ContainerArtifactDelivery,
) -> ArtifactDelivery {
    match delivery {
        bijux_dna_containers::evidence::ContainerArtifactDelivery::RedistributableImage => {
            ArtifactDelivery::RedistributableImage
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::ExternalArtifact => {
            ArtifactDelivery::ExternalArtifact
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::PrivateSiteBuild => {
            ArtifactDelivery::PrivateSiteBuild
        }
        bijux_dna_containers::evidence::ContainerArtifactDelivery::SiteModule => {
            ArtifactDelivery::SiteModule
        }
    }
}

pub(super) const fn artifact_availability_name(value: ArtifactAvailability) -> &'static str {
    match value {
        ArtifactAvailability::Admitted => "admitted",
        ArtifactAvailability::BlockedExternalAccess => "blocked_external_access",
        ArtifactAvailability::Unavailable => "unavailable",
    }
}

pub(super) fn security_decision(
    context: &QualificationContext<'_>,
    tool_id: &str,
    image_sha256: &str,
) -> EvidenceDecision {
    context.security_evidence.map_or_else(
        || {
            missing_publication(
                SECURITY_EVIDENCE_RULE_V1,
                "admitted_publication",
                "image_coverage",
                "current admitted security publication",
                "no admitted security publication covers this image",
            )
        },
        |security| security.tool_decision(tool_id, image_sha256),
    )
}

pub(super) fn not_applicable(tool: &ContainerCatalogTool) -> EvidenceDecision {
    EvidenceDecision::not_applicable(
        SUPPORT_DIMENSIONS_RULE_V2,
        BTreeMap::from([
            ("product_support".to_string(), tool.product_support.as_str().to_string()),
            ("artifact_delivery".to_string(), tool.artifact_delivery.as_str().to_string()),
        ]),
        BTreeMap::from([(
            "product_support".to_string(),
            "supported for qualification".to_string(),
        )]),
        "tool is retired or excluded from the product qualification population",
    )
}

pub(super) fn missing_publication(
    rule_version: &str,
    measured_key: &str,
    threshold_key: &str,
    threshold: &str,
    reason: &str,
) -> EvidenceDecision {
    EvidenceDecision::missing(
        rule_version,
        BTreeMap::from([(measured_key.to_string(), "missing".to_string())]),
        BTreeMap::from([(threshold_key.to_string(), threshold.to_string())]),
        reason,
    )
}
