use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, ensure, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::evaluation::{CLAIM_REQUIREMENTS, REQUIRED_EVIDENCE_DIMENSIONS};

pub const QUALIFICATION_ASSESSMENT_SCHEMA_V2: &str = "bijux.container.qualification_assessment.v2";

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
    pub audit: DecisionAudit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionAudit {
    pub rule_version: String,
    pub measured_values: BTreeMap<String, String>,
    pub thresholds: BTreeMap<String, String>,
    pub outcome: EvidenceVerdict,
    pub reason: String,
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
        "const": QUALIFICATION_ASSESSMENT_SCHEMA_V2,
    })
}

impl QualificationAssessment {
    /// Build a v1 assessment and derive its public claims from the supplied evidence.
    ///
    /// # Errors
    /// Returns an error when the evidence dimensions or decisions are invalid.
    pub fn v2(
        subject_id: String,
        support_disposition: SupportDisposition,
        publication_disposition: PublicationDisposition,
        dimensions: BTreeMap<EvidenceDimension, EvidenceDecision>,
    ) -> Result<Self> {
        let mut assessment = Self {
            schema_version: QUALIFICATION_ASSESSMENT_SCHEMA_V2.to_string(),
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
            self.schema_version == QUALIFICATION_ASSESSMENT_SCHEMA_V2,
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

impl EvidenceDecision {
    #[must_use]
    pub fn passed(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        evidence_sha256: String,
        reason: &str,
    ) -> Self {
        Self {
            verdict: EvidenceVerdict::Passed,
            evidence_sha256: Some(evidence_sha256),
            reasons: Vec::new(),
            audit: DecisionAudit::new(
                rule_version,
                measured_values,
                thresholds,
                EvidenceVerdict::Passed,
                reason,
            ),
        }
    }

    #[must_use]
    pub fn failed(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        reason: &str,
    ) -> Self {
        Self {
            verdict: EvidenceVerdict::Failed,
            evidence_sha256: None,
            reasons: vec![reason.to_string()],
            audit: DecisionAudit::new(
                rule_version,
                measured_values,
                thresholds,
                EvidenceVerdict::Failed,
                reason,
            ),
        }
    }

    #[must_use]
    pub fn blocked(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        reason: &str,
    ) -> Self {
        Self {
            verdict: EvidenceVerdict::Blocked,
            evidence_sha256: None,
            reasons: vec![reason.to_string()],
            audit: DecisionAudit::new(
                rule_version,
                measured_values,
                thresholds,
                EvidenceVerdict::Blocked,
                reason,
            ),
        }
    }

    #[must_use]
    pub fn missing(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        reason: &str,
    ) -> Self {
        Self {
            verdict: EvidenceVerdict::Missing,
            evidence_sha256: None,
            reasons: vec![reason.to_string()],
            audit: DecisionAudit::new(
                rule_version,
                measured_values,
                thresholds,
                EvidenceVerdict::Missing,
                reason,
            ),
        }
    }

    #[must_use]
    pub fn not_applicable(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        reason: &str,
    ) -> Self {
        Self {
            verdict: EvidenceVerdict::NotApplicable,
            evidence_sha256: None,
            reasons: Vec::new(),
            audit: DecisionAudit::new(
                rule_version,
                measured_values,
                thresholds,
                EvidenceVerdict::NotApplicable,
                reason,
            ),
        }
    }
}

impl DecisionAudit {
    fn new(
        rule_version: &str,
        measured_values: BTreeMap<String, String>,
        thresholds: BTreeMap<String, String>,
        outcome: EvidenceVerdict,
        reason: &str,
    ) -> Self {
        Self {
            rule_version: rule_version.to_string(),
            measured_values,
            thresholds,
            outcome,
            reason: reason.to_string(),
        }
    }
}

fn validate_decision(dimension: EvidenceDimension, decision: &EvidenceDecision) -> Result<()> {
    ensure!(
        !decision.audit.rule_version.trim().is_empty(),
        "{dimension:?} decision has no rule version"
    );
    ensure!(
        !decision.audit.measured_values.is_empty(),
        "{dimension:?} decision has no measured values"
    );
    ensure!(!decision.audit.thresholds.is_empty(), "{dimension:?} decision has no thresholds");
    ensure!(
        decision.audit.outcome == decision.verdict,
        "{dimension:?} decision outcome disagrees with verdict"
    );
    ensure!(!decision.audit.reason.trim().is_empty(), "{dimension:?} decision has no reason");
    for (kind, values) in [
        ("measured value", &decision.audit.measured_values),
        ("threshold", &decision.audit.thresholds),
    ] {
        ensure!(
            values.iter().all(|(key, value)| {
                !key.trim().is_empty()
                    && key == key.trim()
                    && !value.trim().is_empty()
                    && value == value.trim()
            }),
            "{dimension:?} decision has malformed {kind}"
        );
    }
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
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
        QualificationAssessment, QualificationClaim, SupportDisposition,
        QUALIFICATION_ASSESSMENT_SCHEMA_V2,
    };
    use crate::qualification::REQUIRED_EVIDENCE_DIMENSIONS;

    fn decision(verdict: EvidenceVerdict) -> EvidenceDecision {
        let measured = BTreeMap::from([("evidence_state".to_string(), format!("{verdict:?}"))]);
        let thresholds = BTreeMap::from([("required_state".to_string(), "passed".to_string())]);
        match verdict {
            EvidenceVerdict::Passed => EvidenceDecision::passed(
                "qualification.test.v1",
                measured,
                thresholds,
                "a".repeat(64),
                "test evidence passes",
            ),
            EvidenceVerdict::Failed => EvidenceDecision::failed(
                "qualification.test.v1",
                measured,
                thresholds,
                "evidence is not admitted",
            ),
            EvidenceVerdict::Blocked => EvidenceDecision::blocked(
                "qualification.test.v1",
                measured,
                thresholds,
                "evidence is not admitted",
            ),
            EvidenceVerdict::Missing => EvidenceDecision::missing(
                "qualification.test.v1",
                measured,
                thresholds,
                "evidence is not admitted",
            ),
            EvidenceVerdict::NotApplicable => EvidenceDecision::not_applicable(
                "qualification.test.v1",
                measured,
                thresholds,
                "evidence is outside the test population",
            ),
        }
    }

    fn retained(verdict: EvidenceVerdict) -> anyhow::Result<QualificationAssessment> {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| (dimension, decision(verdict)))
            .collect();
        QualificationAssessment::v2(
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
            EvidenceDecision::missing(
                "qualification.security.test.v1",
                BTreeMap::from([("campaign".to_string(), "missing".to_string())]),
                BTreeMap::from([("campaign".to_string(), "required".to_string())]),
                "security campaign not collected",
            ),
        );
        assert!(assessment.validate().is_err());
        let assessment = QualificationAssessment::v2(
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
                EvidenceDecision::passed(
                    "qualification.test.v1",
                    BTreeMap::from([("evidence_state".to_string(), "passed".to_string())]),
                    BTreeMap::from([("required_state".to_string(), "passed".to_string())]),
                    "b".repeat(64),
                    "test evidence passes",
                ),
            );
        }
        let assessment = QualificationAssessment::v2(
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
            .map(|dimension| (dimension, decision(EvidenceVerdict::NotApplicable)))
            .collect();
        let mut assessment = QualificationAssessment::v2(
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
            EvidenceDecision::passed(
                "qualification.test.v1",
                BTreeMap::from([("evidence_state".to_string(), "passed".to_string())]),
                BTreeMap::from([("required_state".to_string(), "passed".to_string())]),
                "c".repeat(64),
                "test evidence passes",
            ),
        );
        assert!(assessment.validate().is_err());
        Ok(())
    }

    #[test]
    fn decision_audit_outcome_cannot_disagree_with_verdict() -> anyhow::Result<()> {
        let mut assessment = retained(EvidenceVerdict::Passed)?;
        let catalog = assessment
            .dimensions
            .get_mut(&EvidenceDimension::Catalog)
            .ok_or_else(|| anyhow::anyhow!("catalog decision is absent"))?;
        catalog.audit.outcome = EvidenceVerdict::Failed;
        assert!(assessment.validate().is_err());
        Ok(())
    }

    #[test]
    fn decision_audit_cannot_omit_measured_values_or_thresholds() -> anyhow::Result<()> {
        let mut assessment = retained(EvidenceVerdict::Passed)?;
        let catalog = assessment
            .dimensions
            .get_mut(&EvidenceDimension::Catalog)
            .ok_or_else(|| anyhow::anyhow!("catalog decision is absent"))?;
        catalog.audit.measured_values.clear();
        assert!(assessment.validate().is_err());

        let mut assessment = retained(EvidenceVerdict::Passed)?;
        let catalog = assessment
            .dimensions
            .get_mut(&EvidenceDimension::Catalog)
            .ok_or_else(|| anyhow::anyhow!("catalog decision is absent"))?;
        catalog.audit.thresholds.clear();
        assert!(assessment.validate().is_err());
        Ok(())
    }

    #[test]
    fn unknown_fields_fail_deserialization() {
        let payload = serde_json::json!({
            "schema_version": QUALIFICATION_ASSESSMENT_SCHEMA_V2,
            "subject_id": "samtools",
            "support_disposition": "retained",
            "publication_disposition": "current",
            "dimensions": {},
            "qualification": "qualified"
        });
        assert!(serde_json::from_value::<QualificationAssessment>(payload).is_err());
    }
}
