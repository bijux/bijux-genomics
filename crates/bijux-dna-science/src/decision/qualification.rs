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

    if let Some(observation) = executed
        .iter()
        .copied()
        .find(|observation| observation.scientific_status == ScientificAssessmentStatus::Passed)
    {
        return EvidenceDecision::passed(observation.evidence_sha256.clone());
    }
    if executed
        .iter()
        .any(|observation| observation.scientific_status == ScientificAssessmentStatus::Failed)
    {
        return EvidenceDecision::failed(
            "scientific acceptance failed for the declared-stage execution",
        );
    }
    if !executed.is_empty() {
        return EvidenceDecision::missing(
            "declared-stage execution has no scientific acceptance assessment",
        );
    }
    EvidenceDecision::blocked("scientific acceptance requires a passing declared-stage execution")
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
