use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_SEX_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.local_smoke.report.v1";
const LOCAL_SEX_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.sex.local_smoke.metrics.v1";
const LOCAL_SEX_TOOL_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.tool_smoke.report.v1";
const LOCAL_SEX_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.sex.stage_metrics.v1";
const SEX_TOOL_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.v1";
const INSUFFICIENT_SEX_SAMPLE_ID: &str = "adna_y_haplogroup_panel";
const INSUFFICIENT_SEX_BAM_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_y_haplogroup_panel.sam";

#[derive(Debug, Clone, Copy)]
enum ProofCaseKind {
    Ready,
    Insufficient,
}

impl ProofCaseKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Insufficient => "insufficient",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct LocalSexSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    method: String,
    chromosome_system: Option<String>,
    minimum_y_sites: Option<u32>,
    x_coverage: f64,
    y_coverage: f64,
    autosomal_coverage: f64,
    x_to_y_ratio: Option<f64>,
    call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
    confidence: f64,
    status: String,
    insufficiency_reason: Option<String>,
    sex_report: String,
    sex_estimate: String,
    population_metrics: String,
    haplogroup_report: String,
    sex_summary: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize)]
struct LocalSexToolSmokeCaseReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    proof_case: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    method: String,
    chromosome_system: Option<String>,
    minimum_y_sites: Option<u32>,
    x_coverage: f64,
    y_coverage: f64,
    autosomal_coverage: f64,
    x_to_y_ratio: Option<f64>,
    call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
    confidence: f64,
    status: String,
    insufficiency_reason: Option<String>,
    sex_report: String,
    sex_estimate: String,
    population_metrics: String,
    haplogroup_report: String,
    sex_summary: String,
    stage_metrics: String,
    declared_output_ids: Vec<String>,
    artifact_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalSexToolSmokeReport {
    schema_version: String,
    stage_id: String,
    ready_sample_id: String,
    insufficient_sample_id: String,
    tool_ids: Vec<String>,
    case_count: usize,
    rows: Vec<LocalSexToolSmokeCaseReport>,
}

struct LocalSexOutputPaths {
    sex_report: PathBuf,
    sex_estimate: PathBuf,
    population_metrics: PathBuf,
    haplogroup_report: PathBuf,
    sex_summary: PathBuf,
    stage_metrics: PathBuf,
}

struct LocalSexExpectationDeltas {
    expectation_matched: bool,
    x_coverage_delta: f64,
    y_coverage_delta: f64,
    autosomal_coverage_delta: f64,
    confidence_delta: f64,
}

struct LocalSexToolExpectation {
    expectation_matched: bool,
    expected_call: String,
    expected_confidence: f64,
    expected_status: String,
    expected_insufficiency_reason: Option<String>,
}

/// Materialize the governed local-smoke `bam.sex` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.sex/sex.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_sex_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_sex_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.sex expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.sex");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_sex_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("sex.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Materialize governed `bam.sex` tool-smoke outputs for all retained tools.
///
/// The written report lives at `runs/bench/local-smoke/bam.sex/tool_smoke.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if governed BAM sex output-contract plans cannot be built, if the insufficient
/// evidence fixture is missing, or if any proof artifact cannot be written.
pub fn write_local_sex_tool_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plans = bijux_dna_planner_bam::stage_api::local_sex_output_contract_plans(&repo_root)?;
    if plans.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.sex expects governed output-contract plans, found none"
        ));
    }

    let output_root = repo_root.join("runs/bench/local-smoke/bam.sex");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let ready_sample_id = plans[0].sample_id.clone();
    let mut tool_ids = Vec::with_capacity(plans.len());
    let mut rows = Vec::with_capacity(plans.len() * 2);
    for plan in &plans {
        tool_ids.push(plan.plan.tool_id.as_str().to_string());
        rows.push(materialize_local_sex_tool_case(&repo_root, plan, ProofCaseKind::Ready)?);
        rows.push(materialize_local_sex_tool_case(&repo_root, plan, ProofCaseKind::Insufficient)?);
    }
    tool_ids.sort();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.proof_case.cmp(&right.proof_case))
    });

    let report = LocalSexToolSmokeReport {
        schema_version: LOCAL_SEX_TOOL_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.sex".to_string(),
        ready_sample_id,
        insufficient_sample_id: INSUFFICIENT_SEX_SAMPLE_ID.to_string(),
        tool_ids,
        case_count: rows.len(),
        rows,
    };

    let report_path = output_root.join("tool_smoke.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable `bam.sex` report and summary artifacts beside stage outputs.
///
/// # Errors
/// Returns an error if the BAM fixture cannot be summarized or artifacts cannot be written.
pub(crate) fn write_stage_sex_artifacts(
    stage_dir: &Path,
    input_bam: &Path,
    reference_fasta: &Path,
    method: &str,
    chromosome_system: Option<&str>,
    minimum_y_sites: Option<u32>,
) -> Result<bijux_dna_domain_bam::BamSexSummaryV1> {
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
        input_bam,
        reference_fasta,
        method,
        chromosome_system,
        minimum_y_sites,
    )?;
    let report_path = stage_dir.join("sex.json");
    let summary_path = stage_dir.join("sex.summary.json");
    bijux_dna_infra::atomic_write_json(&report_path, &sex_tool_report(&summary))
        .with_context(|| format!("write {}", report_path.display()))?;
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    Ok(summary)
}

fn materialize_local_sex_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
) -> Result<LocalSexSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let output_paths = resolve_local_sex_output_paths(repo_root, &case.plan)?;
    let input_bam = repo_root.join(&case.bam);
    let reference_fasta = repo_root.join(&case.reference);

    let summary = write_stage_sex_artifacts(
        &case_out_dir,
        &input_bam,
        &reference_fasta,
        &case.expected_method,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    let deltas = local_sex_expectation(case, &summary);
    write_local_sex_support_artifacts(
        &output_paths,
        case.plan.tool_id.as_str(),
        None,
        &summary,
        "not_applicable_for_local_rxy_smoke",
    )?;
    write_local_sex_stage_metrics(&output_paths.stage_metrics, case, &summary, &deltas)?;

    Ok(LocalSexSmokeReport {
        schema_version: LOCAL_SEX_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.sex".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched: deltas.expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        method: summary.method.clone(),
        chromosome_system: summary.chromosome_system.clone(),
        minimum_y_sites: summary.minimum_y_sites,
        x_coverage: summary.x_coverage,
        y_coverage: summary.y_coverage,
        autosomal_coverage: summary.autosomal_coverage,
        x_to_y_ratio: summary.x_to_y_ratio,
        call: summary.call,
        confidence: summary.confidence,
        status: summary.status.clone(),
        insufficiency_reason: summary.insufficiency_reason.clone(),
        sex_report: path_relative_to_repo(repo_root, &output_paths.sex_report),
        sex_estimate: path_relative_to_repo(repo_root, &output_paths.sex_estimate),
        population_metrics: path_relative_to_repo(repo_root, &output_paths.population_metrics),
        haplogroup_report: path_relative_to_repo(repo_root, &output_paths.haplogroup_report),
        sex_summary: path_relative_to_repo(repo_root, &output_paths.sex_summary),
        stage_metrics: path_relative_to_repo(repo_root, &output_paths.stage_metrics),
    })
}

fn materialize_local_sex_tool_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
    proof_case: ProofCaseKind,
) -> Result<LocalSexToolSmokeCaseReport> {
    let tool_id = case.plan.tool_id.as_str();
    let reference_fasta = repo_root.join(&case.reference);
    let (sample_id, input_bam) = match proof_case {
        ProofCaseKind::Ready => (case.sample_id.clone(), repo_root.join(&case.bam)),
        ProofCaseKind::Insufficient => {
            (INSUFFICIENT_SEX_SAMPLE_ID.to_string(), repo_root.join(INSUFFICIENT_SEX_BAM_PATH))
        }
    };
    if !input_bam.is_file() {
        return Err(anyhow!("bam.sex tool-smoke BAM fixture is missing: {}", input_bam.display()));
    }

    let case_dir = repo_root
        .join("runs/bench/local-smoke/bam.sex")
        .join("tool_smoke")
        .join(tool_id)
        .join(proof_case.as_str());
    bijux_dna_infra::ensure_dir(&case_dir)?;

    let summary = write_stage_sex_artifacts(
        &case_dir,
        &input_bam,
        &reference_fasta,
        tool_id,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    let output_paths = LocalSexOutputPaths {
        sex_report: case_dir.join("sex.json"),
        sex_estimate: case_dir.join("sex_estimate.json"),
        population_metrics: case_dir.join("population_metrics.json"),
        haplogroup_report: case_dir.join("haplogroup_report.json"),
        sex_summary: case_dir.join("sex.summary.json"),
        stage_metrics: case_dir.join("stage.metrics.json"),
    };
    let expectation = local_sex_tool_expectation(case, &summary, proof_case, tool_id);
    write_local_sex_support_artifacts(
        &output_paths,
        tool_id,
        Some(proof_case),
        &summary,
        local_sex_tool_haplogroup_status(proof_case),
    )?;
    write_local_sex_tool_stage_metrics(
        &output_paths.stage_metrics,
        &sample_id,
        tool_id,
        proof_case,
        &summary,
        &expectation,
    )?;

    let artifact_paths = vec![
        path_relative_to_repo(repo_root, &output_paths.sex_report),
        path_relative_to_repo(repo_root, &output_paths.sex_estimate),
        path_relative_to_repo(repo_root, &output_paths.population_metrics),
        path_relative_to_repo(repo_root, &output_paths.haplogroup_report),
        path_relative_to_repo(repo_root, &output_paths.sex_summary),
        path_relative_to_repo(repo_root, &output_paths.stage_metrics),
    ];

    Ok(LocalSexToolSmokeCaseReport {
        schema_version: LOCAL_SEX_TOOL_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.sex".to_string(),
        tool_id: tool_id.to_string(),
        proof_case: proof_case.as_str().to_string(),
        sample_id,
        expectation_matched: expectation.expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        method: summary.method.clone(),
        chromosome_system: summary.chromosome_system.clone(),
        minimum_y_sites: summary.minimum_y_sites,
        x_coverage: summary.x_coverage,
        y_coverage: summary.y_coverage,
        autosomal_coverage: summary.autosomal_coverage,
        x_to_y_ratio: summary.x_to_y_ratio,
        call: summary.call,
        confidence: summary.confidence,
        status: summary.status.clone(),
        insufficiency_reason: summary.insufficiency_reason.clone(),
        sex_report: path_relative_to_repo(repo_root, &output_paths.sex_report),
        sex_estimate: path_relative_to_repo(repo_root, &output_paths.sex_estimate),
        population_metrics: path_relative_to_repo(repo_root, &output_paths.population_metrics),
        haplogroup_report: path_relative_to_repo(repo_root, &output_paths.haplogroup_report),
        sex_summary: path_relative_to_repo(repo_root, &output_paths.sex_summary),
        stage_metrics: path_relative_to_repo(repo_root, &output_paths.stage_metrics),
        declared_output_ids: case
            .plan
            .io
            .outputs
            .iter()
            .map(|artifact| artifact.name.as_str().to_string())
            .collect(),
        artifact_paths,
    })
}

fn resolve_local_sex_output_paths(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<LocalSexOutputPaths> {
    Ok(LocalSexOutputPaths {
        sex_report: resolve_output_path(repo_root, plan, "sex_report")?,
        sex_estimate: resolve_output_path(repo_root, plan, "sex_estimate")?,
        population_metrics: resolve_output_path(repo_root, plan, "population_metrics")?,
        haplogroup_report: resolve_output_path(repo_root, plan, "haplogroup_report")?,
        sex_summary: resolve_output_path(repo_root, plan, "summary")?,
        stage_metrics: resolve_output_path(repo_root, plan, "stage_metrics")?,
    })
}

fn local_sex_expectation(
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
    summary: &bijux_dna_domain_bam::BamSexSummaryV1,
) -> LocalSexExpectationDeltas {
    LocalSexExpectationDeltas {
        expectation_matched: summary.method == case.expected_method
            && summary.chromosome_system.as_deref() == Some(case.chromosome_system.as_str())
            && summary.minimum_y_sites == Some(case.minimum_y_sites)
            && float_matches(summary.x_coverage, case.expected_x_coverage)
            && float_matches(summary.y_coverage, case.expected_y_coverage)
            && float_matches(summary.autosomal_coverage, case.expected_autosomal_coverage)
            && summary.call == case.expected_call
            && float_matches(summary.confidence, case.expected_confidence)
            && summary.status == case.expected_status,
        x_coverage_delta: summary.x_coverage - case.expected_x_coverage,
        y_coverage_delta: summary.y_coverage - case.expected_y_coverage,
        autosomal_coverage_delta: summary.autosomal_coverage - case.expected_autosomal_coverage,
        confidence_delta: summary.confidence - case.expected_confidence,
    }
}

fn write_local_sex_support_artifacts(
    output_paths: &LocalSexOutputPaths,
    tool_id: &str,
    proof_case: Option<ProofCaseKind>,
    summary: &bijux_dna_domain_bam::BamSexSummaryV1,
    haplogroup_status: &str,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        &output_paths.sex_estimate,
        &serde_json::json!({
            "artifact_id": "sex_estimate",
            "stage_id": "bam.sex",
            "tool_id": tool_id,
            "proof_case": proof_case.map(ProofCaseKind::as_str),
            "call": summary.call,
            "confidence": summary.confidence,
            "status": summary.status,
            "x_to_y_ratio": summary.x_to_y_ratio,
            "insufficiency_reason": summary.insufficiency_reason,
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &output_paths.population_metrics,
        &serde_json::json!({
            "artifact_id": "population_metrics",
            "stage_id": "bam.sex",
            "tool_id": tool_id,
            "proof_case": proof_case.map(ProofCaseKind::as_str),
            "chromosome_system": summary.chromosome_system,
            "x_coverage": summary.x_coverage,
            "y_coverage": summary.y_coverage,
            "autosomal_coverage": summary.autosomal_coverage,
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &output_paths.haplogroup_report,
        &serde_json::json!({
            "artifact_id": "haplogroup_report",
            "stage_id": "bam.sex",
            "tool_id": tool_id,
            "proof_case": proof_case.map(ProofCaseKind::as_str),
            "status": haplogroup_status,
            "chromosome_system": summary.chromosome_system,
        }),
    )
}

fn write_local_sex_stage_metrics(
    path: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
    summary: &bijux_dna_domain_bam::BamSexSummaryV1,
    deltas: &LocalSexExpectationDeltas,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "schema_version": LOCAL_SEX_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.sex",
            "sample_id": case.sample_id,
            "expected_method": case.expected_method,
            "method": summary.method,
            "expected_chromosome_system": case.chromosome_system,
            "chromosome_system": summary.chromosome_system,
            "expected_minimum_y_sites": case.minimum_y_sites,
            "minimum_y_sites": summary.minimum_y_sites,
            "expected_x_coverage": case.expected_x_coverage,
            "x_coverage": summary.x_coverage,
            "x_coverage_delta": deltas.x_coverage_delta,
            "expected_y_coverage": case.expected_y_coverage,
            "y_coverage": summary.y_coverage,
            "y_coverage_delta": deltas.y_coverage_delta,
            "expected_autosomal_coverage": case.expected_autosomal_coverage,
            "autosomal_coverage": summary.autosomal_coverage,
            "autosomal_coverage_delta": deltas.autosomal_coverage_delta,
            "x_to_y_ratio": summary.x_to_y_ratio,
            "expected_call": case.expected_call,
            "call": summary.call,
            "expected_confidence": case.expected_confidence,
            "confidence": summary.confidence,
            "confidence_delta": deltas.confidence_delta,
            "expected_status": case.expected_status,
            "status": summary.status,
            "insufficiency_reason": summary.insufficiency_reason,
            "expectation_matched": deltas.expectation_matched,
        }),
    )
}

fn local_sex_tool_expectation(
    case: &bijux_dna_planner_bam::stage_api::LocalSexSmokeCasePlan,
    summary: &bijux_dna_domain_bam::BamSexSummaryV1,
    proof_case: ProofCaseKind,
    tool_id: &str,
) -> LocalSexToolExpectation {
    match proof_case {
        ProofCaseKind::Ready => LocalSexToolExpectation {
            expectation_matched: summary.method == case.expected_method
                && summary.chromosome_system.as_deref() == Some(case.chromosome_system.as_str())
                && summary.minimum_y_sites == Some(case.minimum_y_sites)
                && float_matches(summary.x_coverage, case.expected_x_coverage)
                && float_matches(summary.y_coverage, case.expected_y_coverage)
                && float_matches(summary.autosomal_coverage, case.expected_autosomal_coverage)
                && summary.call == case.expected_call
                && float_matches(summary.confidence, case.expected_confidence)
                && summary.status == case.expected_status
                && summary.insufficiency_reason.is_none(),
            expected_call: sex_call_name(case.expected_call).to_string(),
            expected_confidence: case.expected_confidence,
            expected_status: case.expected_status.clone(),
            expected_insufficiency_reason: None,
        },
        ProofCaseKind::Insufficient => LocalSexToolExpectation {
            expectation_matched: summary.method == tool_id
                && summary.call == bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient
                && float_matches(summary.confidence, 0.0)
                && summary.status == "insufficient_chromosomes"
                && summary.insufficiency_reason.as_deref() == Some("insufficient_chromosomes")
                && float_matches(summary.x_coverage, 0.0)
                && float_matches(summary.autosomal_coverage, 0.0),
            expected_call: "insufficient".to_string(),
            expected_confidence: 0.0,
            expected_status: "insufficient_chromosomes".to_string(),
            expected_insufficiency_reason: Some("insufficient_chromosomes".to_string()),
        },
    }
}

fn local_sex_tool_haplogroup_status(proof_case: ProofCaseKind) -> &'static str {
    match proof_case {
        ProofCaseKind::Ready => "not_applicable_for_sex_inference",
        ProofCaseKind::Insufficient => "not_applicable_due_to_insufficient_chromosomes",
    }
}

fn write_local_sex_tool_stage_metrics(
    path: &Path,
    sample_id: &str,
    tool_id: &str,
    proof_case: ProofCaseKind,
    summary: &bijux_dna_domain_bam::BamSexSummaryV1,
    expectation: &LocalSexToolExpectation,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "schema_version": LOCAL_SEX_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.sex",
            "sample_id": sample_id,
            "tool_id": tool_id,
            "proof_case": proof_case.as_str(),
            "method": summary.method,
            "expected_call": expectation.expected_call,
            "call": summary.call,
            "expected_confidence": expectation.expected_confidence,
            "confidence": summary.confidence,
            "expected_status": expectation.expected_status,
            "status": summary.status,
            "expected_insufficiency_reason": expectation.expected_insufficiency_reason,
            "insufficiency_reason": summary.insufficiency_reason,
            "x_coverage": summary.x_coverage,
            "y_coverage": summary.y_coverage,
            "autosomal_coverage": summary.autosomal_coverage,
            "x_to_y_ratio": summary.x_to_y_ratio,
            "expectation_matched": expectation.expectation_matched,
        }),
    )
}

fn sex_tool_report(summary: &bijux_dna_domain_bam::BamSexSummaryV1) -> serde_json::Value {
    serde_json::json!({
        "schema_version": SEX_TOOL_REPORT_SCHEMA_VERSION,
        "method": summary.method,
        "chromosome_system": summary.chromosome_system,
        "minimum_y_sites": summary.minimum_y_sites,
        "x_coverage": summary.x_coverage,
        "y_coverage": summary.y_coverage,
        "autosomal_coverage": summary.autosomal_coverage,
        "x_to_y_ratio": summary.x_to_y_ratio,
        "classification": summary.call,
        "call": summary.call,
        "confidence": summary.confidence,
        "status": summary.status,
        "insufficiency_reason": summary.insufficiency_reason,
    })
}

fn sex_call_name(call: bijux_dna_domain_bam::metrics::SexConfidenceClass) -> &'static str {
    match call {
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Male => "male",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Female => "female",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Ambiguous => "ambiguous",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient => "insufficient",
    }
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.sex local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}
