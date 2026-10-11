use std::collections::BTreeMap;

use bijux_dna_policies::EvidenceDecision;

pub const SCIENTIFIC_ACCEPTANCE_RULE_V1: &str = "science.acceptance_evidence.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScientificExecutionStatus {
    Passed,
    Failed,
    Blocked,
    Incomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScientificAssessmentStatus {
    Passed,
    Failed,
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScientificAcceptanceObservation {
    pub run_id: String,
    pub stage: String,
    pub evidence_sha256: String,
    pub execution_status: ScientificExecutionStatus,
    pub scientific_status: ScientificAssessmentStatus,
}

#[must_use]
pub fn evaluate_scientific_acceptance(
    observations: &[ScientificAcceptanceObservation],
    required_stage: &str,
) -> EvidenceDecision {
    let mut executed = observations
        .iter()
        .filter(|observation| {
            observation.stage == required_stage
                && observation.execution_status == ScientificExecutionStatus::Passed
        })
        .collect::<Vec<_>>();
    executed.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    let measured_values = BTreeMap::from([
        ("required_stage".to_string(), required_stage.to_string()),
        ("passing_execution_count".to_string(), executed.len().to_string()),
        (
            "scientific_statuses".to_string(),
            if executed.is_empty() {
                "none".to_string()
            } else {
                executed
                    .iter()
                    .map(|observation| {
                        format!(
                            "{}:{}",
                            observation.run_id,
                            scientific_status_name(observation.scientific_status)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            },
        ),
    ]);
    let thresholds = BTreeMap::from([
        ("execution_status".to_string(), "passed".to_string()),
        ("scientific_status".to_string(), "passed".to_string()),
        ("stage_identity".to_string(), "exact match".to_string()),
    ]);

    if let Some(observation) = executed
        .iter()
        .copied()
        .find(|observation| observation.scientific_status == ScientificAssessmentStatus::Passed)
    {
        return EvidenceDecision::passed(
            SCIENTIFIC_ACCEPTANCE_RULE_V1,
            measured_values,
            thresholds,
            observation.evidence_sha256.clone(),
            "a passing execution has an explicit scientific acceptance pass",
        );
    }
    if executed
        .iter()
        .any(|observation| observation.scientific_status == ScientificAssessmentStatus::Failed)
    {
        return EvidenceDecision::failed(
            SCIENTIFIC_ACCEPTANCE_RULE_V1,
            measured_values,
            thresholds,
            "scientific acceptance failed for the declared-stage execution",
        );
    }
    if !executed.is_empty() {
        return EvidenceDecision::missing(
            SCIENTIFIC_ACCEPTANCE_RULE_V1,
            measured_values,
            thresholds,
            "declared-stage execution has no scientific acceptance assessment",
        );
    }
    EvidenceDecision::blocked(
        SCIENTIFIC_ACCEPTANCE_RULE_V1,
        measured_values,
        thresholds,
        "scientific acceptance requires a passing declared-stage execution",
    )
}

fn scientific_status_name(status: ScientificAssessmentStatus) -> &'static str {
    match status {
        ScientificAssessmentStatus::Passed => "passed",
        ScientificAssessmentStatus::Failed => "failed",
        ScientificAssessmentStatus::NotAssessed => "not_assessed",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_scientific_acceptance, ScientificAcceptanceObservation,
        ScientificAssessmentStatus, ScientificExecutionStatus,
    };
    use bijux_dna_policies::EvidenceVerdict;

    fn observation(
        execution_status: ScientificExecutionStatus,
        scientific_status: ScientificAssessmentStatus,
    ) -> ScientificAcceptanceObservation {
        ScientificAcceptanceObservation {
            run_id: "run-1001".to_string(),
            stage: "fastq.trim_reads".to_string(),
            evidence_sha256: "a".repeat(64),
            execution_status,
            scientific_status,
        }
    }

    #[test]
    fn exit_success_cannot_imply_scientific_acceptance() {
        let decision = evaluate_scientific_acceptance(
            &[observation(
                ScientificExecutionStatus::Passed,
                ScientificAssessmentStatus::NotAssessed,
            )],
            "fastq.trim_reads",
        );
        assert_eq!(decision.verdict, EvidenceVerdict::Missing);
    }

    #[test]
    fn scientific_pass_cannot_override_failed_execution() {
        let decision = evaluate_scientific_acceptance(
            &[observation(ScientificExecutionStatus::Failed, ScientificAssessmentStatus::Passed)],
            "fastq.trim_reads",
        );
        assert_eq!(decision.verdict, EvidenceVerdict::Blocked);
    }
}
