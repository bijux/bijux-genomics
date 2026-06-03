#[cfg(feature = "bam_downstream")]
use std::path::{Path, PathBuf};

#[cfg(feature = "bam_downstream")]
use anyhow::{anyhow, Context, Result};
#[cfg(feature = "bam_downstream")]
use serde::Serialize;

#[cfg(feature = "bam_downstream")]
const LOCAL_KINSHIP_SMOKE_REPORT_SCHEMA_VERSION: &str = "bijux.bam.kinship.local_smoke.report.v1";
#[cfg(feature = "bam_downstream")]
const LOCAL_KINSHIP_SMOKE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.kinship.local_smoke.metrics.v1";
#[cfg(feature = "bam_downstream")]
const KINSHIP_TOOL_REPORT_SCHEMA_VERSION: &str = "bijux.bam.kinship.v1";
#[cfg(feature = "bam_downstream")]
const KINSHIP_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.kinship.stage_metrics.v1";

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone, Serialize)]
struct LocalKinshipSmokeCaseReport {
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    reference_panel: String,
    reference_build: String,
    population_scope: String,
    min_overlap_snps: u32,
    requires_cohort_context: bool,
    observed_max_overlap_snps: u32,
    pair_count: u32,
    status: String,
    insufficiency_reason: Option<String>,
    pairwise_results: Vec<bijux_dna_domain_bam::BamKinshipPairResultV1>,
    kinship_report: String,
    kinship_summary: String,
    kinship_segments: String,
    stage_metrics: String,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone, Serialize)]
struct LocalKinshipSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: u64,
    all_cases_matched: bool,
    cases: Vec<LocalKinshipSmokeCaseReport>,
}

/// Materialize the governed local-smoke `bam.kinship` artifacts and top-level report.
///
/// The written report lives at `target/local-smoke/bam.kinship/kinship.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
#[cfg(feature = "bam_downstream")]
pub fn write_local_kinship_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_kinship_smoke_plans(&repo_root)?;
    let output_root = repo_root.join("target/local-smoke/bam.kinship");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let case_reports = cases
        .iter()
        .map(|case| materialize_local_kinship_smoke_case(&repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    let report = LocalKinshipSmokeReport {
        schema_version: LOCAL_KINSHIP_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.kinship".to_string(),
        case_count: case_reports.len() as u64,
        all_cases_matched: case_reports.iter().all(|case| case.expectation_matched),
        cases: case_reports,
    };
    let report_path = output_root.join("kinship.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable `bam.kinship` report, summary, and pairwise-segment artifacts beside stage outputs.
///
/// # Errors
/// Returns an error if the input fixture cannot be summarized or the kinship artifacts cannot be
/// written.
#[cfg(feature = "bam_downstream")]
pub(crate) fn write_stage_kinship_artifacts(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<bijux_dna_domain_bam::BamKinshipSummaryV1> {
    let input_bam = resolve_bam_input_path(stage_dir, plan)?;
    let reference_panel = required_string_param(plan, "reference_panel")?;
    let reference_build = required_string_param(plan, "reference_build")?;
    let population_scope = required_string_param(plan, "population_scope")?;
    let min_overlap_snps = required_u32_param(plan, "min_overlap_snps")?;
    let requires_cohort_context = plan
        .params
        .get("requires_cohort_context")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let summary = bijux_dna_domain_bam::summarize_tiny_bam_kinship(
        &input_bam,
        plan.tool_id.as_str(),
        &reference_panel,
        &reference_build,
        &population_scope,
        min_overlap_snps,
        requires_cohort_context,
    )?;
    let report_path = stage_dir.join("kinship.json");
    let summary_path = stage_dir.join("kinship.summary.json");
    let segments_path = stage_dir.join("kinship.segments.tsv");
    let stage_metrics_path = stage_dir.join("stage.metrics.json");

    bijux_dna_infra::atomic_write_json(&report_path, &kinship_tool_report(&summary))
        .with_context(|| format!("write {}", report_path.display()))?;
    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    bijux_dna_infra::atomic_write_bytes(
        &segments_path,
        render_kinship_segments(&summary.pairwise_results).as_bytes(),
    )
    .with_context(|| format!("write {}", segments_path.display()))?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": KINSHIP_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.kinship",
            "method": summary.method,
            "reference_panel": summary.reference_panel,
            "reference_build": summary.reference_build,
            "population_scope": summary.population_scope,
            "min_overlap_snps": summary.min_overlap_snps,
            "requires_cohort_context": summary.requires_cohort_context,
            "sample_count": summary.sample_count,
            "observed_max_overlap_snps": summary.observed_max_overlap_snps,
            "pair_count": summary.pair_count,
            "status": summary.status,
            "insufficiency_reason": summary.insufficiency_reason,
            "segments_path": segments_path,
        }),
    )
    .with_context(|| format!("write {}", stage_metrics_path.display()))?;
    Ok(summary)
}

#[cfg(feature = "bam_downstream")]
fn materialize_local_kinship_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalKinshipSmokeCasePlan,
) -> Result<LocalKinshipSmokeCaseReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let kinship_report_path = resolve_output_path(repo_root, &case.plan, "kinship_report")?;
    let kinship_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let kinship_segments_path = case_out_dir.join("kinship.segments.tsv");
    let summary = write_stage_kinship_artifacts(&case_out_dir, &case.plan)?;
    let expectation_matched = kinship_summary_matches_case(&summary, case);

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_KINSHIP_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.kinship",
            "sample_id": case.sample_id,
            "method": summary.method,
            "reference_panel": summary.reference_panel,
            "reference_build": summary.reference_build,
            "population_scope": summary.population_scope,
            "min_overlap_snps": summary.min_overlap_snps,
            "requires_cohort_context": summary.requires_cohort_context,
            "sample_count": summary.sample_count,
            "observed_max_overlap_snps": summary.observed_max_overlap_snps,
            "pair_count": summary.pair_count,
            "status": summary.status,
            "insufficiency_reason": summary.insufficiency_reason,
            "pairwise_results": summary.pairwise_results,
            "expectation_matched": expectation_matched,
        }),
    )
    .with_context(|| format!("write {}", stage_metrics_path.display()))?;

    Ok(LocalKinshipSmokeCaseReport {
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &repo_root.join(&case.bam)),
        method: summary.method.clone(),
        reference_panel: summary.reference_panel.clone(),
        reference_build: summary.reference_build.clone(),
        population_scope: summary.population_scope.clone(),
        min_overlap_snps: summary.min_overlap_snps,
        requires_cohort_context: summary.requires_cohort_context,
        observed_max_overlap_snps: summary.observed_max_overlap_snps,
        pair_count: summary.pair_count,
        status: summary.status.clone(),
        insufficiency_reason: summary.insufficiency_reason.clone(),
        pairwise_results: summary.pairwise_results.clone(),
        kinship_report: path_relative_to_repo(repo_root, &kinship_report_path),
        kinship_summary: path_relative_to_repo(repo_root, &kinship_summary_path),
        kinship_segments: path_relative_to_repo(repo_root, &kinship_segments_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

#[cfg(feature = "bam_downstream")]
fn kinship_summary_matches_case(
    summary: &bijux_dna_domain_bam::BamKinshipSummaryV1,
    case: &bijux_dna_planner_bam::stage_api::LocalKinshipSmokeCasePlan,
) -> bool {
    summary.method == case.plan.tool_id.as_str()
        && summary.reference_panel == case.reference_panel
        && summary.reference_build == case.reference_build
        && summary.population_scope == case.population_scope
        && summary.min_overlap_snps == case.min_overlap_snps
        && summary.requires_cohort_context == case.requires_cohort_context
        && summary.status == case.expected_status
        && summary.observed_max_overlap_snps == case.expected_observed_max_overlap_snps
        && summary.insufficiency_reason == case.expected_insufficiency_reason
        && summary.pairwise_results.len() == case.expected_pairwise_results.len()
        && summary.pairwise_results.iter().zip(case.expected_pairwise_results.iter()).all(
            |(observed, expected)| {
                observed.sample_a == expected.sample_a
                    && observed.sample_b == expected.sample_b
                    && observed.overlap_snps == expected.overlap_snps
                    && observed.matching_sites == expected.matching_sites
                    && observed.mismatch_sites == expected.mismatch_sites
                    && float_matches(observed.concordance, expected.concordance)
                    && float_matches(observed.kinship_coefficient, expected.kinship_coefficient)
                    && observed.relationship_label == expected.relationship_label
            },
        )
}

#[cfg(feature = "bam_downstream")]
fn kinship_tool_report(summary: &bijux_dna_domain_bam::BamKinshipSummaryV1) -> serde_json::Value {
    serde_json::json!({
        "schema_version": KINSHIP_TOOL_REPORT_SCHEMA_VERSION,
        "method": summary.method,
        "reference_panel": summary.reference_panel,
        "reference_build": summary.reference_build,
        "population_scope": summary.population_scope,
        "min_overlap_snps": summary.min_overlap_snps,
        "requires_cohort_context": summary.requires_cohort_context,
        "sample_count": summary.sample_count,
        "observed_max_overlap_snps": summary.observed_max_overlap_snps,
        "pair_count": summary.pair_count,
        "status": summary.status,
        "insufficiency_reason": summary.insufficiency_reason,
        "pairwise_results": summary.pairwise_results,
    })
}

#[cfg(feature = "bam_downstream")]
fn render_kinship_segments(
    pairwise_results: &[bijux_dna_domain_bam::BamKinshipPairResultV1],
) -> String {
    let mut rendered = String::from(
        "sample_a\tsample_b\toverlap_snps\tmatching_sites\tmismatch_sites\tconcordance\tkinship_coefficient\trelationship_label\n",
    );
    for pair in pairwise_results {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.6}\t{:.6}\t{}\n",
            pair.sample_a,
            pair.sample_b,
            pair.overlap_snps,
            pair.matching_sites,
            pair.mismatch_sites,
            pair.concordance,
            pair.kinship_coefficient,
            pair.relationship_label
        ));
    }
    rendered
}

#[cfg(feature = "bam_downstream")]
fn resolve_bam_input_path(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map(|artifact| artifact.path.clone())
        .or_else(|| plan.params.get("bam").and_then(serde_json::Value::as_str).map(PathBuf::from))
        .unwrap_or_else(|| stage_dir.join("in.bam"));
    Ok(resolve_stage_input_path(&input_bam))
}

#[cfg(feature = "bam_downstream")]
fn required_string_param(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    key: &str,
) -> Result<String> {
    plan.params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("bam.kinship plan is missing required string param `{key}`"))
}

#[cfg(feature = "bam_downstream")]
fn required_u32_param(plan: &bijux_dna_stage_contract::StagePlanV1, key: &str) -> Result<u32> {
    plan.params
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| anyhow!("bam.kinship plan is missing required u32 param `{key}`"))
}

#[cfg(feature = "bam_downstream")]
fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-6
}

#[cfg(feature = "bam_downstream")]
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
            anyhow!("bam.kinship local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

#[cfg(feature = "bam_downstream")]
fn resolve_stage_input_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        crate::support::workspace::resolve_repo_root()
            .map(|repo_root| repo_root.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

#[cfg(feature = "bam_downstream")]
fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

#[cfg(feature = "bam_downstream")]
fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

#[cfg(feature = "bam_downstream")]
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}
