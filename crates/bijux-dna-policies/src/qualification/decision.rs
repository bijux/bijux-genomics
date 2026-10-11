use std::collections::BTreeMap;

use super::EvidenceDecision;

pub const SOURCE_IDENTITY_RULE_V1: &str = "qualification.source_identity.v1";
pub const EXECUTION_EVIDENCE_RULE_V1: &str = "qualification.execution_evidence.v1";
pub const BENCHMARK_EVIDENCE_RULE_V1: &str = "qualification.benchmark_evidence.v1";
pub const CATALOG_EVIDENCE_RULE_V1: &str = "qualification.catalog_evidence.v1";
pub const BUILD_EVIDENCE_RULE_V1: &str = "qualification.build_evidence.v1";
pub const RUNTIME_EVIDENCE_RULE_V1: &str = "qualification.runtime_evidence.v1";
pub const FRESHNESS_EVIDENCE_RULE_V1: &str = "qualification.bound_evidence_freshness.v1";
pub const SECURITY_EVIDENCE_RULE_V1: &str = "qualification.security_evidence.v1";
pub const RELEASE_EVIDENCE_RULE_V1: &str = "qualification.release_evidence.v1";
pub const SUPPORT_DISPOSITION_RULE_V1: &str = "qualification.support_disposition.v1";
pub const SUPPORT_DIMENSIONS_RULE_V2: &str = "qualification.support_dimensions.v2";

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
    let measured_values = BTreeMap::from([
        ("source_sha256".to_string(), display_value(source_sha256)),
        ("pinned_commit".to_string(), display_value(pinned_commit)),
        ("pinned_commit_evidence_sha256".to_string(), display_value(pinned_commit_evidence_sha256)),
    ]);
    let thresholds = BTreeMap::from([
        ("source_sha256".to_string(), "non-placeholder SHA-256".to_string()),
        ("pinned_commit".to_string(), "non-placeholder full Git commit".to_string()),
    ]);
    if valid_sha256(source_sha256) && !placeholder_identity(source_sha256) {
        return EvidenceDecision::passed(
            SOURCE_IDENTITY_RULE_V1,
            measured_values,
            thresholds,
            source_sha256.to_string(),
            "catalog source SHA-256 satisfies the immutable identity rule",
        );
    }
    if valid_git_commit(pinned_commit)
        && !placeholder_identity(pinned_commit)
        && valid_sha256(pinned_commit_evidence_sha256)
    {
        return EvidenceDecision::passed(
            SOURCE_IDENTITY_RULE_V1,
            measured_values,
            thresholds,
            pinned_commit_evidence_sha256.to_string(),
            "catalog pinned commit satisfies the immutable identity rule",
        );
    }
    EvidenceDecision::missing(
        SOURCE_IDENTITY_RULE_V1,
        measured_values,
        thresholds,
        "catalog source identity is unresolved",
    )
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
    let measured_values = BTreeMap::from([
        ("required_stage".to_string(), required_stage.to_string()),
        ("matching_run_count".to_string(), matching.len().to_string()),
        (
            "matching_run_statuses".to_string(),
            if matching.is_empty() {
                "none".to_string()
            } else {
                matching
                    .iter()
                    .map(|observation| {
                        format!("{}:{}", observation.run_id, status_name(observation.status))
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            },
        ),
    ]);
    let thresholds = BTreeMap::from([
        ("stage_identity".to_string(), "exact match".to_string()),
        ("passing_execution_count".to_string(), ">=1".to_string()),
    ]);

    if let Some(observation) = matching
        .iter()
        .copied()
        .find(|observation| observation.status == ExecutionEvidenceStatus::Passed)
    {
        return EvidenceDecision::passed(
            EXECUTION_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            observation.evidence_sha256.clone(),
            "a declared-stage execution has immutable passing evidence",
        );
    }
    if matching.iter().any(|observation| observation.status == ExecutionEvidenceStatus::Failed) {
        return EvidenceDecision::failed(
            EXECUTION_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            "all completed declared-stage executions failed",
        );
    }
    if !matching.is_empty() {
        return EvidenceDecision::blocked(
            EXECUTION_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            "declared-stage execution is blocked or incomplete",
        );
    }
    EvidenceDecision::missing(
        EXECUTION_EVIDENCE_RULE_V1,
        measured_values,
        thresholds,
        "no immutable run evidence matches the declared real-test stage",
    )
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
    let passing_count = current
        .iter()
        .filter(|observation| observation.report_passed && observation.tool_passed)
        .count();
    let measured_values = BTreeMap::from([
        ("required_repository_commit".to_string(), required_repository_commit.to_string()),
        ("required_image_sha256".to_string(), required_image_sha256.to_string()),
        ("current_evidence_count".to_string(), current.len().to_string()),
        ("passing_evidence_count".to_string(), passing_count.to_string()),
    ]);
    let thresholds = BTreeMap::from([
        ("identity_match".to_string(), "repository commit and image SHA-256".to_string()),
        ("report_verdict".to_string(), "passed".to_string()),
        ("tool_verdict".to_string(), "passed".to_string()),
    ]);

    if let Some(observation) = current
        .iter()
        .copied()
        .find(|observation| observation.report_passed && observation.tool_passed)
    {
        return EvidenceDecision::passed(
            BENCHMARK_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            observation.evidence_sha256.clone(),
            "current benchmark report and tool verdict both pass",
        );
    }
    if !current.is_empty() {
        return EvidenceDecision::failed(
            BENCHMARK_EVIDENCE_RULE_V1,
            measured_values,
            thresholds,
            "current-image benchmark evidence failed",
        );
    }
    EvidenceDecision::missing(
        BENCHMARK_EVIDENCE_RULE_V1,
        measured_values,
        thresholds,
        "no admitted benchmark report covers the current image and repository commit",
    )
}

#[must_use]
pub fn evaluate_identity_evidence(
    rule_version: &str,
    identity_label: &str,
    evidence_sha256: &str,
) -> EvidenceDecision {
    let measured_values =
        BTreeMap::from([(identity_label.to_string(), display_value(evidence_sha256))]);
    let thresholds = BTreeMap::from([(identity_label.to_string(), "non-zero SHA-256".to_string())]);
    if valid_sha256(evidence_sha256) {
        EvidenceDecision::passed(
            rule_version,
            measured_values,
            thresholds,
            evidence_sha256.to_string(),
            "governed evidence identity satisfies the required digest rule",
        )
    } else {
        EvidenceDecision::missing(
            rule_version,
            measured_values,
            thresholds,
            "governed evidence identity is absent or invalid",
        )
    }
}

fn status_name(status: ExecutionEvidenceStatus) -> &'static str {
    match status {
        ExecutionEvidenceStatus::Passed => "passed",
        ExecutionEvidenceStatus::Failed => "failed",
        ExecutionEvidenceStatus::Blocked => "blocked",
        ExecutionEvidenceStatus::Incomplete => "incomplete",
    }
}

fn display_value(value: &str) -> String {
    if value.is_empty() {
        "missing".to_string()
    } else {
        value.to_string()
    }
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
