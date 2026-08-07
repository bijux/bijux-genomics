use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench::{AdmittedBenchmarkStageReport, BenchmarkReportVerdict};
use bijux_dna_containers::evidence::{
    ContainerCatalogTool, ContainerEvidenceChain, ContainerImageInventoryTool,
};
use bijux_dna_policies::{
    evaluate_benchmark_evidence, evaluate_execution_evidence, evaluate_source_identity,
    BenchmarkEvidenceObservation, EvidenceDecision, EvidenceDimension,
    ExecutionEvidenceObservation, ExecutionEvidenceStatus, PublicationDisposition,
    QualificationAssessment, QualificationEvidenceSource, QualificationReport, SupportDisposition,
    REQUIRED_EVIDENCE_DIMENSIONS,
};
use bijux_dna_runtime::{
    load_run_evidence_index, AttemptExecutionStatus, RunEvidenceRecord, ScientificAcceptanceStatus,
};
use bijux_dna_science::decision::{
    evaluate_scientific_acceptance, ScientificAcceptanceObservation,
    ScientificAssessmentStatus as ScienceAssessmentStatus,
    ScientificExecutionStatus as ScienceExecutionStatus,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const INVENTORY_PATH: &str = "containers/reports/image-inventory.json";
const RUNTIME_PATH: &str = "containers/reports/runtime-verification.json";
const RUNS_PATH: &str = "containers/reports/run-evidence.json";

struct QualificationContext<'a> {
    benchmark_reports: &'a [AdmittedBenchmarkStageReport],
    chain: &'a ContainerEvidenceChain,
    freshness_sha256: &'a str,
    runtime_sha256: &'a str,
}

/// Compose a current qualification report from admitted container, runtime, and benchmark facts.
///
/// # Errors
/// Returns an error when an evidence source is missing, stale, malformed, or internally
/// inconsistent.
pub fn derive_container_qualification_report(
    repository_root: &Path,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
) -> Result<QualificationReport> {
    let inventory_source = repository_source(repository_root, INVENTORY_PATH)?;
    let runtime_source = repository_source(repository_root, RUNTIME_PATH)?;
    let runs_source = repository_source(repository_root, RUNS_PATH)?;
    let runtime_sha256 = runtime_source.sha256.clone();
    let mut sources = BTreeMap::from([
        (
            "catalog".to_string(),
            QualificationEvidenceSource {
                uri: format!("repository://{CATALOG_PATH}"),
                sha256: chain.catalog_sha256.clone(),
            },
        ),
        ("inventory".to_string(), inventory_source),
        ("runs".to_string(), runs_source),
        ("runtime".to_string(), runtime_source),
    ]);
    for admission in benchmark_reports {
        sources.insert(
            format!("benchmark:{}", admission.report.run_id),
            QualificationEvidenceSource {
                uri: format!("repository://benchmarks/reports/{}", admission.report_file_name),
                sha256: admission.source_sha256.clone(),
            },
        );
    }
    let freshness_sha256 = freshness_identity(&chain.repository_commit, &sources)?;
    let context = QualificationContext {
        benchmark_reports,
        chain,
        freshness_sha256: &freshness_sha256,
        runtime_sha256: &runtime_sha256,
    };
    let runs = load_run_evidence_index(&repository_root.join(RUNS_PATH))?;
    let inventory_tools = chain
        .inventory
        .tools
        .iter()
        .map(|tool| (tool.tool.as_str(), tool))
        .collect::<BTreeMap<_, _>>();
    let run_records = runs.runs.iter().fold(
        BTreeMap::<&str, Vec<&RunEvidenceRecord>>::new(),
        |mut records, run| {
            records.entry(run.tool.as_str()).or_default().push(run);
            records
        },
    );
    let assessments = chain
        .catalog
        .tools
        .iter()
        .map(|tool| {
            derive_tool_assessment(
                tool,
                inventory_tools.get(tool.id.as_str()).copied(),
                run_records.get(tool.id.as_str()).map(Vec::as_slice).unwrap_or_default(),
                &context,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    QualificationReport::v2(sources, assessments)
}

fn derive_tool_assessment(
    tool: &ContainerCatalogTool,
    inventory: Option<&ContainerImageInventoryTool>,
    runs: &[&RunEvidenceRecord],
    context: &QualificationContext<'_>,
) -> Result<QualificationAssessment> {
    if tool.support_disposition != "retained" {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| (dimension, not_applicable()))
            .collect();
        return QualificationAssessment::v1(
            tool.id.clone(),
            SupportDisposition::Unsupported,
            PublicationDisposition::Current,
            dimensions,
        );
    }

    let inventory = inventory.ok_or_else(|| {
        anyhow::anyhow!("qualification inventory omits retained tool {}", tool.id)
    })?;
    let image_sha256 = inventory
        .selected_image_sha256
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("qualification inventory has no image for {}", tool.id))?;
    let execution_observations = execution_observations(runs);
    let scientific_observations = scientific_observations(runs);
    let stage = evaluate_execution_evidence(&execution_observations, &tool.real_test_stage);
    let science = evaluate_scientific_acceptance(&scientific_observations, &tool.real_test_stage);
    let dimensions = BTreeMap::from([
        (
            EvidenceDimension::Catalog,
            EvidenceDecision::passed(context.chain.catalog_sha256.clone()),
        ),
        (
            EvidenceDimension::Source,
            evaluate_source_identity(
                &tool.source_sha256,
                &tool.pinned_commit,
                &bijux_dna_infra::hash_bytes_sha256(
                    format!("git:{}", tool.pinned_commit).as_bytes(),
                ),
            ),
        ),
        (EvidenceDimension::Build, EvidenceDecision::passed(image_sha256.to_string())),
        (EvidenceDimension::Runtime, EvidenceDecision::passed(context.runtime_sha256.to_string())),
        (EvidenceDimension::Stage, stage.clone()),
        (EvidenceDimension::RealData, stage),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            evaluate_benchmark_evidence(
                &benchmark_observations(&tool.id, context.benchmark_reports),
                &context.chain.repository_commit,
                image_sha256,
            ),
        ),
        (
            EvidenceDimension::Security,
            EvidenceDecision::missing("no admitted security publication covers this image"),
        ),
        (
            EvidenceDimension::Freshness,
            EvidenceDecision::passed(context.freshness_sha256.to_string()),
        ),
        (
            EvidenceDimension::Release,
            EvidenceDecision::missing("no immutable release publication covers this evidence set"),
        ),
    ]);
    QualificationAssessment::v1(
        tool.id.clone(),
        SupportDisposition::Retained,
        PublicationDisposition::Current,
        dimensions,
    )
}

fn benchmark_observations(
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

fn execution_observations(runs: &[&RunEvidenceRecord]) -> Vec<ExecutionEvidenceObservation> {
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

fn scientific_observations(runs: &[&RunEvidenceRecord]) -> Vec<ScientificAcceptanceObservation> {
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

fn repository_source(
    repository_root: &Path,
    relative: &str,
) -> Result<QualificationEvidenceSource> {
    let bytes = fs::read(repository_root.join(relative))
        .with_context(|| format!("read qualification source {relative}"))?;
    Ok(QualificationEvidenceSource {
        uri: format!("repository://{relative}"),
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    })
}

fn freshness_identity(
    repository_commit: &str,
    sources: &BTreeMap<String, QualificationEvidenceSource>,
) -> Result<String> {
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&(repository_commit, sources))?))
}

fn not_applicable() -> EvidenceDecision {
    EvidenceDecision::not_applicable()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::freshness_identity;
    use bijux_dna_policies::QualificationEvidenceSource;

    #[test]
    fn freshness_identity_covers_benchmark_sources() -> anyhow::Result<()> {
        let mut sources = BTreeMap::from([(
            "benchmark:run-1001".to_string(),
            QualificationEvidenceSource {
                uri: "repository://benchmarks/reports/run-1001.json".to_string(),
                sha256: "a".repeat(64),
            },
        )]);
        let original = freshness_identity(&"b".repeat(40), &sources)?;
        let source = sources
            .get_mut("benchmark:run-1001")
            .ok_or_else(|| anyhow::anyhow!("benchmark source fixture is absent"))?;
        source.sha256 = "c".repeat(64);
        assert_ne!(original, freshness_identity(&"b".repeat(40), &sources)?);
        Ok(())
    }
}
