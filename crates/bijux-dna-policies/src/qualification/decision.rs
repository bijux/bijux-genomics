use super::EvidenceDecision;

pub const SOURCE_IDENTITY_RULE_V1: &str = "qualification.source_identity.v1";
pub const EXECUTION_EVIDENCE_RULE_V1: &str = "qualification.execution_evidence.v1";
pub const BENCHMARK_EVIDENCE_RULE_V1: &str = "qualification.benchmark_evidence.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionEvidenceStatus {
    Passed,
    Failed,
    Blocked,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEvidenceObservation {
    pub run_id: String,
    pub stage: String,
    pub evidence_sha256: String,
    pub status: ExecutionEvidenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkEvidenceObservation {
    pub run_id: String,
    pub repository_commit: String,
    pub image_sha256: String,
    pub evidence_sha256: String,
    pub report_passed: bool,
    pub tool_passed: bool,
}

#[must_use]
pub fn evaluate_source_identity(
    source_sha256: &str,
    pinned_commit: &str,
    pinned_commit_evidence_sha256: &str,
) -> EvidenceDecision {
    if valid_sha256(source_sha256) && !placeholder_identity(source_sha256) {
        return EvidenceDecision::passed(source_sha256.to_string());
    }
    if valid_git_commit(pinned_commit)
        && !placeholder_identity(pinned_commit)
        && valid_sha256(pinned_commit_evidence_sha256)
    {
        return EvidenceDecision::passed(pinned_commit_evidence_sha256.to_string());
    }
    EvidenceDecision::missing("catalog source identity is unresolved")
}

#[must_use]
pub fn evaluate_execution_evidence(
    observations: &[ExecutionEvidenceObservation],
    required_stage: &str,
) -> EvidenceDecision {
    let mut matching = observations
        .iter()
        .filter(|observation| observation.stage == required_stage)
        .collect::<Vec<_>>();
    matching.sort_by(|left, right| left.run_id.cmp(&right.run_id));

    if let Some(observation) = matching
        .iter()
        .copied()
        .find(|observation| observation.status == ExecutionEvidenceStatus::Passed)
    {
        return EvidenceDecision::passed(observation.evidence_sha256.clone());
    }
    if matching.iter().any(|observation| observation.status == ExecutionEvidenceStatus::Failed) {
        return EvidenceDecision::failed("all completed declared-stage executions failed");
    }
    if !matching.is_empty() {
        return EvidenceDecision::blocked("declared-stage execution is blocked");
    }
    EvidenceDecision::missing("no immutable run evidence matches the declared real-test stage")
}

#[must_use]
pub fn evaluate_benchmark_evidence(
    observations: &[BenchmarkEvidenceObservation],
    required_repository_commit: &str,
    required_image_sha256: &str,
) -> EvidenceDecision {
    let mut current = observations
        .iter()
        .filter(|observation| {
            observation.repository_commit == required_repository_commit
                && observation.image_sha256 == required_image_sha256
        })
        .collect::<Vec<_>>();
    current.sort_by(|left, right| left.run_id.cmp(&right.run_id));

    if let Some(observation) = current
        .iter()
        .copied()
        .find(|observation| observation.report_passed && observation.tool_passed)
    {
        return EvidenceDecision::passed(observation.evidence_sha256.clone());
    }
    if !current.is_empty() {
        return EvidenceDecision::failed("current-image benchmark evidence failed");
    }
    EvidenceDecision::missing(
        "no admitted benchmark report covers the current image and repository commit",
    )
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

fn valid_git_commit(value: &str) -> bool {
    value.len() == 40
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

fn placeholder_identity(value: &str) -> bool {
    value.is_empty()
        || value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_source_identity,
        BenchmarkEvidenceObservation, ExecutionEvidenceObservation, ExecutionEvidenceStatus,
    };
    use crate::qualification::EvidenceVerdict;

    #[test]
    fn source_identity_rejects_placeholders_and_invalid_fallback_evidence() {
        let decision = evaluate_source_identity(&"a".repeat(64), &"b".repeat(40), "invalid");
        assert_eq!(decision.verdict, EvidenceVerdict::Missing);
    }

    #[test]
    fn execution_evidence_does_not_accept_a_different_stage() {
        let observation = ExecutionEvidenceObservation {
            run_id: "run-1001".to_string(),
            stage: "fastq.read_qc".to_string(),
            evidence_sha256: "a".repeat(64),
            status: ExecutionEvidenceStatus::Passed,
        };
        let decision = evaluate_execution_evidence(&[observation], "fastq.trim_reads");
        assert_eq!(decision.verdict, EvidenceVerdict::Missing);
    }

    #[test]
    fn benchmark_evidence_requires_report_and_tool_passes() {
        let observation = BenchmarkEvidenceObservation {
            run_id: "run-1001".to_string(),
            repository_commit: "b".repeat(40),
            image_sha256: "c".repeat(64),
            evidence_sha256: "d".repeat(64),
            report_passed: true,
            tool_passed: false,
        };
        let decision =
            evaluate_benchmark_evidence(&[observation], &"b".repeat(40), &"c".repeat(64));
        assert_eq!(decision.verdict, EvidenceVerdict::Failed);
    }

    #[test]
    fn benchmark_evidence_rejects_stale_identity_even_when_passed() {
        let observation = BenchmarkEvidenceObservation {
            run_id: "run-1001".to_string(),
            repository_commit: "b".repeat(40),
            image_sha256: "c".repeat(64),
            evidence_sha256: "d".repeat(64),
            report_passed: true,
            tool_passed: true,
        };
        let decision =
            evaluate_benchmark_evidence(&[observation], &"e".repeat(40), &"c".repeat(64));
        assert_eq!(decision.verdict, EvidenceVerdict::Missing);
    }
}
