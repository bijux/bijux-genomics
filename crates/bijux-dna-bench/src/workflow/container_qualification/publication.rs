//! Container qualification publication from admitted evidence.
use anyhow::Result;
use bijux_dna_policies::{
    AdmittedReleaseEvidence, EvidenceDimension, QualificationAssessment,
    QualificationEvidenceSource,
};
use std::collections::BTreeMap;

pub(super) fn apply_release_evidence(
    assessments: Vec<QualificationAssessment>,
    release: &AdmittedReleaseEvidence<'_>,
) -> Result<Vec<QualificationAssessment>> {
    assessments
        .into_iter()
        .map(|assessment| {
            let mut dimensions = assessment.dimensions;
            dimensions
                .insert(EvidenceDimension::Release, release.tool_decision(&assessment.subject_id));
            QualificationAssessment::v3(
                assessment.subject_id,
                assessment.product_support,
                assessment.artifact_delivery,
                assessment.redistribution_permitted,
                assessment.availability,
                assessment.publication_disposition,
                dimensions,
            )
        })
        .collect()
}

pub(super) fn freshness_identity(
    repository_commit: &str,
    sources: &BTreeMap<String, QualificationEvidenceSource>,
) -> Result<String> {
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&(repository_commit, sources))?))
}
