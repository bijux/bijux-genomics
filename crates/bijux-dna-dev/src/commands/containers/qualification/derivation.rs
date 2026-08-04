use std::collections::BTreeMap;
use std::fs;

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::qualification::{
    EvidenceDecision, EvidenceDimension, EvidenceVerdict, PublicationDisposition,
    QualificationAssessment, QualificationEvidenceSource, QualificationReport, SupportDisposition,
    REQUIRED_EVIDENCE_DIMENSIONS,
};
use bijux_dna_bench::{AdmittedBenchmarkStageReport, BenchmarkReportVerdict};
use bijux_dna_environment::evidence::{
    ContainerCatalogTool, ContainerEvidenceChain, ContainerImageInventoryTool,
};
use bijux_dna_runtime::{
    load_run_evidence_index, AttemptExecutionStatus, RunEvidenceRecord, ScientificAcceptanceStatus,
};

use super::super::Workspace;

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

pub(in crate::commands::containers) fn derive_qualification_report(
    workspace: &Workspace,
    chain: &ContainerEvidenceChain,
    benchmark_reports: &[AdmittedBenchmarkStageReport],
) -> Result<QualificationReport> {
    let inventory_source = repository_source(workspace, INVENTORY_PATH)?;
    let runtime_source = repository_source(workspace, RUNTIME_PATH)?;
    let runs_source = repository_source(workspace, RUNS_PATH)?;
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
    let runs = load_run_evidence_index(&workspace.path(RUNS_PATH))?;
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
    let stage = execution_decision(runs, &tool.real_test_stage);
    let science = science_decision(runs, &tool.real_test_stage);
    let dimensions = BTreeMap::from([
        (EvidenceDimension::Catalog, passed(context.chain.catalog_sha256.clone())),
        (EvidenceDimension::Source, source_decision(tool)),
        (EvidenceDimension::Build, passed(image_sha256.to_string())),
        (EvidenceDimension::Runtime, passed(context.runtime_sha256.to_string())),
        (EvidenceDimension::Stage, stage.clone()),
        (EvidenceDimension::RealData, stage),
        (EvidenceDimension::Science, science),
        (
            EvidenceDimension::Benchmark,
            benchmark_decision(
                &tool.id,
                image_sha256,
                &context.chain.repository_commit,
                context.benchmark_reports,
            ),
        ),
        (
            EvidenceDimension::Security,
            missing("no admitted security publication covers this image"),
        ),
        (EvidenceDimension::Freshness, passed(context.freshness_sha256.to_string())),
        (
            EvidenceDimension::Release,
            missing("no immutable release publication covers this evidence set"),
        ),
    ]);
    QualificationAssessment::v1(
        tool.id.clone(),
        SupportDisposition::Retained,
        PublicationDisposition::Current,
        dimensions,
    )
}

fn source_decision(tool: &ContainerCatalogTool) -> EvidenceDecision {
    if valid_sha256(&tool.source_sha256) && !placeholder_identity(&tool.source_sha256) {
        return passed(tool.source_sha256.clone());
    }
    if valid_git_commit(&tool.pinned_commit) && !placeholder_identity(&tool.pinned_commit) {
        return passed(bijux_dna_infra::hash_bytes_sha256(
            format!("git:{}", tool.pinned_commit).as_bytes(),
        ));
    }
    missing("catalog source identity is unresolved")
}

fn execution_decision(runs: &[&RunEvidenceRecord], stage: &str) -> EvidenceDecision {
    let matching = runs.iter().copied().filter(|run| run.stage == stage).collect::<Vec<_>>();
    if let Some(run) = matching
        .iter()
        .copied()
        .filter(|run| run.execution_status == AttemptExecutionStatus::Passed)
        .min_by(|left, right| left.run_id.cmp(&right.run_id))
    {
        return passed(run.evidence_lock_sha256.clone());
    }
    if matching.iter().any(|run| run.execution_status == AttemptExecutionStatus::Failed) {
        return failed("all completed declared-stage executions failed");
    }
    if !matching.is_empty() {
        return blocked("declared-stage execution is blocked");
    }
    missing("no immutable run evidence matches the declared real-test stage")
}

fn science_decision(runs: &[&RunEvidenceRecord], stage: &str) -> EvidenceDecision {
    let mut executed = runs
        .iter()
        .copied()
        .filter(|run| run.stage == stage && run.execution_status == AttemptExecutionStatus::Passed)
        .collect::<Vec<_>>();
    executed.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    if let Some(run) = executed
        .iter()
        .copied()
        .find(|run| run.scientific_status == ScientificAcceptanceStatus::Passed)
    {
        return passed(run.evidence_lock_sha256.clone());
    }
    if executed.iter().any(|run| run.scientific_status == ScientificAcceptanceStatus::Failed) {
        return failed("scientific acceptance failed for the declared-stage execution");
    }
    if !executed.is_empty() {
        return missing("declared-stage execution has no scientific acceptance assessment");
    }
    blocked("scientific acceptance requires a passing declared-stage execution")
}

fn benchmark_decision(
    tool_id: &str,
    image_sha256: &str,
    repository_commit: &str,
    reports: &[AdmittedBenchmarkStageReport],
) -> EvidenceDecision {
    let mut current = reports
        .iter()
        .filter_map(|admission| {
            let tool = admission.report.tools.iter().find(|tool| tool.tool == tool_id)?;
            (tool.repository_commit == repository_commit && tool.image.sha256 == image_sha256)
                .then_some((admission, tool))
        })
        .collect::<Vec<_>>();
    current.sort_by(|(left, _), (right, _)| left.report.run_id.cmp(&right.report.run_id));
    if let Some((admission, _)) = current.iter().copied().find(|(admission, tool)| {
        admission.report.verdict == BenchmarkReportVerdict::Passed
            && tool.verdict == BenchmarkReportVerdict::Passed
    }) {
        return passed(admission.source_sha256.clone());
    }
    if !current.is_empty() {
        return failed("current-image benchmark evidence failed");
    }
    missing("no admitted benchmark report covers the current image and repository commit")
}

fn repository_source(workspace: &Workspace, relative: &str) -> Result<QualificationEvidenceSource> {
    let bytes = fs::read(workspace.path(relative))
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

fn passed(evidence_sha256: String) -> EvidenceDecision {
    EvidenceDecision {
        verdict: EvidenceVerdict::Passed,
        evidence_sha256: Some(evidence_sha256),
        reasons: Vec::new(),
    }
}

fn failed(reason: &str) -> EvidenceDecision {
    EvidenceDecision {
        verdict: EvidenceVerdict::Failed,
        evidence_sha256: None,
        reasons: vec![reason.to_string()],
    }
}

fn blocked(reason: &str) -> EvidenceDecision {
    EvidenceDecision {
        verdict: EvidenceVerdict::Blocked,
        evidence_sha256: None,
        reasons: vec![reason.to_string()],
    }
}

fn missing(reason: &str) -> EvidenceDecision {
    EvidenceDecision {
        verdict: EvidenceVerdict::Missing,
        evidence_sha256: None,
        reasons: vec![reason.to_string()],
    }
}

fn not_applicable() -> EvidenceDecision {
    EvidenceDecision {
        verdict: EvidenceVerdict::NotApplicable,
        evidence_sha256: None,
        reasons: Vec::new(),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_git_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn placeholder_identity(value: &str) -> bool {
    value.is_empty()
        || value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{execution_decision, freshness_identity, science_decision};
    use bijux_dna_api::v1::api::qualification::{EvidenceVerdict, QualificationEvidenceSource};
    use bijux_dna_runtime::{
        AttemptExecutionStatus, RunEvidenceRecord, ScientificAcceptanceStatus,
    };

    fn run(
        run_id: &str,
        execution_status: AttemptExecutionStatus,
        scientific_status: ScientificAcceptanceStatus,
    ) -> RunEvidenceRecord {
        RunEvidenceRecord {
            campaign: "campaign".to_string(),
            corpus: "corpus".to_string(),
            domain: "fastq".to_string(),
            evidence_lock_sha256: "a".repeat(64),
            execution_status,
            pipeline_schema_version: "bijux.pipeline.v1".to_string(),
            repository_commit: "b".repeat(40),
            run_id: run_id.to_string(),
            sample: "sample".to_string(),
            scientific_status,
            slurm_job_id: "1234".to_string(),
            source_file_count: 1,
            source_result: "study://runs/result".to_string(),
            stage: "fastq.trim_reads".to_string(),
            tool: "atropos".to_string(),
        }
    }

    #[test]
    fn execution_decision_preserves_pass_failure_and_blocking() {
        let passed_run = run(
            "run-1001",
            AttemptExecutionStatus::Passed,
            ScientificAcceptanceStatus::NotAssessed,
        );
        assert_eq!(
            execution_decision(&[&passed_run], "fastq.trim_reads").verdict,
            EvidenceVerdict::Passed
        );
        let failed_run = run(
            "run-1002",
            AttemptExecutionStatus::Failed,
            ScientificAcceptanceStatus::NotAssessed,
        );
        assert_eq!(
            execution_decision(&[&failed_run], "fastq.trim_reads").verdict,
            EvidenceVerdict::Failed
        );
        let blocked_run = run(
            "run-1003",
            AttemptExecutionStatus::Blocked,
            ScientificAcceptanceStatus::NotAssessed,
        );
        assert_eq!(
            execution_decision(&[&blocked_run], "fastq.trim_reads").verdict,
            EvidenceVerdict::Blocked
        );
    }

    #[test]
    fn science_decision_does_not_infer_acceptance_from_execution() {
        let unassessed = run(
            "run-1001",
            AttemptExecutionStatus::Passed,
            ScientificAcceptanceStatus::NotAssessed,
        );
        assert_eq!(
            science_decision(&[&unassessed], "fastq.trim_reads").verdict,
            EvidenceVerdict::Missing
        );
        let accepted =
            run("run-1002", AttemptExecutionStatus::Passed, ScientificAcceptanceStatus::Passed);
        assert_eq!(
            science_decision(&[&accepted], "fastq.trim_reads").verdict,
            EvidenceVerdict::Passed
        );
    }

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
        sources.get_mut("benchmark:run-1001").expect("benchmark source fixture").sha256 =
            "c".repeat(64);
        assert_ne!(original, freshness_identity(&"b".repeat(40), &sources)?);
        Ok(())
    }
}
