//! Container qualification external from admitted evidence.
use super::decisions::{
    artifact_availability_name, artifact_delivery, missing_publication, security_decision,
};
use super::observations::{
    benchmark_observations, execution_observations, scientific_observations,
};
use super::QualificationContext;
use anyhow::Result;
use bijux_dna_containers::evidence::{ContainerCatalogTool, RunEvidenceRecord};
use bijux_dna_policies::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_identity_evidence,
    evaluate_source_identity, ArtifactAvailability, EvidenceDecision, EvidenceDimension,
    ProductSupport, PublicationDisposition, QualificationAssessment, ToolSecurityEvidence,
    BUILD_EVIDENCE_RULE_V1, CATALOG_EVIDENCE_RULE_V1, FRESHNESS_EVIDENCE_RULE_V1,
    RELEASE_EVIDENCE_RULE_V1, REQUIRED_EVIDENCE_DIMENSIONS, SECURITY_EVIDENCE_RULE_V1,
    SUPPORT_DIMENSIONS_RULE_V2,
};
use bijux_dna_science::decision::evaluate_scientific_acceptance;
use std::collections::BTreeMap;

pub(super) fn external_artifact_assessment(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
) -> Result<QualificationAssessment> {
    let evidence = context.security_evidence.and_then(|security| security.tool_evidence(&tool.id));
    let availability = evidence.map_or(ArtifactAvailability::Unavailable, |item| item.availability);
    if let Some(item) = evidence {
        validate_external_security_dimensions(tool, item)?;
    }
    let dimensions = match (availability, evidence) {
        (ArtifactAvailability::Admitted, Some(item)) => {
            admitted_external_artifact_dimensions(tool, runs, context, item)?
        }
        _ => unavailable_external_artifact_dimensions(tool, context, availability),
    };
    QualificationAssessment::v3(
        tool.id.clone(),
        ProductSupport::Supported,
        artifact_delivery(tool.artifact_delivery),
        tool.redistribution_permitted,
        availability,
        PublicationDisposition::Current,
        dimensions,
    )
}

pub(super) fn validate_external_security_dimensions(
    tool: &ContainerCatalogTool,
    evidence: &ToolSecurityEvidence,
) -> Result<()> {
    if evidence.product_support != ProductSupport::Supported
        || evidence.artifact_delivery != artifact_delivery(tool.artifact_delivery)
        || evidence.redistribution_permitted != tool.redistribution_permitted
    {
        anyhow::bail!("security evidence support dimensions differ from catalog for {}", tool.id);
    }
    Ok(())
}

pub(super) fn admitted_external_artifact_dimensions(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
    evidence: &ToolSecurityEvidence,
) -> Result<BTreeMap<EvidenceDimension, EvidenceDecision>> {
    let artifact_sha256 = evidence
        .artifact_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("admitted external tool lacks artifact identity"))?;
    let admission_sha256 = evidence
        .admission_evidence_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("admitted external tool lacks admission identity"))?;
    let execution =
        evaluate_execution_evidence(&execution_observations(runs), &tool.real_test_stage);
    let science =
        evaluate_scientific_acceptance(&scientific_observations(runs), &tool.real_test_stage);
    Ok(BTreeMap::from([
        (
            EvidenceDimension::Catalog,
            evaluate_identity_evidence(
                CATALOG_EVIDENCE_RULE_V1,
                "catalog_sha256",
                &context.chain.catalog_sha256,
            ),
        ),
        (
            EvidenceDimension::Source,
            evaluate_source_identity(
                &tool.source_sha256,
                &tool.pinned_commit,
                &bijux_dna_infra::hash_bytes_sha256(
                    format!("git:{}", tool.pinned_commit).as_bytes(),
                ),
            ),
        ),
        (
            EvidenceDimension::Build,
            evaluate_identity_evidence(
                BUILD_EVIDENCE_RULE_V1,
                "artifact_admission_sha256",
                admission_sha256,
            ),
        ),
        (EvidenceDimension::Runtime, execution.clone()),
        (EvidenceDimension::Stage, execution.clone()),
        (EvidenceDimension::RealData, execution),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            evaluate_benchmark_evidence(
                &benchmark_observations(&tool.id, context.benchmark_reports),
                &context.chain.repository_commit,
                artifact_sha256,
            ),
        ),
        (EvidenceDimension::Security, security_decision(context, &tool.id, artifact_sha256)),
        (
            EvidenceDimension::Freshness,
            evaluate_identity_evidence(
                FRESHNESS_EVIDENCE_RULE_V1,
                "bound_evidence_set_sha256",
                context.freshness_sha256,
            ),
        ),
        (
            EvidenceDimension::Release,
            missing_publication(
                RELEASE_EVIDENCE_RULE_V1,
                "immutable_publication",
                "evidence_set_coverage",
                "current immutable release publication",
                "no immutable release publication covers this evidence set",
            ),
        ),
    ]))
}

pub(super) fn unavailable_external_artifact_dimensions(
    tool: &ContainerCatalogTool,
    context: &QualificationContext<'_>,
    availability: ArtifactAvailability,
) -> BTreeMap<EvidenceDimension, EvidenceDecision> {
    REQUIRED_EVIDENCE_DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let decision = match dimension {
                EvidenceDimension::Catalog => evaluate_identity_evidence(
                    CATALOG_EVIDENCE_RULE_V1,
                    "catalog_sha256",
                    &context.chain.catalog_sha256,
                ),
                EvidenceDimension::Source => evaluate_source_identity(
                    &tool.source_sha256,
                    &tool.pinned_commit,
                    &bijux_dna_infra::hash_bytes_sha256(
                        format!("git:{}", tool.pinned_commit).as_bytes(),
                    ),
                ),
                EvidenceDimension::Security => context.security_evidence.map_or_else(
                    || {
                        missing_publication(
                            SECURITY_EVIDENCE_RULE_V1,
                            "admitted_publication",
                            "artifact_coverage",
                            "current admitted security publication",
                            "no admitted security publication covers this external artifact",
                        )
                    },
                    |security| security.tool_availability_decision(&tool.id),
                ),
                _ => EvidenceDecision::blocked(
                    SUPPORT_DIMENSIONS_RULE_V2,
                    BTreeMap::from([
                        ("product_support".to_string(), "supported".to_string()),
                        (
                            "artifact_delivery".to_string(),
                            tool.artifact_delivery.as_str().to_string(),
                        ),
                        (
                            "availability".to_string(),
                            artifact_availability_name(availability).to_string(),
                        ),
                    ]),
                    BTreeMap::from([(
                        "availability".to_string(),
                        "admitted checksum-bound artifact".to_string(),
                    )]),
                    "no admitted external artifact is present in the current qualification evidence chain",
                ),
            };
            (dimension, decision)
        })
        .collect()
}
