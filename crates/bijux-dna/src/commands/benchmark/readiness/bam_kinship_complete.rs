use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[cfg(feature = "bam_downstream")]
use super::bam_kinship_ready::{
    render_bam_kinship_ready, BamKinshipReadyRow, DEFAULT_BAM_KINSHIP_READY_PATH,
};
#[cfg(feature = "bam_downstream")]
use super::real_output_parser_smoke::{
    render_real_output_parser_smoke, RealOutputParserSmokeReport, RealOutputParserSmokeRow,
    DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH,
};
#[cfg(feature = "bam_downstream")]
use crate::commands::benchmark::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
    LocalPipelineDagValidationNodeReport, LocalPipelineDagValidationReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;
#[cfg(feature = "bam_downstream")]
use std::fs;

pub(crate) const DEFAULT_BAM_KINSHIP_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.kinship.complete.json";
const BAM_KINSHIP_COMPLETE_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_kinship_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.kinship";
const EXPECTED_READY_CASE_SAMPLE_ID: &str = "human_like_kinship_related_pair";
const EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID: &str = "human_like_kinship_low_overlap_pair";
const EXPECTED_METHOD: &str = "king";
const EXPECTED_REFERENCE_PANEL: &str = "human_like_relatedness_panel";
const EXPECTED_REFERENCE_BUILD: &str = "grch38";
const EXPECTED_POPULATION_SCOPE: &str = "human_diploid_panel";
const EXPECTED_READY_STATUS: &str = "ok";
const EXPECTED_INSUFFICIENT_STATUS: &str = "insufficient";
const EXPECTED_INSUFFICIENCY_REASON: &str = "insufficient_overlap_snps";
const EXPECTED_PARSER_SCHEMA_VERSION: &str = "bijux.bam.kinship_summary.v1";
const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.kinship.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.kinship_summary.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.kinship.local_smoke.metrics.v1";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.kinship.local_smoke.report.v1";
const EXPECTED_BAM_PIPELINE_ID: &str = "bam-kinship";
const EXPECTED_VCF_PIPELINE_ID: &str = "bam-genotyping-to-vcf-downstream";
const EXPECTED_READY_CASE_PAIR_COUNT: u64 = 1;
const EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT: u64 = 0;
const EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS: u64 = 6;
const EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS: u64 = 4;
const EXPECTED_READY_CASE_SAMPLE_A: &str = "sample_a";
const EXPECTED_READY_CASE_SAMPLE_B: &str = "sample_b";
const EXPECTED_READY_CASE_OVERLAP_SNPS: u64 = 6;
const EXPECTED_READY_CASE_KINSHIP_COEFFICIENT: f64 = 0.416667;
const EXPECTED_READY_CASE_RELATIONSHIP_LABEL: &str = "first_degree";
const EXPECTED_PAIRWISE_TABLE_ROW: &str =
    "sample_a\tsample_b\t6\t5\t1\t0.833333\t0.416667\tfirst_degree";
const CHECKED_SURFACE_COUNT: usize = 19;
const EXPECTED_TOOL_IDS: [&str; 2] = ["angsd", "king"];
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["kinship_report", "summary", "stage_metrics"];
const REQUIRED_BAM_PIPELINE_DEPENDENCIES: [&str; 2] = ["bam.genotyping", "bam.overlap_correction"];
const REQUIRED_BAM_PIPELINE_EXTERNAL_INPUTS: [&str; 3] = [
    "kinship_pairing_contract",
    "kinship_population_scope_contract",
    "kinship_reference_panel_contract",
];
const REQUIRED_BAM_PIPELINE_UPSTREAM_INPUTS: [&str; 4] = [
    "genotyping_report_json",
    "overlap_corrected_bai",
    "overlap_corrected_bam",
    "overlap_correction_summary_json",
];
const REQUIRED_BAM_PIPELINE_OUTPUTS: [&str; 4] = [
    "kinship_report_json",
    "kinship_segments_tsv",
    "kinship_stage_metrics",
    "kinship_summary_json",
];

#[derive(Debug, Clone, Deserialize)]
struct LocalKinshipSmokeReport {
    schema_version: String,
    stage_id: String,
    case_count: usize,
    all_cases_matched: bool,
    cases: Vec<LocalKinshipSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalKinshipSmokeCaseReport {
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    reference_panel: String,
    reference_build: String,
    population_scope: String,
    min_overlap_snps: u64,
    requires_cohort_context: bool,
    observed_max_overlap_snps: u64,
    pair_count: u64,
    status: String,
    insufficiency_reason: Option<String>,
    pairwise_results: Vec<LocalKinshipPairwiseResult>,
    kinship_report: String,
    kinship_summary: String,
    kinship_segments: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalKinshipPairwiseResult {
    sample_a: String,
    sample_b: String,
    overlap_snps: u64,
    kinship_coefficient: f64,
    relationship_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamKinshipCompleteRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_scope: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) report_section_id: String,
    pub(crate) summary_table_id: String,
    pub(crate) command_readiness_kind: String,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) expected_schema_extension_id: String,
    pub(crate) schema_extension_id: String,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) ready_case_report_path: String,
    pub(crate) ready_case_summary_path: String,
    pub(crate) ready_case_segments_path: String,
    pub(crate) ready_case_stage_metrics_path: String,
    pub(crate) insufficient_case_report_path: String,
    pub(crate) insufficient_case_summary_path: String,
    pub(crate) insufficient_case_segments_path: String,
    pub(crate) insufficient_case_stage_metrics_path: String,
    pub(crate) parser_smoke_proof_path: String,
    pub(crate) bam_pipeline_proof_path: String,
    pub(crate) vcf_pipeline_proof_path: String,
    pub(crate) ready_case_sample_id: String,
    pub(crate) ready_case_status: String,
    pub(crate) ready_case_reference_panel: String,
    pub(crate) ready_case_reference_build: String,
    pub(crate) ready_case_population_scope: String,
    pub(crate) ready_case_observed_max_overlap_snps: u64,
    pub(crate) ready_case_pair_count: u64,
    pub(crate) ready_case_pair_sample_a: String,
    pub(crate) ready_case_pair_sample_b: String,
    pub(crate) ready_case_pair_overlap_snps: u64,
    pub(crate) ready_case_kinship_coefficient: f64,
    pub(crate) ready_case_relationship_label: String,
    pub(crate) insufficient_case_sample_id: String,
    pub(crate) insufficient_case_status: String,
    pub(crate) insufficient_case_reference_panel: String,
    pub(crate) insufficient_case_reference_build: String,
    pub(crate) insufficient_case_population_scope: String,
    pub(crate) insufficient_case_observed_max_overlap_snps: u64,
    pub(crate) insufficient_case_pair_count: u64,
    pub(crate) insufficient_case_insufficiency_reason: String,
    pub(crate) parser_smoke_schema_version: String,
    pub(crate) parser_smoke_method: String,
    pub(crate) parser_smoke_status: String,
    pub(crate) parser_smoke_pair_count: u64,
    pub(crate) parser_smoke_observed_max_overlap_snps: u64,
    pub(crate) bam_pipeline_id: String,
    pub(crate) bam_pipeline_upstream_inputs: Vec<String>,
    pub(crate) bam_pipeline_external_inputs: Vec<String>,
    pub(crate) bam_pipeline_outputs: Vec<String>,
    pub(crate) vcf_pipeline_id: String,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) ready_case_report_ready: bool,
    pub(crate) ready_case_summary_ready: bool,
    pub(crate) ready_case_pairwise_table_ready: bool,
    pub(crate) ready_case_stage_metrics_ready: bool,
    pub(crate) insufficient_case_report_ready: bool,
    pub(crate) insufficient_case_summary_ready: bool,
    pub(crate) insufficient_case_stage_metrics_ready: bool,
    pub(crate) parser_smoke_ready: bool,
    pub(crate) bam_pipeline_ready: bool,
    pub(crate) bam_locality_ready: bool,
    pub(crate) vcf_isolation_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamKinshipCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) local_smoke_case_count: usize,
    pub(crate) bam_pipeline_id: String,
    pub(crate) vcf_pipeline_id: String,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamKinshipCompleteRow>,
    pub(crate) violations: Vec<BamKinshipCompleteRow>,
}

pub(crate) fn run_render_bam_kinship_complete(
    args: &parse::BenchReadinessRenderBamKinshipCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_kinship_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_KINSHIP_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

#[cfg(feature = "bam_downstream")]
pub(crate) fn render_bam_kinship_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamKinshipCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_kinship_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam kinship completion must keep active scope, command, output, parser, expected-result, report, schema, pairwise evidence, insufficiency evidence, parser smoke, and locality proof for all retained tools"
        ));
    }
    Ok(report)
}

#[cfg(not(feature = "bam_downstream"))]
pub(crate) fn render_bam_kinship_complete(
    _repo_root: &Path,
    _output_path: PathBuf,
) -> Result<BamKinshipCompleteReport> {
    Err(anyhow!("bam.kinship completion requires the `bam_downstream` feature"))
}

#[cfg(feature = "bam_downstream")]
fn build_bam_kinship_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamKinshipCompleteReport> {
    let readiness =
        render_bam_kinship_ready(repo_root, PathBuf::from(DEFAULT_BAM_KINSHIP_READY_PATH))?;
    let smoke_path = bijux_dna_api::v1::api::bam::write_local_kinship_smoke_report()?;
    let smoke_report: LocalKinshipSmokeReport = serde_json::from_str(
        &fs::read_to_string(&smoke_path)
            .with_context(|| format!("read {}", smoke_path.display()))?,
    )
    .with_context(|| format!("parse {}", smoke_path.display()))?;
    let parser_smoke = render_real_output_parser_smoke(
        repo_root,
        PathBuf::from(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH),
    )?;
    let bam_pipeline_report_path =
        repo_root.join("benchmarks/readiness/local-ready/pipeline-dag/bam-kinship.json");
    let bam_pipeline = validate_pipeline_dag_path(
        repo_root,
        &benchmark_local_pipeline_config_path(repo_root, EXPECTED_BAM_PIPELINE_ID),
        &bam_pipeline_report_path,
    )?;
    let vcf_pipeline_report_path = repo_root.join(
        "benchmarks/readiness/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json",
    );
    let vcf_pipeline = validate_pipeline_dag_path(
        repo_root,
        &benchmark_local_pipeline_config_path(repo_root, EXPECTED_VCF_PIPELINE_ID),
        &vcf_pipeline_report_path,
    )?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.kinship local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.kinship local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.case_count != 2 || smoke_report.cases.len() != 2 {
        return Err(anyhow!("bam.kinship local-smoke report must keep exactly two governed cases"));
    }

    let ready_case = find_case(&smoke_report, EXPECTED_READY_CASE_SAMPLE_ID)?;
    let insufficient_case = find_case(&smoke_report, EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID)?;
    let parser_row = find_parser_smoke_row(&parser_smoke)?;
    let bam_pipeline_node = find_pipeline_node(&bam_pipeline, EXPECTED_STAGE_ID)?;

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.kinship readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_kinship_complete_row(
                repo_root,
                &smoke_path,
                &smoke_report,
                ready_case,
                insufficient_case,
                readiness_row,
                parser_row,
                &bam_pipeline,
                bam_pipeline_node,
                &vcf_pipeline,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    Ok(BamKinshipCompleteReport {
        schema_version: BAM_KINSHIP_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_case_count: smoke_report.case_count,
        bam_pipeline_id: EXPECTED_BAM_PIPELINE_ID.to_string(),
        vcf_pipeline_id: EXPECTED_VCF_PIPELINE_ID.to_string(),
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    })
}

#[cfg(feature = "bam_downstream")]
#[allow(clippy::too_many_arguments)]
fn build_bam_kinship_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalKinshipSmokeReport,
    ready_case: &LocalKinshipSmokeCaseReport,
    insufficient_case: &LocalKinshipSmokeCaseReport,
    readiness_row: &BamKinshipReadyRow,
    parser_row: &RealOutputParserSmokeRow,
    bam_pipeline: &LocalPipelineDagValidationReport,
    bam_pipeline_node: &LocalPipelineDagValidationNodeReport,
    vcf_pipeline: &LocalPipelineDagValidationReport,
) -> Result<BamKinshipCompleteRow> {
    ensure_repo_relative_file(repo_root, &ready_case.kinship_report)?;
    ensure_repo_relative_file(repo_root, &ready_case.kinship_summary)?;
    ensure_repo_relative_file(repo_root, &ready_case.kinship_segments)?;
    ensure_repo_relative_file(repo_root, &ready_case.stage_metrics)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_report)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_summary)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.kinship_segments)?;
    ensure_repo_relative_file(repo_root, &insufficient_case.stage_metrics)?;

    let ready_report = read_json(repo_root.join(&ready_case.kinship_report))?;
    let ready_summary = read_json(repo_root.join(&ready_case.kinship_summary))?;
    let ready_stage_metrics = read_json(repo_root.join(&ready_case.stage_metrics))?;
    let insufficient_report = read_json(repo_root.join(&insufficient_case.kinship_report))?;
    let insufficient_summary = read_json(repo_root.join(&insufficient_case.kinship_summary))?;
    let insufficient_stage_metrics = read_json(repo_root.join(&insufficient_case.stage_metrics))?;
    let ready_segments = fs::read_to_string(repo_root.join(&ready_case.kinship_segments))
        .with_context(|| {
            format!("read {}", repo_root.join(&ready_case.kinship_segments).display())
        })?;

    let ready_pair = ready_case
        .pairwise_results
        .first()
        .ok_or_else(|| anyhow!("ready bam.kinship smoke case must keep one pairwise result"))?;

    let local_smoke_ready = smoke_report.all_cases_matched
        && ready_case.expectation_matched
        && ready_case.sample_id == EXPECTED_READY_CASE_SAMPLE_ID
        && ready_case.method == EXPECTED_METHOD
        && ready_case.reference_panel == EXPECTED_REFERENCE_PANEL
        && ready_case.reference_build == EXPECTED_REFERENCE_BUILD
        && ready_case.population_scope == EXPECTED_POPULATION_SCOPE
        && ready_case.observed_max_overlap_snps == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && ready_case.pair_count == EXPECTED_READY_CASE_PAIR_COUNT
        && ready_case.status == EXPECTED_READY_STATUS
        && ready_case.insufficiency_reason.is_none()
        && ready_case.requires_cohort_context
        && ready_case.min_overlap_snps == EXPECTED_READY_CASE_OVERLAP_SNPS
        && insufficient_case.expectation_matched
        && insufficient_case.sample_id == EXPECTED_INSUFFICIENT_CASE_SAMPLE_ID
        && insufficient_case.method == EXPECTED_METHOD
        && insufficient_case.reference_panel == EXPECTED_REFERENCE_PANEL
        && insufficient_case.reference_build == EXPECTED_REFERENCE_BUILD
        && insufficient_case.population_scope == EXPECTED_POPULATION_SCOPE
        && insufficient_case.observed_max_overlap_snps
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && insufficient_case.pair_count == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && insufficient_case.status == EXPECTED_INSUFFICIENT_STATUS
        && insufficient_case.insufficiency_reason.as_deref() == Some(EXPECTED_INSUFFICIENCY_REASON)
        && insufficient_case.requires_cohort_context
        && insufficient_case.min_overlap_snps == 5
        && insufficient_case.pairwise_results.is_empty();

    let ready_case_report_ready = required_string(&ready_report, "schema_version")?
        == EXPECTED_REPORT_SCHEMA_VERSION
        && required_string(&ready_report, "status")? == EXPECTED_READY_STATUS
        && required_u64(&ready_report, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_u64(&ready_report, "observed_max_overlap_snps")?
            == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && required_pair_field(&ready_report, 0, "sample_a")? == EXPECTED_READY_CASE_SAMPLE_A
        && required_pair_field(&ready_report, 0, "sample_b")? == EXPECTED_READY_CASE_SAMPLE_B
        && required_pair_u64(&ready_report, 0, "overlap_snps")? == EXPECTED_READY_CASE_OVERLAP_SNPS
        && approx_eq(
            required_pair_f64(&ready_report, 0, "kinship_coefficient")?,
            EXPECTED_READY_CASE_KINSHIP_COEFFICIENT,
        )
        && required_pair_field(&ready_report, 0, "relationship_label")?
            == EXPECTED_READY_CASE_RELATIONSHIP_LABEL;
    let ready_case_summary_ready = required_string(&ready_summary, "schema_version")?
        == EXPECTED_SUMMARY_SCHEMA_VERSION
        && required_string(&ready_summary, "stage_id")? == EXPECTED_STAGE_ID
        && required_string(&ready_summary, "method")? == EXPECTED_METHOD
        && required_string(&ready_summary, "reference_panel")? == EXPECTED_REFERENCE_PANEL
        && required_string(&ready_summary, "reference_build")? == EXPECTED_REFERENCE_BUILD
        && required_string(&ready_summary, "population_scope")? == EXPECTED_POPULATION_SCOPE
        && required_string(&ready_summary, "status")? == EXPECTED_READY_STATUS
        && required_u64(&ready_summary, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_u64(&ready_summary, "observed_max_overlap_snps")?
            == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS;
    let ready_case_pairwise_table_ready = ready_segments.contains(EXPECTED_PAIRWISE_TABLE_ROW);
    let ready_case_stage_metrics_ready = required_string(&ready_stage_metrics, "schema_version")?
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && required_string(&ready_stage_metrics, "status")? == EXPECTED_READY_STATUS
        && required_u64(&ready_stage_metrics, "pair_count")? == EXPECTED_READY_CASE_PAIR_COUNT
        && required_bool(&ready_stage_metrics, "expectation_matched")?;

    let insufficient_case_report_ready = required_string(&insufficient_report, "schema_version")?
        == EXPECTED_REPORT_SCHEMA_VERSION
        && required_string(&insufficient_report, "status")? == EXPECTED_INSUFFICIENT_STATUS
        && required_u64(&insufficient_report, "pair_count")?
            == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && required_u64(&insufficient_report, "observed_max_overlap_snps")?
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && optional_string(&insufficient_report, "insufficiency_reason")?.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON)
        && required_pair_array_len(&insufficient_report)? == 0;
    let insufficient_case_summary_ready = required_string(&insufficient_summary, "schema_version")?
        == EXPECTED_SUMMARY_SCHEMA_VERSION
        && required_string(&insufficient_summary, "stage_id")? == EXPECTED_STAGE_ID
        && required_string(&insufficient_summary, "method")? == EXPECTED_METHOD
        && required_string(&insufficient_summary, "reference_panel")? == EXPECTED_REFERENCE_PANEL
        && required_string(&insufficient_summary, "reference_build")? == EXPECTED_REFERENCE_BUILD
        && required_string(&insufficient_summary, "population_scope")? == EXPECTED_POPULATION_SCOPE
        && required_string(&insufficient_summary, "status")? == EXPECTED_INSUFFICIENT_STATUS
        && required_u64(&insufficient_summary, "pair_count")?
            == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
        && required_u64(&insufficient_summary, "observed_max_overlap_snps")?
            == EXPECTED_INSUFFICIENT_CASE_OBSERVED_MAX_OVERLAP_SNPS
        && optional_string(&insufficient_summary, "insufficiency_reason")?.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON);
    let insufficient_case_stage_metrics_ready =
        required_string(&insufficient_stage_metrics, "schema_version")?
            == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
            && required_string(&insufficient_stage_metrics, "status")?
                == EXPECTED_INSUFFICIENT_STATUS
            && required_u64(&insufficient_stage_metrics, "pair_count")?
                == EXPECTED_INSUFFICIENT_CASE_PAIR_COUNT
            && required_bool(&insufficient_stage_metrics, "expectation_matched")?
            && optional_string(&insufficient_stage_metrics, "insufficiency_reason")?.as_deref()
                == Some(EXPECTED_INSUFFICIENCY_REASON);

    let parser_smoke_method = required_snapshot_string(parser_row, "method")?;
    let parser_smoke_status = required_snapshot_string(parser_row, "status")?;
    let parser_smoke_pair_count = required_snapshot_u64(parser_row, "pair_count")?;
    let parser_smoke_observed_max_overlap_snps =
        required_snapshot_u64(parser_row, "observed_max_overlap_snps")?;
    let parser_smoke_ready = parser_row.passed
        && parser_row.parsed_schema_version == EXPECTED_PARSER_SCHEMA_VERSION
        && parser_smoke_method == EXPECTED_METHOD
        && parser_smoke_status == EXPECTED_READY_STATUS
        && parser_smoke_pair_count == EXPECTED_READY_CASE_PAIR_COUNT
        && parser_smoke_observed_max_overlap_snps == EXPECTED_READY_CASE_OBSERVED_MAX_OVERLAP_SNPS;

    let bam_pipeline_ready = bam_pipeline.valid
        && bam_pipeline.pipeline_id == EXPECTED_BAM_PIPELINE_ID
        && sorted_strings(&bam_pipeline_node.depends_on)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_DEPENDENCIES)
        && sorted_strings(&bam_pipeline_node.external_inputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_EXTERNAL_INPUTS)
        && sorted_strings(&bam_pipeline_node.upstream_inputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_UPSTREAM_INPUTS)
        && sorted_strings(&bam_pipeline_node.outputs)
            == sorted_strings_from_slice(&REQUIRED_BAM_PIPELINE_OUTPUTS);
    let bam_locality_ready = bam_pipeline.nodes.iter().all(|node| {
        node.stage_id == EXPECTED_STAGE_ID
            || !node.upstream_inputs.iter().any(|input| input == "overlap_correction_summary_json")
    });
    let vcf_isolation_ready = vcf_pipeline.valid
        && vcf_pipeline.pipeline_id == EXPECTED_VCF_PIPELINE_ID
        && !vcf_pipeline.nodes.iter().any(|node| {
            node.stage_id == EXPECTED_STAGE_ID
                || node.upstream_inputs.iter().any(|input| {
                    input == "overlap_correction_summary_json" || input == "kinship_report_json"
                })
        });

    let mut missing_surfaces = Vec::new();
    if !readiness_row.active_scope_ready {
        missing_surfaces.push("active_scope".to_string());
    }
    if !readiness_row.command_ready {
        missing_surfaces.push("command".to_string());
    }
    if !readiness_row.output_ready {
        missing_surfaces.push("output".to_string());
    }
    if !readiness_row.parser_ready {
        missing_surfaces.push("parser".to_string());
    }
    if !readiness_row.expected_result_ready {
        missing_surfaces.push("expected_result".to_string());
    }
    if !readiness_row.report_ready {
        missing_surfaces.push("report".to_string());
    }
    if !readiness_row.schema_ready {
        missing_surfaces.push("schema".to_string());
    }
    if !local_smoke_ready {
        missing_surfaces.push("local_smoke".to_string());
    }
    if !ready_case_report_ready {
        missing_surfaces.push("ready_case_report".to_string());
    }
    if !ready_case_summary_ready {
        missing_surfaces.push("ready_case_summary".to_string());
    }
    if !ready_case_pairwise_table_ready {
        missing_surfaces.push("ready_case_pairwise_table".to_string());
    }
    if !ready_case_stage_metrics_ready {
        missing_surfaces.push("ready_case_stage_metrics".to_string());
    }
    if !insufficient_case_report_ready {
        missing_surfaces.push("insufficient_case_report".to_string());
    }
    if !insufficient_case_summary_ready {
        missing_surfaces.push("insufficient_case_summary".to_string());
    }
    if !insufficient_case_stage_metrics_ready {
        missing_surfaces.push("insufficient_case_stage_metrics".to_string());
    }
    if !parser_smoke_ready {
        missing_surfaces.push("parser_smoke".to_string());
    }
    if !bam_pipeline_ready {
        missing_surfaces.push("bam_pipeline".to_string());
    }
    if !bam_locality_ready {
        missing_surfaces.push("bam_locality".to_string());
    }
    if !vcf_isolation_ready {
        missing_surfaces.push("vcf_isolation".to_string());
    }
    let coverage_status = if missing_surfaces.is_empty() { "complete" } else { "incomplete" };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `bam.kinship` / `{}` keeps pairwise kinship evidence, explicit insufficient-overlap behavior, parser-smoke proof, and locality across BAM and VCF handoffs complete",
            readiness_row.tool_id
        )
    } else {
        format!(
            "binding `bam.kinship` / `{}` is missing {} required completion surface(s): {}",
            readiness_row.tool_id,
            missing_surfaces.len(),
            missing_surfaces.join(", ")
        )
    };

    Ok(BamKinshipCompleteRow {
        result_id: readiness_row.result_id.clone(),
        stage_id: readiness_row.stage_id.clone(),
        tool_id: readiness_row.tool_id.clone(),
        sample_scope: readiness_row.sample_scope.clone(),
        benchmark_status: readiness_row.benchmark_status.clone(),
        support_status: readiness_row.support_status.clone(),
        adapter_status: readiness_row.adapter_status.clone(),
        parser_status: readiness_row.parser_status.clone(),
        corpus_status: readiness_row.corpus_status.clone(),
        report_section_id: readiness_row.report_section_id.clone(),
        summary_table_id: readiness_row.summary_table_id.clone(),
        command_readiness_kind: readiness_row.command_readiness_kind.clone(),
        required_output_ids: readiness_row.required_output_ids.clone(),
        stage_output_ids: readiness_row.stage_output_ids.clone(),
        expected_schema_extension_id: readiness_row.expected_schema_extension_id.clone(),
        schema_extension_id: readiness_row.schema_extension_id.clone(),
        active_scope_proof_path: readiness_row.active_scope_proof_path.clone(),
        command_proof_path: readiness_row.command_proof_path.clone(),
        output_contract_proof_path: readiness_row.output_contract_proof_path.clone(),
        parser_proof_path: readiness_row.parser_proof_path.clone(),
        expected_result_proof_path: readiness_row.expected_result_proof_path.clone(),
        report_map_proof_path: readiness_row.report_map_proof_path.clone(),
        schema_proof_path: readiness_row.schema_proof_path.clone(),
        local_smoke_proof_path: path_relative_to_repo(repo_root, smoke_path),
        ready_case_report_path: ready_case.kinship_report.clone(),
        ready_case_summary_path: ready_case.kinship_summary.clone(),
        ready_case_segments_path: ready_case.kinship_segments.clone(),
        ready_case_stage_metrics_path: ready_case.stage_metrics.clone(),
        insufficient_case_report_path: insufficient_case.kinship_report.clone(),
        insufficient_case_summary_path: insufficient_case.kinship_summary.clone(),
        insufficient_case_segments_path: insufficient_case.kinship_segments.clone(),
        insufficient_case_stage_metrics_path: insufficient_case.stage_metrics.clone(),
        parser_smoke_proof_path: parser_row.proof_path.clone(),
        bam_pipeline_proof_path: bam_pipeline.output_path.clone(),
        vcf_pipeline_proof_path: vcf_pipeline.output_path.clone(),
        ready_case_sample_id: ready_case.sample_id.clone(),
        ready_case_status: ready_case.status.clone(),
        ready_case_reference_panel: ready_case.reference_panel.clone(),
        ready_case_reference_build: ready_case.reference_build.clone(),
        ready_case_population_scope: ready_case.population_scope.clone(),
        ready_case_observed_max_overlap_snps: ready_case.observed_max_overlap_snps,
        ready_case_pair_count: ready_case.pair_count,
        ready_case_pair_sample_a: ready_pair.sample_a.clone(),
        ready_case_pair_sample_b: ready_pair.sample_b.clone(),
        ready_case_pair_overlap_snps: ready_pair.overlap_snps,
        ready_case_kinship_coefficient: ready_pair.kinship_coefficient,
        ready_case_relationship_label: ready_pair.relationship_label.clone(),
        insufficient_case_sample_id: insufficient_case.sample_id.clone(),
        insufficient_case_status: insufficient_case.status.clone(),
        insufficient_case_reference_panel: insufficient_case.reference_panel.clone(),
        insufficient_case_reference_build: insufficient_case.reference_build.clone(),
        insufficient_case_population_scope: insufficient_case.population_scope.clone(),
        insufficient_case_observed_max_overlap_snps: insufficient_case.observed_max_overlap_snps,
        insufficient_case_pair_count: insufficient_case.pair_count,
        insufficient_case_insufficiency_reason: insufficient_case
            .insufficiency_reason
            .clone()
            .unwrap_or_default(),
        parser_smoke_schema_version: parser_row.parsed_schema_version.clone(),
        parser_smoke_method,
        parser_smoke_status,
        parser_smoke_pair_count,
        parser_smoke_observed_max_overlap_snps,
        bam_pipeline_id: bam_pipeline.pipeline_id.clone(),
        bam_pipeline_upstream_inputs: bam_pipeline_node.upstream_inputs.clone(),
        bam_pipeline_external_inputs: bam_pipeline_node.external_inputs.clone(),
        bam_pipeline_outputs: bam_pipeline_node.outputs.clone(),
        vcf_pipeline_id: vcf_pipeline.pipeline_id.clone(),
        active_scope_ready: readiness_row.active_scope_ready,
        command_ready: readiness_row.command_ready,
        output_ready: readiness_row.output_ready,
        parser_ready: readiness_row.parser_ready,
        expected_result_ready: readiness_row.expected_result_ready,
        report_ready: readiness_row.report_ready,
        schema_ready: readiness_row.schema_ready,
        local_smoke_ready,
        ready_case_report_ready,
        ready_case_summary_ready,
        ready_case_pairwise_table_ready,
        ready_case_stage_metrics_ready,
        insufficient_case_report_ready,
        insufficient_case_summary_ready,
        insufficient_case_stage_metrics_ready,
        parser_smoke_ready,
        bam_pipeline_ready,
        bam_locality_ready,
        vcf_isolation_ready,
        coverage_status: coverage_status.to_string(),
        missing_surfaces,
        reason,
    })
}

#[cfg(feature = "bam_downstream")]
fn find_case<'a>(
    report: &'a LocalKinshipSmokeReport,
    sample_id: &str,
) -> Result<&'a LocalKinshipSmokeCaseReport> {
    report
        .cases
        .iter()
        .find(|case| case.sample_id == sample_id)
        .ok_or_else(|| anyhow!("missing bam.kinship local-smoke case `{sample_id}`"))
}

#[cfg(feature = "bam_downstream")]
fn find_parser_smoke_row(
    report: &RealOutputParserSmokeReport,
) -> Result<&RealOutputParserSmokeRow> {
    report
        .rows
        .iter()
        .find(|row| {
            row.representative_stage_id == EXPECTED_STAGE_ID
                && row.representative_tool_id == EXPECTED_METHOD
        })
        .ok_or_else(|| anyhow!("missing real-output parser-smoke row for `bam.kinship` / `king`"))
}

#[cfg(feature = "bam_downstream")]
fn find_pipeline_node<'a>(
    report: &'a LocalPipelineDagValidationReport,
    stage_id: &str,
) -> Result<&'a LocalPipelineDagValidationNodeReport> {
    report
        .nodes
        .iter()
        .find(|node| node.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing `{stage_id}` node in pipeline `{}`", report.pipeline_id))
}

#[cfg(feature = "bam_downstream")]
fn read_json(path: PathBuf) -> Result<serde_json::Value> {
    let payload = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&payload).with_context(|| format!("parse {}", path.display()))
}

fn required_string(payload: &serde_json::Value, key: &str) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing string field `{key}`"))
}

fn required_u64(payload: &serde_json::Value, key: &str) -> Result<u64> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing integer field `{key}`"))
}

fn required_bool(payload: &serde_json::Value, key: &str) -> Result<bool> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| anyhow!("missing boolean field `{key}`"))
}

fn optional_string(payload: &serde_json::Value, key: &str) -> Result<Option<String>> {
    let Some(value) = payload.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    value
        .as_str()
        .map(|item| Some(item.to_string()))
        .ok_or_else(|| anyhow!("field `{key}` must be string or null"))
}

fn required_pair_array_len(payload: &serde_json::Value) -> Result<usize> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .map(std::vec::Vec::len)
        .ok_or_else(|| anyhow!("missing array field `pairwise_results`"))
}

fn required_pair_field(payload: &serde_json::Value, index: usize, key: &str) -> Result<String> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing string pairwise field `{key}` at index {index}"))
}

fn required_pair_u64(payload: &serde_json::Value, index: usize, key: &str) -> Result<u64> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing integer pairwise field `{key}` at index {index}"))
}

fn required_pair_f64(payload: &serde_json::Value, index: usize, key: &str) -> Result<f64> {
    payload
        .get("pairwise_results")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| items.get(index))
        .and_then(|item| item.get(key))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing float pairwise field `{key}` at index {index}"))
}

#[cfg(feature = "bam_downstream")]
fn required_snapshot_string(row: &RealOutputParserSmokeRow, key: &str) -> Result<String> {
    row.normalized_snapshot
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("missing parser-smoke snapshot string `{key}`"))
}

#[cfg(feature = "bam_downstream")]
fn required_snapshot_u64(row: &RealOutputParserSmokeRow, key: &str) -> Result<u64> {
    row.normalized_snapshot
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing parser-smoke snapshot integer `{key}`"))
}

fn ensure_repo_relative_file(repo_root: &Path, relative: &str) -> Result<()> {
    let path = repo_root.join(relative);
    if !path.is_file() {
        return Err(anyhow!("governed kinship artifact is missing: {}", path.display()));
    }
    Ok(())
}

fn sorted_strings(values: &[String]) -> Vec<String> {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted
}

fn sorted_strings_from_slice(values: &[&str]) -> Vec<String> {
    let mut sorted = values.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    sorted.sort();
    sorted
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    repo_relative_path(repo_root, path).display().to_string()
}
