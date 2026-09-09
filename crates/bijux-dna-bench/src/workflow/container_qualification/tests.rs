use std::collections::BTreeMap;

use super::{apply_release_evidence, freshness_identity, load_release_publication};
use bijux_dna_policies::{
    admit_release_evidence, derive_release_evidence, qualification_sources_sha256,
    ArtifactAvailability, ArtifactDelivery, EvidenceDecision, EvidenceDimension, EvidenceVerdict,
    ProductSupport, PublicationDisposition, QualificationAssessment, QualificationClaim,
    QualificationEvidenceSource, ReleaseEvidenceDerivationRequest, REQUIRED_EVIDENCE_DIMENSIONS,
};

const REPOSITORY_COMMIT: &str = "1111111111111111111111111111111111111111";
const CATALOG_SHA256: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const PUBLICATION_SHA256: &str = "3333333333333333333333333333333333333333333333333333333333333333";

#[test]
fn freshness_identity_covers_benchmark_sources() -> anyhow::Result<()> {
    let mut sources = BTreeMap::from([(
        "benchmark:run-1001".to_string(),
        QualificationEvidenceSource {
            uri: "repository://benchmarks/reports/run-1001.json".to_string(),
            sha256: "a".repeat(64),
        },
    )]);
    let original = freshness_identity(&"b".repeat(40), &sources)?;
    let source = sources
        .get_mut("benchmark:run-1001")
        .ok_or_else(|| anyhow::anyhow!("benchmark source fixture is absent"))?;
    source.sha256 = "c".repeat(64);
    assert_ne!(original, freshness_identity(&"b".repeat(40), &sources)?);
    Ok(())
}

#[test]
fn admitted_release_evidence_closes_the_release_dimension() -> anyhow::Result<()> {
    let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let measured = BTreeMap::from([("state".to_string(), "present".to_string())]);
            let thresholds = BTreeMap::from([("state".to_string(), "present".to_string())]);
            let decision = if dimension == EvidenceDimension::Release {
                EvidenceDecision::missing(
                    "qualification.release.fixture.v1",
                    measured,
                    thresholds,
                    "release publication is absent",
                )
            } else {
                EvidenceDecision::passed(
                    "qualification.release.fixture.v1",
                    measured,
                    thresholds,
                    "a".repeat(64),
                    "fixture evidence passes",
                )
            };
            (dimension, decision)
        })
        .collect();
    let assessment = QualificationAssessment::v3(
        "samtools".to_string(),
        ProductSupport::Supported,
        ArtifactDelivery::RedistributableImage,
        true,
        ArtifactAvailability::Admitted,
        PublicationDisposition::Current,
        dimensions,
    )?;
    let sources = BTreeMap::from([(
        "catalog".to_string(),
        QualificationEvidenceSource {
            uri: "repository://containers/contracts/catalog.toml".to_string(),
            sha256: CATALOG_SHA256.to_string(),
        },
    )]);
    let sources_sha256 = qualification_sources_sha256(&sources)?;
    let assessments = vec![assessment];
    let publication = derive_release_evidence(&ReleaseEvidenceDerivationRequest {
        released_at_utc: "2026-08-10T12:00:00Z",
        repository_commit: REPOSITORY_COMMIT,
        catalog_sha256: CATALOG_SHA256,
        qualification_sources_sha256: &sources_sha256,
        assessments: &assessments,
    })?;
    let admitted = admit_release_evidence(
        &publication,
        PUBLICATION_SHA256,
        REPOSITORY_COMMIT,
        CATALOG_SHA256,
        &sources_sha256,
        &assessments,
    )?;
    let closed = apply_release_evidence(assessments, &admitted)?;
    assert_eq!(closed[0].dimensions[&EvidenceDimension::Release].verdict, EvidenceVerdict::Passed);
    assert!(closed[0].claims.contains(&QualificationClaim::Qualified));
    Ok(())
}

#[test]
fn release_publication_source_is_content_addressed_not_path_addressed() -> anyhow::Result<()> {
    let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let decision = if dimension == EvidenceDimension::Release {
                EvidenceDecision::missing(
                    "qualification.release.fixture.v1",
                    BTreeMap::from([("state".to_string(), "missing".to_string())]),
                    BTreeMap::from([("state".to_string(), "present".to_string())]),
                    "release publication is absent",
                )
            } else {
                EvidenceDecision::passed(
                    "qualification.release.fixture.v1",
                    BTreeMap::from([("state".to_string(), "present".to_string())]),
                    BTreeMap::from([("state".to_string(), "present".to_string())]),
                    "a".repeat(64),
                    "fixture evidence passes",
                )
            };
            (dimension, decision)
        })
        .collect();
    let assessments = vec![QualificationAssessment::v3(
        "samtools".to_string(),
        ProductSupport::Supported,
        ArtifactDelivery::RedistributableImage,
        true,
        ArtifactAvailability::Admitted,
        PublicationDisposition::Current,
        dimensions,
    )?];
    let publication = derive_release_evidence(&ReleaseEvidenceDerivationRequest {
        released_at_utc: "2026-08-10T12:00:00Z",
        repository_commit: REPOSITORY_COMMIT,
        catalog_sha256: CATALOG_SHA256,
        qualification_sources_sha256: &qualification_sources_sha256(&BTreeMap::from([(
            "catalog".to_string(),
            QualificationEvidenceSource {
                uri: "repository://containers/contracts/catalog.toml".to_string(),
                sha256: CATALOG_SHA256.to_string(),
            },
        )]))?,
        assessments: &assessments,
    })?;
    let bytes = serde_json::to_vec(&publication)?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("governed-release.json");
    std::fs::write(&path, &bytes)?;

    let (loaded, source) = load_release_publication(&path)?;
    let expected_sha256 = bijux_dna_infra::hash_bytes_sha256(&bytes);
    assert_eq!(loaded, publication);
    assert_eq!(source.sha256, expected_sha256);
    assert_eq!(source.uri, format!("evidence://release/{expected_sha256}"));
    assert!(!source.uri.contains(&path.to_string_lossy().to_string()));
    Ok(())
}
