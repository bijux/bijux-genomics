//! Container qualification image from admitted evidence.
use super::decisions::{missing_publication, security_decision};
use super::observations::{
    benchmark_observations, execution_observations, scientific_observations,
};
use super::QualificationContext;
use bijux_dna_containers::evidence::{ContainerCatalogTool, RunEvidenceRecord};
use bijux_dna_policies::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_identity_evidence,
    evaluate_source_identity, EvidenceDecision, EvidenceDimension, BUILD_EVIDENCE_RULE_V1,
    CATALOG_EVIDENCE_RULE_V1, FRESHNESS_EVIDENCE_RULE_V1, RELEASE_EVIDENCE_RULE_V1,
    RUNTIME_EVIDENCE_RULE_V1,
};
use bijux_dna_science::decision::evaluate_scientific_acceptance;
use std::collections::BTreeMap;

pub(super) fn redistributable_image_dimensions(
    tool: &ContainerCatalogTool,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
    image_sha256: &str,
) -> BTreeMap<EvidenceDimension, EvidenceDecision> {
    let stage = evaluate_execution_evidence(&execution_observations(runs), &tool.real_test_stage);
    let science =
        evaluate_scientific_acceptance(&scientific_observations(runs), &tool.real_test_stage);
    BTreeMap::from([
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
            evaluate_identity_evidence(BUILD_EVIDENCE_RULE_V1, "image_sha256", image_sha256),
        ),
        (
            EvidenceDimension::Runtime,
            evaluate_identity_evidence(
                RUNTIME_EVIDENCE_RULE_V1,
                "runtime_publication_sha256",
                context.runtime_sha256,
            ),
        ),
        (EvidenceDimension::Stage, stage.clone()),
        (EvidenceDimension::RealData, stage),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            evaluate_benchmark_evidence(
                &benchmark_observations(&tool.id, context.benchmark_reports),
                &context.chain.repository_commit,
                image_sha256,
            ),
        ),
        (EvidenceDimension::Security, security_decision(context, &tool.id, image_sha256)),
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
    ])
}
