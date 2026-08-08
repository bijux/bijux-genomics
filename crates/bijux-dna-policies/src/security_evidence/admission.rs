use std::collections::BTreeMap;

use anyhow::{bail, Result};

use super::{validate_security_evidence_publication, ContainerSecurityEvidencePublication};
use crate::qualification::{EvidenceDecision, EvidenceVerdict, SECURITY_EVIDENCE_RULE_V1};

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
    /// Derive one qualification decision from validated, image-bound tool evidence.
    #[must_use]
    pub fn tool_decision(&self, tool_id: &str, image_sha256: &str) -> EvidenceDecision {
        let evidence = self.publication.tools.get(tool_id);
        let observed_image =
            evidence.and_then(|item| item.image_sha256.as_deref()).unwrap_or("missing");
        let observed_verdict = evidence.map_or_else(
            || "missing".to_string(),
            |item| format!("{:?}", item.verdict).to_ascii_lowercase(),
        );
        let measured_values = BTreeMap::from([
            ("required_image_sha256".to_string(), image_sha256.to_string()),
            ("observed_image_sha256".to_string(), observed_image.to_string()),
            ("security_verdict".to_string(), observed_verdict),
        ]);
        let thresholds = BTreeMap::from([
            ("image_identity".to_string(), "exact SHA-256 match".to_string()),
            ("security_verdict".to_string(), "passed".to_string()),
        ]);
        let Some(evidence) = evidence else {
            return EvidenceDecision::missing(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "admitted security publication omits the retained tool",
            );
        };
        if evidence.image_sha256.as_deref() != Some(image_sha256) {
            return EvidenceDecision::failed(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "security evidence is bound to a different image",
            );
        }
        match evidence.verdict {
            EvidenceVerdict::Passed => EvidenceDecision::passed(
                SECURITY_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                self.publication_sha256.clone(),
                "validated security controls pass for the exact image",
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
                "retained tool security evidence cannot be not applicable",
            ),
        }
    }

    #[must_use]
    pub fn publication_sha256(&self) -> &str {
        &self.publication_sha256
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
