//! Owner: benchmark qualification composition.
//! Container qualification observations from admitted evidence.
use crate::{AdmittedBenchmarkStageReport, BenchmarkReportVerdict};
use bijux_dna_containers::evidence::{
    AttemptExecutionStatus, RunEvidenceRecord, ScientificAcceptanceStatus,
};
use bijux_dna_policies::{
    BenchmarkEvidenceObservation, ExecutionEvidenceObservation, ExecutionEvidenceStatus,
};
use bijux_dna_science::decision::{
    ScientificAcceptanceObservation, ScientificAssessmentStatus as ScienceAssessmentStatus,
    ScientificExecutionStatus as ScienceExecutionStatus,
};

pub(super) fn benchmark_observations(
    tool_id: &str,
    reports: &[AdmittedBenchmarkStageReport],
) -> Vec<BenchmarkEvidenceObservation> {
    reports
        .iter()
        .filter_map(|admission| {
            let tool = admission.report.tools.iter().find(|tool| tool.tool == tool_id)?;
            Some(BenchmarkEvidenceObservation {
                run_id: admission.report.run_id.clone(),
                repository_commit: tool.repository_commit.clone(),
                image_sha256: tool.image.sha256.clone(),
                evidence_sha256: admission.source_sha256.clone(),
                report_passed: admission.report.verdict == BenchmarkReportVerdict::Passed,
                tool_passed: tool.verdict == BenchmarkReportVerdict::Passed,
            })
        })
        .collect()
}

pub(super) fn execution_observations(
    runs: &[&RunEvidenceRecord],
) -> Vec<ExecutionEvidenceObservation> {
    runs.iter()
        .map(|run| ExecutionEvidenceObservation {
            run_id: run.run_id.clone(),
            stage: run.stage.clone(),
            evidence_sha256: run.evidence_lock_sha256.clone(),
            status: match run.execution_status {
                AttemptExecutionStatus::Passed => ExecutionEvidenceStatus::Passed,
                AttemptExecutionStatus::Failed => ExecutionEvidenceStatus::Failed,
                AttemptExecutionStatus::Blocked => ExecutionEvidenceStatus::Blocked,
                AttemptExecutionStatus::Incomplete => ExecutionEvidenceStatus::Incomplete,
            },
        })
        .collect()
}

pub(super) fn scientific_observations(
    runs: &[&RunEvidenceRecord],
) -> Vec<ScientificAcceptanceObservation> {
    runs.iter()
        .map(|run| ScientificAcceptanceObservation {
            run_id: run.run_id.clone(),
            stage: run.stage.clone(),
            evidence_sha256: run.evidence_lock_sha256.clone(),
            execution_status: match run.execution_status {
                AttemptExecutionStatus::Passed => ScienceExecutionStatus::Passed,
                AttemptExecutionStatus::Failed => ScienceExecutionStatus::Failed,
                AttemptExecutionStatus::Blocked => ScienceExecutionStatus::Blocked,
                AttemptExecutionStatus::Incomplete => ScienceExecutionStatus::Incomplete,
            },
            scientific_status: match run.scientific_status {
                ScientificAcceptanceStatus::Passed => ScienceAssessmentStatus::Passed,
                ScientificAcceptanceStatus::Failed => ScienceAssessmentStatus::Failed,
                ScientificAcceptanceStatus::NotAssessed => ScienceAssessmentStatus::NotAssessed,
            },
        })
        .collect()
}
