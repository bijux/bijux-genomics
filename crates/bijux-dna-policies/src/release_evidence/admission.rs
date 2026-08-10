use std::collections::{BTreeMap, BTreeSet};

use anyhow::{ensure, Result};

use super::{
    derivation::release_verdict, qualification_evidence_set_sha256,
    validate_release_evidence_publication, ContainerReleaseEvidencePublication,
};
use crate::qualification::{
    EvidenceDecision, EvidenceVerdict, QualificationAssessment, RELEASE_EVIDENCE_RULE_V1,
};

pub struct AdmittedReleaseEvidence<'a> {
    publication: &'a ContainerReleaseEvidencePublication,
    publication_sha256: String,
}

/// Admit a release publication against the exact current authority and qualification evidence.
///
/// # Errors
/// Returns an error for authority drift, coverage drift, or changed qualification evidence.
pub fn admit_release_evidence<'a>(
    publication: &'a ContainerReleaseEvidencePublication,
    publication_sha256: &str,
    repository_commit: &str,
    catalog_sha256: &str,
    qualification_sources_sha256: &str,
    assessments: &[QualificationAssessment],
) -> Result<AdmittedReleaseEvidence<'a>> {
    validate_release_evidence_publication(publication)?;
    ensure!(valid_sha256(publication_sha256), "release publication SHA-256 is invalid");
    ensure!(
        publication.repository_commit == repository_commit,
        "release publication repository commit differs from current authority"
    );
    ensure!(
        publication.catalog_sha256 == catalog_sha256,
        "release publication catalog differs from current authority"
    );
    ensure!(
        publication.qualification_sources_sha256 == qualification_sources_sha256,
        "release publication qualification sources differ from current authority"
    );
    let expected = assessments
        .iter()
        .map(|assessment| assessment.subject_id.as_str())
        .collect::<BTreeSet<_>>();
    ensure!(
        expected.len() == assessments.len(),
        "qualification contains duplicate release subjects"
    );
    let observed = publication.tools.keys().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(expected == observed, "release publication tool coverage differs from qualification");
    for assessment in assessments {
        assessment.validate()?;
        let evidence = &publication.tools[&assessment.subject_id];
        ensure!(
            evidence.product_support == assessment.product_support
                && evidence.artifact_delivery == assessment.artifact_delivery
                && evidence.redistribution_permitted == assessment.redistribution_permitted
                && evidence.availability == assessment.availability,
            "release support dimensions differ from qualification for {}",
            assessment.subject_id
        );
        ensure!(
            evidence.qualification_evidence_set_sha256
                == qualification_evidence_set_sha256(assessment)?,
            "release evidence set differs from qualification for {}",
            assessment.subject_id
        );
        let (expected_verdict, expected_reasons) = release_verdict(assessment);
        ensure!(
            evidence.verdict == expected_verdict && evidence.reasons == expected_reasons,
            "release verdict differs from qualification policy for {}",
            assessment.subject_id
        );
    }
    Ok(AdmittedReleaseEvidence { publication, publication_sha256: publication_sha256.to_string() })
}

impl AdmittedReleaseEvidence<'_> {
    #[must_use]
    pub fn tool_decision(&self, tool_id: &str) -> EvidenceDecision {
        let evidence = self.publication.tools.get(tool_id);
        let measured_values = BTreeMap::from([
            (
                "publication_coverage".to_string(),
                if evidence.is_some() { "present" } else { "missing" }.to_string(),
            ),
            (
                "release_verdict".to_string(),
                evidence.map_or("missing", |item| verdict_name(item.verdict)).to_string(),
            ),
        ]);
        let thresholds = BTreeMap::from([
            ("publication_coverage".to_string(), "present".to_string()),
            ("release_verdict".to_string(), "passed".to_string()),
        ]);
        let Some(evidence) = evidence else {
            return EvidenceDecision::missing(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "admitted release publication omits the tool",
            );
        };
        match evidence.verdict {
            EvidenceVerdict::Passed => EvidenceDecision::passed(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                self.publication_sha256.clone(),
                "immutable release publication covers the exact qualification evidence set",
            ),
            EvidenceVerdict::Failed => EvidenceDecision::failed(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                &reason(&evidence.reasons, "release policy failed"),
            ),
            EvidenceVerdict::Blocked => EvidenceDecision::blocked(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                &reason(&evidence.reasons, "release policy is blocked"),
            ),
            EvidenceVerdict::Missing => EvidenceDecision::missing(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                &reason(&evidence.reasons, "release evidence is incomplete"),
            ),
            EvidenceVerdict::NotApplicable => EvidenceDecision::not_applicable(
                RELEASE_EVIDENCE_RULE_V1,
                measured_values,
                thresholds,
                "release policy does not apply to an unsupported tool",
            ),
        }
    }

    #[must_use]
    pub fn publication_sha256(&self) -> &str {
        &self.publication_sha256
    }
}

fn reason(reasons: &[String], fallback: &str) -> String {
    if reasons.is_empty() {
        fallback.to_string()
    } else {
        reasons.join("; ")
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

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}
