use std::collections::BTreeMap;

use anyhow::{ensure, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    validate_release_evidence_publication, ContainerReleaseEvidencePublication,
    ToolReleaseEvidence, CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1,
};
use crate::qualification::{
    EvidenceDecision, EvidenceDimension, EvidenceVerdict, ProductSupport, QualificationAssessment,
    QualificationEvidenceSource,
};

const RELEASE_POLICY_SPECIFICATION: &str = concat!(
    "bijux.container.release_policy.v1\n",
    "identity=repository_commit+catalog_sha256+qualification_sources_sha256\n",
    "tool_identity=non_release_qualification_dimensions\n",
    "supported_admitted=all_non_release_dimensions_pass\n",
    "supported_unadmitted=blocked\n",
    "unsupported=not_applicable\n",
);

pub struct ReleaseEvidenceDerivationRequest<'a> {
    pub released_at_utc: &'a str,
    pub repository_commit: &'a str,
    pub catalog_sha256: &'a str,
    pub qualification_sources_sha256: &'a str,
    pub assessments: &'a [QualificationAssessment],
}

#[derive(Serialize)]
struct QualificationEvidenceSet<'a> {
    subject_id: &'a str,
    product_support: crate::qualification::ProductSupport,
    artifact_delivery: crate::qualification::ArtifactDelivery,
    redistribution_permitted: bool,
    availability: crate::qualification::ArtifactAvailability,
    publication_disposition: crate::qualification::PublicationDisposition,
    dimensions: BTreeMap<EvidenceDimension, &'a EvidenceDecision>,
}

/// Derive a release publication from validated qualification assessments.
///
/// The release dimension is deliberately excluded from each evidence-set digest, preventing a
/// publication from depending on its own identity.
///
/// # Errors
/// Returns an error when an assessment is invalid or the derived publication is inconsistent.
pub fn derive_release_evidence(
    request: &ReleaseEvidenceDerivationRequest<'_>,
) -> Result<ContainerReleaseEvidencePublication> {
    let mut tools = BTreeMap::new();
    for assessment in request.assessments {
        assessment.validate()?;
        let (verdict, reasons) = release_verdict(assessment);
        let evidence = ToolReleaseEvidence {
            tool_id: assessment.subject_id.clone(),
            product_support: assessment.product_support,
            artifact_delivery: assessment.artifact_delivery,
            redistribution_permitted: assessment.redistribution_permitted,
            availability: assessment.availability,
            qualification_evidence_set_sha256: qualification_evidence_set_sha256(assessment)?,
            verdict,
            reasons,
        };
        ensure!(
            tools.insert(assessment.subject_id.clone(), evidence).is_none(),
            "duplicate release qualification subject: {}",
            assessment.subject_id
        );
    }
    let publication = ContainerReleaseEvidencePublication {
        schema_version: CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1.to_string(),
        released_at_utc: request.released_at_utc.to_string(),
        repository_commit: request.repository_commit.to_string(),
        catalog_sha256: request.catalog_sha256.to_string(),
        qualification_sources_sha256: request.qualification_sources_sha256.to_string(),
        policy_sha256: release_evidence_policy_sha256(),
        tools,
    };
    validate_release_evidence_publication(&publication)?;
    Ok(publication)
}

/// Hash all qualification semantics except the self-referential release decision.
///
/// # Errors
/// Returns an error when the assessment is invalid or cannot be serialized.
pub fn qualification_evidence_set_sha256(assessment: &QualificationAssessment) -> Result<String> {
    assessment.validate()?;
    let dimensions = assessment
        .dimensions
        .iter()
        .filter(|(dimension, _)| **dimension != EvidenceDimension::Release)
        .map(|(dimension, decision)| (*dimension, decision))
        .collect();
    let evidence = QualificationEvidenceSet {
        subject_id: &assessment.subject_id,
        product_support: assessment.product_support,
        artifact_delivery: assessment.artifact_delivery,
        redistribution_permitted: assessment.redistribution_permitted,
        availability: assessment.availability,
        publication_disposition: assessment.publication_disposition,
        dimensions,
    };
    let bytes = serde_json::to_vec(&evidence).context("serialize qualification evidence set")?;
    Ok(sha256_hex(&bytes))
}

/// Hash the portable, non-release source map that feeds qualification.
///
/// A source named `release` is excluded so the publication never signs its own file identity.
///
/// # Errors
/// Returns an error when the source map is empty or cannot be serialized.
pub fn qualification_sources_sha256(
    sources: &BTreeMap<String, QualificationEvidenceSource>,
) -> Result<String> {
    let non_release =
        sources.iter().filter(|(name, _)| name.as_str() != "release").collect::<BTreeMap<_, _>>();
    anyhow::ensure!(!non_release.is_empty(), "qualification source map is empty");
    let bytes = serde_json::to_vec(&non_release).context("serialize qualification source map")?;
    Ok(sha256_hex(&bytes))
}

#[must_use]
pub fn release_evidence_policy_sha256() -> String {
    sha256_hex(RELEASE_POLICY_SPECIFICATION.as_bytes())
}

fn sha256_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

pub(super) fn release_verdict(
    assessment: &QualificationAssessment,
) -> (EvidenceVerdict, Vec<String>) {
    if assessment.product_support != ProductSupport::Supported {
        return (EvidenceVerdict::NotApplicable, Vec::new());
    }
    if assessment.availability != crate::qualification::ArtifactAvailability::Admitted {
        return (EvidenceVerdict::Blocked, vec!["supported artifact is not admitted".to_string()]);
    }
    let non_release = assessment
        .dimensions
        .iter()
        .filter(|(dimension, _)| **dimension != EvidenceDimension::Release)
        .collect::<Vec<_>>();
    let failed = non_release
        .iter()
        .filter(|(_, decision)| decision.verdict == EvidenceVerdict::Failed)
        .map(|(dimension, _)| dimension_name(**dimension))
        .collect::<Vec<_>>();
    if !failed.is_empty() {
        return (
            EvidenceVerdict::Failed,
            vec![format!("failed qualification dimensions: {}", failed.join(", "))],
        );
    }
    let incomplete = non_release
        .iter()
        .filter(|(_, decision)| {
            matches!(decision.verdict, EvidenceVerdict::Blocked | EvidenceVerdict::Missing)
        })
        .map(|(dimension, _)| dimension_name(**dimension))
        .collect::<Vec<_>>();
    if !incomplete.is_empty() {
        return (
            EvidenceVerdict::Blocked,
            vec![format!("incomplete qualification dimensions: {}", incomplete.join(", "))],
        );
    }
    if non_release.iter().any(|(_, decision)| decision.verdict == EvidenceVerdict::NotApplicable) {
        return (
            EvidenceVerdict::Failed,
            vec!["supported artifacts cannot omit required qualification dimensions".to_string()],
        );
    }
    (EvidenceVerdict::Passed, Vec::new())
}

const fn dimension_name(dimension: EvidenceDimension) -> &'static str {
    match dimension {
        EvidenceDimension::Catalog => "catalog",
        EvidenceDimension::Source => "source",
        EvidenceDimension::Build => "build",
        EvidenceDimension::Runtime => "runtime",
        EvidenceDimension::Stage => "stage",
        EvidenceDimension::RealData => "real_data",
        EvidenceDimension::Science => "science",
        EvidenceDimension::Benchmark => "benchmark",
        EvidenceDimension::Security => "security",
        EvidenceDimension::Freshness => "freshness",
        EvidenceDimension::Release => "release",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use anyhow::Result;

    use super::{
        derive_release_evidence, qualification_evidence_set_sha256, qualification_sources_sha256,
        ReleaseEvidenceDerivationRequest,
    };
    use crate::qualification::{
        ArtifactAvailability, ArtifactDelivery, EvidenceDecision, EvidenceDimension,
        EvidenceVerdict, ProductSupport, PublicationDisposition, QualificationAssessment,
        QualificationEvidenceSource, REQUIRED_EVIDENCE_DIMENSIONS,
    };

    const REPOSITORY_COMMIT: &str = "1111111111111111111111111111111111111111";
    const CATALOG_SHA256: &str = "2222222222222222222222222222222222222222222222222222222222222222";
    const SOURCES_SHA256: &str = "3333333333333333333333333333333333333333333333333333333333333333";
    const PUBLICATION_SHA256: &str =
        "4444444444444444444444444444444444444444444444444444444444444444";

    fn assessment(
        subject: &str,
        missing_dimension: Option<EvidenceDimension>,
    ) -> Result<QualificationAssessment> {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| {
                let measured = BTreeMap::from([("state".to_string(), "present".to_string())]);
                let thresholds = BTreeMap::from([("state".to_string(), "present".to_string())]);
                let decision = if Some(dimension) == missing_dimension {
                    EvidenceDecision::missing(
                        "qualification.release.fixture.v1",
                        measured,
                        thresholds,
                        "fixture evidence is absent",
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
        QualificationAssessment::v3(
            subject.to_string(),
            ProductSupport::Supported,
            ArtifactDelivery::RedistributableImage,
            true,
            ArtifactAvailability::Admitted,
            PublicationDisposition::Current,
            dimensions,
        )
    }

    fn request(assessments: &[QualificationAssessment]) -> ReleaseEvidenceDerivationRequest<'_> {
        ReleaseEvidenceDerivationRequest {
            released_at_utc: "2026-08-10T12:00:00Z",
            repository_commit: REPOSITORY_COMMIT,
            catalog_sha256: CATALOG_SHA256,
            qualification_sources_sha256: SOURCES_SHA256,
            assessments,
        }
    }

    #[test]
    fn derives_release_verdicts_from_non_release_dimensions() -> Result<()> {
        let assessments = vec![
            assessment("passing", None)?,
            assessment("incomplete", Some(EvidenceDimension::Science))?,
        ];
        let publication = derive_release_evidence(&request(&assessments))?;
        assert_eq!(publication.tools["passing"].verdict, EvidenceVerdict::Passed);
        assert_eq!(publication.tools["incomplete"].verdict, EvidenceVerdict::Blocked);
        assert!(publication.tools["incomplete"].reasons[0].contains("science"));
        Ok(())
    }

    #[test]
    fn release_decision_is_excluded_from_the_evidence_set_identity() -> Result<()> {
        let original = assessment("samtools", None)?;
        let mut dimensions = original.dimensions.clone();
        dimensions.insert(
            EvidenceDimension::Release,
            EvidenceDecision::failed(
                "qualification.release.fixture.v1",
                BTreeMap::from([("release_state".to_string(), "failed".to_string())]),
                BTreeMap::from([("release_state".to_string(), "passed".to_string())]),
                "release outcome changed",
            ),
        );
        let changed = QualificationAssessment::v3(
            original.subject_id.clone(),
            original.product_support,
            original.artifact_delivery,
            original.redistribution_permitted,
            original.availability,
            original.publication_disposition,
            dimensions,
        )?;
        assert_eq!(
            qualification_evidence_set_sha256(&original)?,
            qualification_evidence_set_sha256(&changed)?
        );
        Ok(())
    }

    #[test]
    fn release_source_is_excluded_from_the_source_set_identity() -> Result<()> {
        let mut sources = BTreeMap::from([(
            "catalog".to_string(),
            QualificationEvidenceSource {
                uri: "repository://containers/contracts/catalog.toml".to_string(),
                sha256: CATALOG_SHA256.to_string(),
            },
        )]);
        let without_release = qualification_sources_sha256(&sources)?;
        sources.insert(
            "release".to_string(),
            QualificationEvidenceSource {
                uri: "study://containers/release-evidence.json".to_string(),
                sha256: PUBLICATION_SHA256.to_string(),
            },
        );
        assert_eq!(without_release, qualification_sources_sha256(&sources)?);
        Ok(())
    }

    #[test]
    fn admission_rejects_changed_qualification_evidence() -> Result<()> {
        let assessments = vec![assessment("samtools", None)?];
        let publication = derive_release_evidence(&request(&assessments))?;
        let changed = assessment("samtools", Some(EvidenceDimension::Science))?;
        let result = crate::release_evidence::admit_release_evidence(
            &publication,
            PUBLICATION_SHA256,
            REPOSITORY_COMMIT,
            CATALOG_SHA256,
            SOURCES_SHA256,
            &[changed],
        );
        assert!(result.is_err_and(|error| error.to_string().contains("evidence set differs")));
        Ok(())
    }

    #[test]
    fn admitted_publication_yields_policy_owned_release_decision() -> Result<()> {
        let assessments = vec![assessment("samtools", None)?];
        let publication = derive_release_evidence(&request(&assessments))?;
        let admitted = crate::release_evidence::admit_release_evidence(
            &publication,
            PUBLICATION_SHA256,
            REPOSITORY_COMMIT,
            CATALOG_SHA256,
            SOURCES_SHA256,
            &assessments,
        )?;
        let decision = admitted.tool_decision("samtools");
        assert_eq!(decision.verdict, EvidenceVerdict::Passed);
        assert_eq!(decision.evidence_sha256.as_deref(), Some(PUBLICATION_SHA256));
        Ok(())
    }

    #[test]
    fn admission_rejects_a_verdict_that_does_not_follow_policy() -> Result<()> {
        let assessments = vec![assessment("samtools", None)?];
        let mut publication = derive_release_evidence(&request(&assessments))?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("release fixture omits samtools");
        };
        evidence.verdict = EvidenceVerdict::Blocked;
        evidence.reasons = vec!["operator override".to_string()];
        let result = crate::release_evidence::admit_release_evidence(
            &publication,
            PUBLICATION_SHA256,
            REPOSITORY_COMMIT,
            CATALOG_SHA256,
            SOURCES_SHA256,
            &assessments,
        );
        assert!(result.is_err_and(|error| error.to_string().contains("verdict differs")));
        Ok(())
    }

    #[test]
    fn derivation_rejects_duplicate_qualification_subjects() -> Result<()> {
        let duplicate = assessment("samtools", None)?;
        let assessments = vec![duplicate.clone(), duplicate];
        let result = derive_release_evidence(&request(&assessments));
        assert!(result.is_err_and(|error| error.to_string().contains("duplicate")));
        Ok(())
    }
}
