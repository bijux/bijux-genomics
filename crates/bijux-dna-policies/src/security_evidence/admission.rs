use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::{
    validate_security_evidence_publication, ContainerSecurityEvidencePublication,
    ToolSecurityEvidence,
};
use crate::qualification::{
    ArtifactAvailability, EvidenceDecision, EvidenceVerdict, SECURITY_EVIDENCE_RULE_V1,
};

/// A security publication validated against the exact qualification authority set.
pub struct AdmittedSecurityEvidence<'a> {
    publication: &'a ContainerSecurityEvidencePublication,
    publication_sha256: String,
}

/// Admit one complete security publication for qualification decisions.
///
/// # Errors
/// Returns an error for malformed evidence, an invalid publication digest, or source, catalog, or
/// inventory identity drift.
pub fn admit_security_evidence<'a>(
    publication: &'a ContainerSecurityEvidencePublication,
    publication_sha256: &str,
    repository_commit: &str,
    catalog_sha256: &str,
    inventory_sha256: &str,
) -> Result<AdmittedSecurityEvidence<'a>> {
    validate_security_evidence_publication(publication)?;
    validate_sha256(publication_sha256, "publication")?;
    if publication.repository_commit != repository_commit {
        bail!("security publication repository commit differs from qualification source");
    }
    if publication.catalog_sha256 != catalog_sha256 {
        bail!("security publication catalog differs from qualification source");
    }
    if publication.inventory_sha256 != inventory_sha256 {
        bail!("security publication inventory differs from qualification source");
    }
    Ok(AdmittedSecurityEvidence { publication, publication_sha256: publication_sha256.to_string() })
}

impl AdmittedSecurityEvidence<'_> {
    /// Return validated evidence for one catalog tool.
    #[must_use]
    pub fn tool_evidence(&self, tool_id: &str) -> Option<&ToolSecurityEvidence> {
        self.publication.tools.get(tool_id)
    }

    /// Derive a qualification decision for a tool whose artifact is not admitted.
    #[must_use]
    pub fn tool_availability_decision(&self, tool_id: &str) -> EvidenceDecision {
        let evidence = self.publication.tools.get(tool_id);
        let measured_values = BTreeMap::from([
            (
                "artifact_availability".to_string(),
                evidence.map_or("missing", |item| availability_name(item.availability)).to_string(),
            ),
            (
                "security_verdict".to_string(),
                evidence.map_or("missing", |item| verdict_name(item.verdict)).to_string(),
            ),
        ]);
        let thresholds = BTreeMap::from([
            ("artifact_availability".to_string(), "admitted".to_string()),
            ("security_verdict".to_string(), "passed".to_string()),
        ]);
        let Some(evidence) = evidence else {
            return EvidenceDecision::missing(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "admitted security publication omits the supported tool",
            );
        };
        verdict_decision(
            evidence,
            measured_values,
            thresholds,
            &self.publication_sha256,
            "validated security controls pass for the admitted artifact",
        )
    }

    /// Derive one qualification decision from evidence bound to an admitted artifact.
    #[must_use]
    pub fn tool_decision(&self, tool_id: &str, artifact_sha256: &str) -> EvidenceDecision {
        let evidence = self.publication.tools.get(tool_id);
        let observed_artifact =
            evidence.and_then(|item| item.artifact_sha256.as_deref()).unwrap_or("missing");
        let observed_verdict =
            evidence.map_or("missing", |item| verdict_name(item.verdict)).to_string();
        let measured_values = BTreeMap::from([
            ("required_artifact_sha256".to_string(), artifact_sha256.to_string()),
            ("observed_artifact_sha256".to_string(), observed_artifact.to_string()),
            ("security_verdict".to_string(), observed_verdict),
        ]);
        let thresholds = BTreeMap::from([
            ("artifact_identity".to_string(), "exact SHA-256 match".to_string()),
            ("security_verdict".to_string(), "passed".to_string()),
        ]);
        let Some(evidence) = evidence else {
            return EvidenceDecision::missing(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "admitted security publication omits the supported tool",
            );
        };
        if evidence.artifact_sha256.as_deref() != Some(artifact_sha256) {
            return EvidenceDecision::failed(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "security evidence is bound to a different artifact",
            );
        }
        verdict_decision(
            evidence,
            measured_values,
            thresholds,
            &self.publication_sha256,
            "validated security controls pass for the exact artifact",
        )
    }

    #[must_use]
    pub fn publication_sha256(&self) -> &str {
        &self.publication_sha256
    }
}

const fn availability_name(availability: ArtifactAvailability) -> &'static str {
    match availability {
        ArtifactAvailability::Admitted => "admitted",
        ArtifactAvailability::BlockedExternalAccess => "blocked_external_access",
        ArtifactAvailability::Unavailable => "unavailable",
    }
}

const fn verdict_name(verdict: EvidenceVerdict) -> &'static str {
    match verdict {
        EvidenceVerdict::Passed => "passed",
        EvidenceVerdict::Failed => "failed",
        EvidenceVerdict::Blocked => "blocked",
        EvidenceVerdict::Missing => "missing",
        EvidenceVerdict::NotApplicable => "not_applicable",
    }
}

fn verdict_decision(
    evidence: &ToolSecurityEvidence,
    measured_values: BTreeMap<String, String>,
    thresholds: BTreeMap<String, String>,
    publication_sha256: &str,
    passed_reason: &str,
) -> EvidenceDecision {
    match evidence.verdict {
        EvidenceVerdict::Passed => EvidenceDecision::passed(
            SECURITY_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            publication_sha256.to_string(),
            passed_reason,
        ),
        EvidenceVerdict::Failed => EvidenceDecision::failed(
            SECURITY_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            &decision_reason(evidence.reasons.as_slice(), "security controls failed"),
        ),
        EvidenceVerdict::Blocked => EvidenceDecision::blocked(
            SECURITY_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            &decision_reason(evidence.reasons.as_slice(), "security controls are blocked"),
        ),
        EvidenceVerdict::Missing => EvidenceDecision::missing(
            SECURITY_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            &decision_reason(evidence.reasons.as_slice(), "security evidence is incomplete"),
        ),
        EvidenceVerdict::NotApplicable => EvidenceDecision::failed(
            SECURITY_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            "supported tool security evidence cannot be not applicable",
        ),
    }
}

fn decision_reason(reasons: &[String], fallback: &str) -> String {
    if reasons.is_empty() {
        fallback.to_string()
    } else {
        reasons.join("; ")
    }
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("security evidence {label} is not a SHA-256 digest: {value}");
    }
    Ok(())
}
