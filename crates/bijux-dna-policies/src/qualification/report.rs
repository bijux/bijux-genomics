use std::collections::BTreeMap;

use anyhow::{ensure, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::{
    EvidenceDimension, EvidenceVerdict, ProductSupport, QualificationAssessment,
    QualificationClaim, REQUIRED_EVIDENCE_DIMENSIONS,
};

pub const QUALIFICATION_REPORT_SCHEMA_V4: &str = "bijux.container.qualification_report.v4";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationEvidenceSource {
    pub uri: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationReportSummary {
    pub tool_count: usize,
    pub supported_count: usize,
    pub retired_count: usize,
    pub excluded_count: usize,
    pub qualified_count: usize,
    pub unverified_count: usize,
    pub blocked_count: usize,
    pub failed_count: usize,
    pub dimension_passes: BTreeMap<EvidenceDimension, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualificationReport {
    #[schemars(schema_with = "qualification_report_schema_version")]
    pub schema_version: String,
    pub sources: BTreeMap<String, QualificationEvidenceSource>,
    pub summary: QualificationReportSummary,
    pub assessments: Vec<QualificationAssessment>,
}

impl QualificationReport {
    /// Build a portable qualification report and derive its summary from validated assessments.
    ///
    /// # Errors
    /// Returns an error for missing or non-portable sources, invalid assessments, or duplicate
    /// qualification subjects.
    pub fn v4(
        sources: BTreeMap<String, QualificationEvidenceSource>,
        mut assessments: Vec<QualificationAssessment>,
    ) -> Result<Self> {
        assessments.sort_by(|left, right| left.subject_id.cmp(&right.subject_id));
        let summary = derive_summary(&assessments);
        let report = Self {
            schema_version: QUALIFICATION_REPORT_SCHEMA_V4.to_string(),
            sources,
            summary,
            assessments,
        };
        report.validate()?;
        Ok(report)
    }

    /// Validate publication identity, source portability, assessment closure, and derived summary.
    ///
    /// # Errors
    /// Returns an error when serialized report content is malformed or contradicts its evidence.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == QUALIFICATION_REPORT_SCHEMA_V4,
            "unsupported qualification report schema: {}",
            self.schema_version
        );
        ensure!(!self.sources.is_empty(), "qualification report has no evidence sources");
        for (name, source) in &self.sources {
            ensure!(!name.trim().is_empty(), "qualification evidence source name is empty");
            ensure!(
                source.uri.starts_with("repository://") || source.uri.starts_with("study://"),
                "qualification evidence source {name} is not portable: {}",
                source.uri
            );
            ensure!(
                valid_sha256(&source.sha256),
                "qualification evidence source {name} has an invalid SHA-256 identity"
            );
        }
        ensure!(!self.assessments.is_empty(), "qualification report has no assessments");
        for assessment in &self.assessments {
            assessment.validate()?;
        }
        ensure!(
            self.assessments.windows(2).all(|pair| pair[0].subject_id < pair[1].subject_id),
            "qualification assessments must be sorted by unique subject ID"
        );
        ensure!(
            self.summary == derive_summary(&self.assessments),
            "qualification report summary does not match evidence-derived assessments"
        );
        Ok(())
    }
}

fn derive_summary(assessments: &[QualificationAssessment]) -> QualificationReportSummary {
    let supported_count = assessments
        .iter()
        .filter(|assessment| assessment.product_support == ProductSupport::Supported)
        .count();
    let retired_count = assessments
        .iter()
        .filter(|assessment| assessment.product_support == ProductSupport::Retired)
        .count();
    let excluded_count = assessments
        .iter()
        .filter(|assessment| assessment.product_support == ProductSupport::Excluded)
        .count();
    let qualified_count = assessments
        .iter()
        .filter(|assessment| assessment.claims.contains(&QualificationClaim::Qualified))
        .count();
    let blocked_count = assessments
        .iter()
        .filter(|assessment| assessment.claims.contains(&QualificationClaim::Blocked))
        .count();
    let failed_count = assessments
        .iter()
        .filter(|assessment| assessment.claims.contains(&QualificationClaim::Failed))
        .count();
    let dimension_passes = REQUIRED_EVIDENCE_DIMENSIONS
        .into_iter()
        .map(|dimension| {
            let count = assessments
                .iter()
                .filter(|assessment| {
                    assessment
                        .dimensions
                        .get(&dimension)
                        .is_some_and(|decision| decision.verdict == EvidenceVerdict::Passed)
                })
                .count();
            (dimension, count)
        })
        .collect();
    QualificationReportSummary {
        tool_count: assessments.len(),
        supported_count,
        retired_count,
        excluded_count,
        qualified_count,
        unverified_count: supported_count - qualified_count,
        blocked_count,
        failed_count,
        dimension_passes,
    }
}

fn qualification_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": QUALIFICATION_REPORT_SCHEMA_V4,
    })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{QualificationEvidenceSource, QualificationReport, QUALIFICATION_REPORT_SCHEMA_V4};
    use crate::qualification::{
        ArtifactAvailability, ArtifactDelivery, EvidenceDecision, EvidenceVerdict, ProductSupport,
        PublicationDisposition, QualificationAssessment, REQUIRED_EVIDENCE_DIMENSIONS,
    };

    fn assessment(
        subject: &str,
        delivery: ArtifactDelivery,
        availability: ArtifactAvailability,
    ) -> anyhow::Result<QualificationAssessment> {
        let verdict = if availability == ArtifactAvailability::Admitted {
            EvidenceVerdict::Passed
        } else {
            EvidenceVerdict::Blocked
        };
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| {
                let measured =
                    BTreeMap::from([("evidence_state".to_string(), format!("{verdict:?}"))]);
                let thresholds =
                    BTreeMap::from([("required_state".to_string(), "passed".to_string())]);
                let decision = if verdict == EvidenceVerdict::Passed {
                    EvidenceDecision::passed(
                        "qualification.report.test.v1",
                        measured,
                        thresholds,
                        "a".repeat(64),
                        "test evidence passes",
                    )
                } else {
                    EvidenceDecision::blocked(
                        "qualification.report.test.v1",
                        measured,
                        thresholds,
                        "test artifact is not admitted",
                    )
                };
                (dimension, decision)
            })
            .collect();
        QualificationAssessment::v3(
            subject.to_string(),
            ProductSupport::Supported,
            delivery,
            delivery == ArtifactDelivery::RedistributableImage,
            availability,
            PublicationDisposition::Current,
            dimensions,
        )
    }

    fn sources() -> BTreeMap<String, QualificationEvidenceSource> {
        BTreeMap::from([(
            "catalog".to_string(),
            QualificationEvidenceSource {
                uri: "repository://containers/contracts/catalog.toml".to_string(),
                sha256: "b".repeat(64),
            },
        )])
    }

    #[test]
    fn report_derives_summary_and_canonical_subject_order() -> anyhow::Result<()> {
        let report = QualificationReport::v4(
            sources(),
            vec![
                assessment(
                    "samtools",
                    ArtifactDelivery::RedistributableImage,
                    ArtifactAvailability::Admitted,
                )?,
                assessment(
                    "angsd",
                    ArtifactDelivery::ExternalArtifact,
                    ArtifactAvailability::BlockedExternalAccess,
                )?,
            ],
        )?;
        assert_eq!(report.assessments[0].subject_id, "angsd");
        assert_eq!(report.summary.tool_count, 2);
        assert_eq!(report.summary.supported_count, 2);
        assert_eq!(report.summary.retired_count, 0);
        assert_eq!(report.summary.excluded_count, 0);
        assert_eq!(report.summary.qualified_count, 1);
        assert_eq!(report.summary.unverified_count, 1);
        assert_eq!(report.summary.blocked_count, 1);
        assert_eq!(report.summary.dimension_passes.values().copied().sum::<usize>(), 11);
        Ok(())
    }

    #[test]
    fn report_rejects_operator_paths_and_stale_summaries() -> anyhow::Result<()> {
        let mut local_sources = sources();
        let catalog_source = local_sources
            .get_mut("catalog")
            .ok_or_else(|| anyhow::anyhow!("catalog fixture is missing"))?;
        catalog_source.uri = "/home/operator/catalog.toml".to_string();
        assert!(QualificationReport::v4(
            local_sources,
            vec![assessment(
                "samtools",
                ArtifactDelivery::RedistributableImage,
                ArtifactAvailability::Admitted,
            )?],
        )
        .is_err_and(|error| error.to_string().contains("not portable")));

        let mut report = QualificationReport::v4(
            sources(),
            vec![assessment(
                "samtools",
                ArtifactDelivery::RedistributableImage,
                ArtifactAvailability::Admitted,
            )?],
        )?;
        report.summary.qualified_count = 0;
        assert!(report
            .validate()
            .is_err_and(|error| error.to_string().contains("summary does not match")));
        Ok(())
    }

    #[test]
    fn report_rejects_unknown_serialized_fields() {
        let payload = serde_json::json!({
            "schema_version": QUALIFICATION_REPORT_SCHEMA_V4,
            "sources": {},
            "summary": {},
            "assessments": [],
            "operator": "local"
        });
        assert!(serde_json::from_value::<QualificationReport>(payload).is_err());
    }
}
