use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, ensure, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::evaluation::{CLAIM_REQUIREMENTS, REQUIRED_EVIDENCE_DIMENSIONS};

pub const QUALIFICATION_ASSESSMENT_SCHEMA_V1: &str = "bijux.container.qualification_assessment.v1";

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum QualificationClaim {
    Unsupported,
    Planned,
    Built,
    RuntimeVerified,
    StageVerified,
    RealDataVerified,
    ScienceVerified,
    BenchmarkVerified,
    SecurityVerified,
    ReleaseVerified,
    Qualified,
    Failed,
    Superseded,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportDisposition {
    Retained,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationDisposition {
    Current,
    Superseded,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDimension {
    Catalog,
    Source,
    Build,
    Runtime,
    Stage,
    RealData,
    Science,
    Benchmark,
    Security,
    Freshness,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerdict {
    Passed,
    Failed,
    Blocked,
    Missing,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceDecision {
    pub verdict: EvidenceVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationAssessment {
    #[schemars(schema_with = "qualification_assessment_schema_version")]
    pub schema_version: String,
    pub subject_id: String,
    pub support_disposition: SupportDisposition,
    pub publication_disposition: PublicationDisposition,
    pub dimensions: BTreeMap<EvidenceDimension, EvidenceDecision>,
    pub claims: BTreeSet<QualificationClaim>,
}

fn qualification_assessment_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": QUALIFICATION_ASSESSMENT_SCHEMA_V1,
    })
}

impl QualificationAssessment {
    /// Build a v1 assessment and derive its public claims from the supplied evidence.
    ///
    /// # Errors
    /// Returns an error when the evidence dimensions or decisions are invalid.
    pub fn v1(
        subject_id: String,
        support_disposition: SupportDisposition,
        publication_disposition: PublicationDisposition,
        dimensions: BTreeMap<EvidenceDimension, EvidenceDecision>,
    ) -> Result<Self> {
        let mut assessment = Self {
            schema_version: QUALIFICATION_ASSESSMENT_SCHEMA_V1.to_string(),
            subject_id,
            support_disposition,
            publication_disposition,
            dimensions,
            claims: BTreeSet::new(),
        };
        assessment.validate_evidence()?;
        assessment.claims = assessment.derive_claims();
        Ok(assessment)
    }

    /// Validate that the assessment is complete and every verdict carries admissible evidence.
    ///
    /// # Errors
    /// Returns an error for an unsupported schema, missing dimension, malformed identity, or
    /// verdict that contradicts the subject's support disposition.
    pub fn validate(&self) -> Result<()> {
        self.validate_evidence()?;
        ensure!(
            self.claims == self.derive_claims(),
            "qualification claims do not match evidence-derived claims"
        );
        Ok(())
    }

    fn validate_evidence(&self) -> Result<()> {
        ensure!(
            self.schema_version == QUALIFICATION_ASSESSMENT_SCHEMA_V1,
            "unsupported qualification assessment schema: {}",
            self.schema_version
        );
        ensure!(!self.subject_id.trim().is_empty(), "qualification subject ID is empty");
        let expected = REQUIRED_EVIDENCE_DIMENSIONS.into_iter().collect::<BTreeSet<_>>();
        let actual = self.dimensions.keys().copied().collect::<BTreeSet<_>>();
        ensure!(actual == expected, "qualification dimensions are incomplete or unknown");

        for (dimension, decision) in &self.dimensions {
            validate_decision(*dimension, decision)?;
            match self.support_disposition {
                SupportDisposition::Retained => ensure!(
                    decision.verdict != EvidenceVerdict::NotApplicable,
                    "retained subject {} marks {dimension:?} not applicable",
                    self.subject_id
                ),
                SupportDisposition::Unsupported => ensure!(
                    decision.verdict == EvidenceVerdict::NotApplicable,
                    "unsupported subject {} carries a {dimension:?} claim",
                    self.subject_id
                ),
            }
        }
        Ok(())
    }

    /// Return every public claim supported by the validated evidence set.
    ///
    /// # Errors
    /// Returns an error when the assessment is invalid.
    pub fn supported_claims(&self) -> Result<BTreeSet<QualificationClaim>> {
        self.validate()?;
        Ok(self.claims.clone())
    }

    fn derive_claims(&self) -> BTreeSet<QualificationClaim> {
        if self.publication_disposition == PublicationDisposition::Superseded {
            return BTreeSet::from([QualificationClaim::Superseded]);
        }
        if self.support_disposition == SupportDisposition::Unsupported {
            return BTreeSet::from([QualificationClaim::Unsupported]);
        }

        let mut claims = BTreeSet::from([QualificationClaim::Planned]);
        for (claim, requirements) in CLAIM_REQUIREMENTS {
            if requirements.iter().all(|dimension| {
                self.dimensions
                    .get(dimension)
                    .is_some_and(|decision| decision.verdict == EvidenceVerdict::Passed)
            }) {
                claims.insert(*claim);
            }
        }
        if self.dimensions.values().any(|decision| decision.verdict == EvidenceVerdict::Failed) {
            claims.insert(QualificationClaim::Failed);
        }
        if self.dimensions.values().any(|decision| {
            matches!(decision.verdict, EvidenceVerdict::Blocked | EvidenceVerdict::Missing)
        }) {
            claims.insert(QualificationClaim::Blocked);
        }
        claims
    }

    /// Return whether the complete production qualification claim is supported.
    ///
    /// # Errors
    /// Returns an error when the assessment is invalid.
    pub fn is_qualified(&self) -> Result<bool> {
        Ok(self.supported_claims()?.contains(&QualificationClaim::Qualified))
    }
}

fn validate_decision(dimension: EvidenceDimension, decision: &EvidenceDecision) -> Result<()> {
    match decision.verdict {
        EvidenceVerdict::Passed => {
            let identity = decision
                .evidence_sha256
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("{dimension:?} pass has no evidence identity"))?;
            ensure!(
                identity.len() == 64
                    && identity.bytes().all(|value| value.is_ascii_hexdigit())
                    && !identity.bytes().all(|value| value == b'0'),
                "{dimension:?} pass has an invalid evidence identity"
            );
        }
        EvidenceVerdict::Failed | EvidenceVerdict::Blocked | EvidenceVerdict::Missing => {
            if decision.reasons.iter().all(|reason| reason.trim().is_empty()) {
                bail!("{dimension:?} {:?} verdict has no reason", decision.verdict);
            }
        }
        EvidenceVerdict::NotApplicable => {
            ensure!(
                decision.evidence_sha256.is_none(),
                "{dimension:?} not-applicable verdict carries evidence"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
        QualificationAssessment, QualificationClaim, SupportDisposition,
        QUALIFICATION_ASSESSMENT_SCHEMA_V1,
    };
    use crate::qualification::REQUIRED_EVIDENCE_DIMENSIONS;

    fn retained(verdict: EvidenceVerdict) -> anyhow::Result<QualificationAssessment> {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| {
                let evidence_sha256 = (verdict == EvidenceVerdict::Passed).then(|| "a".repeat(64));
                let reasons = match verdict {
                    EvidenceVerdict::Failed
                    | EvidenceVerdict::Blocked
                    | EvidenceVerdict::Missing => {
                        vec!["evidence is not admitted".to_string()]
                    }
                    EvidenceVerdict::Passed | EvidenceVerdict::NotApplicable => Vec::new(),
                };
                (dimension, EvidenceDecision { verdict, evidence_sha256, reasons })
            })
            .collect();
        QualificationAssessment::v1(
            "samtools".to_string(),
            SupportDisposition::Retained,
            PublicationDisposition::Current,
            dimensions,
        )
    }

    #[test]
    fn qualified_requires_every_dimension_to_pass() -> anyhow::Result<()> {
        let mut assessment = retained(EvidenceVerdict::Passed)?;
        assert!(assessment.is_qualified()?);
        assessment.dimensions.insert(
            EvidenceDimension::Security,
            EvidenceDecision {
                verdict: EvidenceVerdict::Missing,
                evidence_sha256: None,
                reasons: vec!["security campaign not collected".to_string()],
            },
        );
        assert!(assessment.validate().is_err());
        let assessment = QualificationAssessment::v1(
            assessment.subject_id,
            assessment.support_disposition,
            assessment.publication_disposition,
            assessment.dimensions,
        )?;
        let claims = assessment.supported_claims()?;
        assert!(!claims.contains(&QualificationClaim::Qualified));
        assert!(claims.contains(&QualificationClaim::Blocked));
        assert!(!claims.contains(&QualificationClaim::SecurityVerified));
        Ok(())
    }

    #[test]
    fn built_and_runtime_verified_are_distinct_claims() -> anyhow::Result<()> {
        let mut assessment = retained(EvidenceVerdict::Missing)?;
        for dimension in
            [EvidenceDimension::Catalog, EvidenceDimension::Source, EvidenceDimension::Build]
        {
            assessment.dimensions.insert(
                dimension,
                EvidenceDecision {
                    verdict: EvidenceVerdict::Passed,
                    evidence_sha256: Some("b".repeat(64)),
                    reasons: Vec::new(),
                },
            );
        }
        let assessment = QualificationAssessment::v1(
            assessment.subject_id,
            assessment.support_disposition,
            assessment.publication_disposition,
            assessment.dimensions,
        )?;
        let claims = assessment.supported_claims()?;
        assert!(claims.contains(&QualificationClaim::Built));
        assert!(!claims.contains(&QualificationClaim::RuntimeVerified));
        Ok(())
    }

    #[test]
    fn unsupported_subject_supports_only_the_unsupported_claim() -> anyhow::Result<()> {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| {
                (
                    dimension,
                    EvidenceDecision {
                        verdict: EvidenceVerdict::NotApplicable,
                        evidence_sha256: None,
                        reasons: Vec::new(),
                    },
                )
            })
            .collect();
        let mut assessment = QualificationAssessment::v1(
            "angsd".to_string(),
            SupportDisposition::Unsupported,
            PublicationDisposition::Current,
            dimensions,
        )?;
        assert_eq!(
            assessment.supported_claims()?,
            BTreeSet::from([QualificationClaim::Unsupported])
        );
        assessment.dimensions.insert(
            EvidenceDimension::Catalog,
            EvidenceDecision {
                verdict: EvidenceVerdict::Passed,
                evidence_sha256: Some("c".repeat(64)),
                reasons: Vec::new(),
            },
        );
        assert!(assessment.validate().is_err());
        Ok(())
    }

    #[test]
    fn unknown_fields_fail_deserialization() {
        let payload = serde_json::json!({
            "schema_version": QUALIFICATION_ASSESSMENT_SCHEMA_V1,
            "subject_id": "samtools",
            "support_disposition": "retained",
            "publication_disposition": "current",
            "dimensions": {},
            "qualification": "qualified"
        });
        assert!(serde_json::from_value::<QualificationAssessment>(payload).is_err());
    }
}
