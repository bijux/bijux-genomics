use std::path::{Path, PathBuf};

#[cfg(feature = "bam_downstream")]
use anyhow::Context;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[cfg(feature = "bam_downstream")]
use super::bam_recalibration_genotyping_ready::{
    render_bam_recalibration_genotyping_ready, BamRecalibrationGenotypingReadyRow,
    DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH,
};
#[cfg(feature = "bam_downstream")]
use super::real_output_parser_probe::{
    render_real_output_parser_smoke, RealOutputParserSmokeReport,
    DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH,
};
#[cfg(feature = "bam_downstream")]
use super::vcf_qc_ready::{
    render_vcf_qc_ready, VcfQcReadyReport, VcfQcReadyRow, DEFAULT_VCF_QC_READY_PATH,
};
#[cfg(feature = "bam_downstream")]
use super::vcf_stats_ready::{
    render_vcf_stats_ready, VcfStatsReadyReport, VcfStatsReadyRow, DEFAULT_VCF_STATS_READY_PATH,
};
#[cfg(feature = "bam_downstream")]
use crate::commands::benchmark::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
    LocalPipelineDagValidationNodeReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;
#[cfg(feature = "bam_downstream")]
use std::fs;

pub(crate) const DEFAULT_BAM_GENOTYPING_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.genotyping.complete.json";
const BAM_GENOTYPING_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_genotyping_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.genotyping";
const EXPECTED_TOOL_ID: &str = "angsd";
const EXPECTED_SAMPLE_ID: &str = "human_like_genotyping_candidate_panel";
const EXPECTED_REFERENCE_FASTA: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";
const EXPECTED_SITES_VCF: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf";
const EXPECTED_REGIONS: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt";
const EXPECTED_BAM_PIPELINE_ID: &str = "bam-genotyping";
const EXPECTED_CROSS_PIPELINE_ID: &str = "bam-genotyping-to-vcf-downstream";
const EXPECTED_VCF_STATS_TOOL_ID: &str = "bcftools";
const EXPECTED_VCF_QC_TOOL_ID: &str = "bcftools";
const EXPECTED_PARSER_SCHEMA_VERSION: &str = "bijux.bam.genotyping.v1";
const EXPECTED_PARSER_CALL_RATE: f64 = 1.0;
const EXPECTED_PARSER_MEAN_POSTERIOR: f64 = 0.99;
const EXPECTED_MIN_POSTERIOR: f64 = 0.9;
const EXPECTED_MIN_CALL_RATE: f64 = 0.5;
const EXPECTED_VARIANT_COUNT: u64 = 4;
const EXPECTED_SAMPLE_COUNT: u64 = 2;
const EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD: f64 = 0.5;
const EXPECTED_VARIANT_MISSINGNESS_THRESHOLD: f64 = 0.5;
const CHECKED_SURFACE_COUNT: usize = 15;
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["genotyping_report", "summary", "stage_metrics"];
const REQUIRED_PLAN_OUTPUT_IDS: [&str; 7] = [
    "genotyping_report",
    "genotyping_bcf",
    "genotyping_vcf",
    "genotyping_vcf_tbi",
    "genotyping_gl",
    "summary",
    "stage_metrics",
];
const REQUIRED_BAM_PIPELINE_OUTPUTS: [&str; 6] = [
    "genotyping_bcf",
    "genotyping_vcf_gz",
    "genotyping_vcf_tbi",
    "genotyping_gl_json",
    "genotyping_report_json",
    "genotyping_stage_metrics",
];
const REQUIRED_VCF_FILTER_UPSTREAM_INPUTS: [&str; 3] =
    ["genotyping_vcf_gz", "genotyping_vcf_tbi", "genotyping_report_json"];

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingPlan {
    stage_id: String,
    tool_id: String,
    out_dir: String,
    io: LocalGenotypingPlanIo,
    params: LocalGenotypingPlanParams,
    effective_params: LocalGenotypingEffectiveParams,
    command: LocalGenotypingCommand,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingPlanIo {
    outputs: Vec<LocalGenotypingArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingArtifact {
    name: String,
    path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingPlanParams {
    sample_id: String,
    reference: String,
    sites: String,
    regions: String,
    producer_contract: LocalGenotypingProducerContract,
    caller: String,
    tool: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingProducerContract {
    bcf: String,
    vcf: String,
    tbi: String,
    gl: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingEffectiveParams {
    caller: String,
    min_posterior: f64,
    min_call_rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalGenotypingCommand {
    template: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamGenotypingCompleteRow {
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
    pub(crate) local_plan_proof_path: String,
    pub(crate) parser_smoke_proof_path: String,
    pub(crate) bam_pipeline_proof_path: String,
    pub(crate) cross_pipeline_proof_path: String,
    pub(crate) vcf_stats_proof_path: String,
    pub(crate) vcf_qc_proof_path: String,
    pub(crate) local_plan_out_dir: String,
    pub(crate) local_plan_sample_id: String,
    pub(crate) local_plan_reference_fasta: String,
    pub(crate) local_plan_sites_vcf: String,
    pub(crate) local_plan_regions: String,
    pub(crate) local_plan_caller: String,
    pub(crate) local_plan_tool: String,
    pub(crate) local_plan_min_posterior: f64,
    pub(crate) local_plan_min_call_rate: f64,
    pub(crate) local_plan_genotyping_bcf_path: String,
    pub(crate) local_plan_genotyping_vcf_path: String,
    pub(crate) local_plan_genotyping_vcf_tbi_path: String,
    pub(crate) local_plan_genotyping_gl_path: String,
    pub(crate) local_plan_summary_path: String,
    pub(crate) local_plan_stage_metrics_path: String,
    pub(crate) parser_smoke_schema_version: String,
    pub(crate) parser_smoke_call_rate: f64,
    pub(crate) parser_smoke_mean_posterior: f64,
    pub(crate) parser_smoke_reference_fasta: String,
    pub(crate) parser_smoke_sites_vcf: String,
    pub(crate) bam_pipeline_id: String,
    pub(crate) bam_pipeline_node_outputs: Vec<String>,
    pub(crate) cross_pipeline_id: String,
    pub(crate) cross_pipeline_genotyping_outputs: Vec<String>,
    pub(crate) cross_pipeline_vcf_filter_inputs: Vec<String>,
    pub(crate) downstream_variant_count: u64,
    pub(crate) downstream_sample_count: u64,
    pub(crate) downstream_sample_missingness_count: usize,
    pub(crate) downstream_variant_missingness_count: usize,
    pub(crate) downstream_sample_missingness_exclusion_threshold: f64,
    pub(crate) downstream_variant_missingness_exclusion_threshold: f64,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_plan_ready: bool,
    pub(crate) producer_contract_ready: bool,
    pub(crate) parser_smoke_ready: bool,
    pub(crate) bam_pipeline_ready: bool,
    pub(crate) cross_pipeline_ready: bool,
    pub(crate) downstream_handoff_ready: bool,
    pub(crate) downstream_variant_metrics_ready: bool,
    pub(crate) downstream_missingness_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamGenotypingCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) required_plan_output_ids: Vec<String>,
    pub(crate) bam_pipeline_id: String,
    pub(crate) cross_pipeline_id: String,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamGenotypingCompleteRow>,
    pub(crate) violations: Vec<BamGenotypingCompleteRow>,
}

pub(crate) fn run_render_bam_genotyping_complete(
    args: &parse::BenchReadinessRenderBamGenotypingCompleteArgs,
) -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let report = render_bam_genotyping_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_GENOTYPING_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

#[cfg(feature = "bam_downstream")]
pub(crate) fn render_bam_genotyping_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamGenotypingCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_genotyping_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam genotyping completion must keep active scope, command, output, parser, expected-result, report, schema, local plan, exact vcf handoff, and downstream variant-count plus missingness proof"
        ));
    }
    Ok(report)
}

#[cfg(not(feature = "bam_downstream"))]
pub(crate) fn render_bam_genotyping_complete(
    _repo_root: &Path,
    _output_path: PathBuf,
) -> Result<BamGenotypingCompleteReport> {
    Err(anyhow!("bam.genotyping completion requires the `bam_downstream` feature"))
}

#[cfg(feature = "bam_downstream")]
fn build_bam_genotyping_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamGenotypingCompleteReport> {
    let readiness = render_bam_recalibration_genotyping_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH),
    )?;
    let local_plan_path = bijux_dna_api::v1::api::bam::write_local_genotyping_plan()?;
    let local_plan: LocalGenotypingPlan = serde_json::from_str(
        &fs::read_to_string(&local_plan_path)
            .with_context(|| format!("read {}", local_plan_path.display()))?,
    )
    .with_context(|| format!("parse {}", local_plan_path.display()))?;
    let parser_smoke = render_real_output_parser_smoke(
        repo_root,
        PathBuf::from(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH),
    )?;
    let vcf_stats = render_vcf_stats_ready(repo_root, PathBuf::from(DEFAULT_VCF_STATS_READY_PATH))?;
    let vcf_qc = render_vcf_qc_ready(repo_root, PathBuf::from(DEFAULT_VCF_QC_READY_PATH))?;
    let bam_pipeline_report_path =
        repo_root.join("benchmarks/readiness/local-ready/pipeline-dag/bam-genotyping.json");
    let bam_pipeline = validate_pipeline_dag_path(
        repo_root,
        &benchmark_local_pipeline_config_path(repo_root, EXPECTED_BAM_PIPELINE_ID),
        &bam_pipeline_report_path,
    )?;
    let cross_pipeline_report_path = repo_root.join(
        "benchmarks/readiness/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json",
    );
    let cross_pipeline = validate_pipeline_dag_path(
        repo_root,
        &benchmark_local_pipeline_config_path(repo_root, EXPECTED_CROSS_PIPELINE_ID),
        &cross_pipeline_report_path,
    )?;

    if local_plan.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.genotyping local-plan stage `{}`",
            local_plan.stage_id
        ));
    }
    if local_plan.tool_id != EXPECTED_TOOL_ID {
        return Err(anyhow!("unexpected bam.genotyping local-plan tool `{}`", local_plan.tool_id));
    }

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let expected_tool_ids = vec![EXPECTED_TOOL_ID.to_string()];
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.genotyping readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let parser_row = find_parser_smoke_row(&parser_smoke)?;
    let stats_row = find_vcf_stats_row(&vcf_stats)?;
    let qc_row = find_vcf_qc_row(&vcf_qc)?;
    let bam_pipeline_node = find_pipeline_node(&bam_pipeline, EXPECTED_STAGE_ID)?;
    let cross_pipeline_genotyping_node = find_pipeline_node(&cross_pipeline, EXPECTED_STAGE_ID)?;
    let cross_pipeline_vcf_filter_node = find_pipeline_node(&cross_pipeline, "vcf.filter")?;

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_genotyping_complete_row(
                repo_root,
                &local_plan_path,
                readiness_row,
                &local_plan,
                parser_row,
                &bam_pipeline,
                bam_pipeline_node,
                &cross_pipeline,
                cross_pipeline_genotyping_node,
                cross_pipeline_vcf_filter_node,
                &vcf_stats,
                stats_row,
                &vcf_qc,
                qc_row,
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

    Ok(BamGenotypingCompleteReport {
        schema_version: BAM_GENOTYPING_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        required_plan_output_ids: REQUIRED_PLAN_OUTPUT_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        bam_pipeline_id: EXPECTED_BAM_PIPELINE_ID.to_string(),
        cross_pipeline_id: EXPECTED_CROSS_PIPELINE_ID.to_string(),
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    })
}

#[cfg(feature = "bam_downstream")]
#[allow(clippy::too_many_arguments)]
fn build_bam_genotyping_complete_row(
    repo_root: &Path,
    local_plan_path: &Path,
    readiness_row: &BamRecalibrationGenotypingReadyRow,
    local_plan: &LocalGenotypingPlan,
    parser_row: &super::real_output_parser_probe::RealOutputParserSmokeRow,
    bam_pipeline: &crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
    bam_pipeline_node: &LocalPipelineDagValidationNodeReport,
    cross_pipeline: &crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
    cross_pipeline_genotyping_node: &LocalPipelineDagValidationNodeReport,
    cross_pipeline_vcf_filter_node: &LocalPipelineDagValidationNodeReport,
    vcf_stats: &VcfStatsReadyReport,
    stats_row: &VcfStatsReadyRow,
    vcf_qc: &VcfQcReadyReport,
    qc_row: &VcfQcReadyRow,
) -> Result<BamGenotypingCompleteRow> {
    let summary_path = find_plan_output(&local_plan.io.outputs, "summary")?;
    let stage_metrics_path = find_plan_output(&local_plan.io.outputs, "stage_metrics")?;
    let genotyping_bcf_path = find_plan_output(&local_plan.io.outputs, "genotyping_bcf")?;
    let genotyping_vcf_path = find_plan_output(&local_plan.io.outputs, "genotyping_vcf")?;
    let genotyping_vcf_tbi_path = find_plan_output(&local_plan.io.outputs, "genotyping_vcf_tbi")?;
    let genotyping_gl_path = find_plan_output(&local_plan.io.outputs, "genotyping_gl")?;
    let command_shell = local_plan
        .command
        .template
        .last()
        .cloned()
        .ok_or_else(|| anyhow!("bam.genotyping local plan must carry a shell command"))?;

    let local_plan_ready = local_plan.params.sample_id == EXPECTED_SAMPLE_ID
        && local_plan.params.reference == EXPECTED_REFERENCE_FASTA
        && local_plan.params.sites == EXPECTED_SITES_VCF
        && local_plan.params.regions == EXPECTED_REGIONS
        && local_plan.params.caller == EXPECTED_TOOL_ID
        && local_plan.params.tool == EXPECTED_TOOL_ID
        && local_plan.effective_params.caller == EXPECTED_TOOL_ID
        && approx_eq(local_plan.effective_params.min_posterior, EXPECTED_MIN_POSTERIOR)
        && approx_eq(local_plan.effective_params.min_call_rate, EXPECTED_MIN_CALL_RATE)
        && command_shell.contains(EXPECTED_REFERENCE_FASTA)
        && command_shell.contains(EXPECTED_SITES_VCF)
        && command_shell.contains(EXPECTED_REGIONS)
        && command_shell.contains(&genotyping_bcf_path)
        && command_shell.contains(&genotyping_vcf_path);

    let producer_contract_ready = local_plan.params.producer_contract.bcf == genotyping_bcf_path
        && local_plan.params.producer_contract.vcf == genotyping_vcf_path
        && local_plan.params.producer_contract.tbi == genotyping_vcf_tbi_path
        && local_plan.params.producer_contract.gl == genotyping_gl_path
        && declared_output_names(&local_plan.io.outputs)
            == sorted_output_names(&REQUIRED_PLAN_OUTPUT_IDS);

    let parser_smoke_call_rate = parser_row
        .normalized_snapshot
        .get("call_rate")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let parser_smoke_mean_posterior = parser_row
        .normalized_snapshot
        .get("mean_posterior")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let parser_smoke_reference_fasta = parser_row
        .normalized_snapshot
        .get("reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let parser_smoke_sites_vcf = parser_row
        .normalized_snapshot
        .get("sites")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let parser_smoke_ready = parser_row.passed
        && parser_row.parsed_schema_version == EXPECTED_PARSER_SCHEMA_VERSION
        && approx_eq(parser_smoke_call_rate, EXPECTED_PARSER_CALL_RATE)
        && approx_eq(parser_smoke_mean_posterior, EXPECTED_PARSER_MEAN_POSTERIOR)
        && parser_smoke_reference_fasta == EXPECTED_REFERENCE_FASTA
        && parser_smoke_sites_vcf == EXPECTED_SITES_VCF;

    let bam_pipeline_ready = bam_pipeline.valid
        && bam_pipeline.pipeline_id == EXPECTED_BAM_PIPELINE_ID
        && bam_pipeline_node.outputs
            == REQUIRED_BAM_PIPELINE_OUTPUTS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>();
    let cross_pipeline_ready = cross_pipeline.valid
        && cross_pipeline.pipeline_id == EXPECTED_CROSS_PIPELINE_ID
        && cross_pipeline_genotyping_node.outputs
            == REQUIRED_BAM_PIPELINE_OUTPUTS
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>();
    let downstream_handoff_ready = cross_pipeline_vcf_filter_node.upstream_inputs
        == REQUIRED_VCF_FILTER_UPSTREAM_INPUTS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();

    let downstream_variant_metrics_ready = stats_row.coverage_status == "complete"
        && stats_row.tool_id == EXPECTED_VCF_STATS_TOOL_ID
        && stats_row.smoke_variant_count == EXPECTED_VARIANT_COUNT
        && stats_row.smoke_sample_count == EXPECTED_SAMPLE_COUNT;
    let downstream_missingness_ready = qc_row.coverage_status == "complete"
        && qc_row.tool_id == EXPECTED_VCF_QC_TOOL_ID
        && !qc_row.smoke_sample_missingness.is_empty()
        && !qc_row.smoke_variant_missingness.is_empty()
        && approx_eq(
            qc_row.smoke_sample_missingness_exclusion_threshold,
            EXPECTED_SAMPLE_MISSINGNESS_THRESHOLD,
        )
        && approx_eq(
            qc_row.smoke_variant_missingness_exclusion_threshold,
            EXPECTED_VARIANT_MISSINGNESS_THRESHOLD,
        );

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
    if !local_plan_ready {
        missing_surfaces.push("local_plan".to_string());
    }
    if !producer_contract_ready {
        missing_surfaces.push("producer_contract".to_string());
    }
    if !parser_smoke_ready {
        missing_surfaces.push("parser_smoke".to_string());
    }
    if !bam_pipeline_ready {
        missing_surfaces.push("bam_pipeline".to_string());
    }
    if !cross_pipeline_ready {
        missing_surfaces.push("cross_pipeline".to_string());
    }
    if !downstream_handoff_ready {
        missing_surfaces.push("downstream_handoff".to_string());
    }
    if !downstream_variant_metrics_ready {
        missing_surfaces.push("downstream_variant_metrics".to_string());
    }
    if !downstream_missingness_ready {
        missing_surfaces.push("downstream_missingness".to_string());
    }
    let coverage_status = if missing_surfaces.is_empty() { "complete" } else { "incomplete" };
    let reason = if missing_surfaces.is_empty() {
        "binding `bam.genotyping` / `angsd` keeps local-ready BCF/VCF/GL handoff, parser-smoke call-rate proof, and downstream VCF metrics compatibility complete".to_string()
    } else {
        format!(
            "binding `bam.genotyping` / `angsd` is missing {} required completion surface(s): {}",
            missing_surfaces.len(),
            missing_surfaces.join(", ")
        )
    };

    Ok(BamGenotypingCompleteRow {
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
        local_plan_proof_path: path_relative_to_repo(repo_root, local_plan_path),
        parser_smoke_proof_path: parser_row.proof_path.clone(),
        bam_pipeline_proof_path: bam_pipeline.output_path.clone(),
        cross_pipeline_proof_path: cross_pipeline.output_path.clone(),
        vcf_stats_proof_path: vcf_stats.output_path.clone(),
        vcf_qc_proof_path: vcf_qc.output_path.clone(),
        local_plan_out_dir: local_plan.out_dir.clone(),
        local_plan_sample_id: local_plan.params.sample_id.clone(),
        local_plan_reference_fasta: local_plan.params.reference.clone(),
        local_plan_sites_vcf: local_plan.params.sites.clone(),
        local_plan_regions: local_plan.params.regions.clone(),
        local_plan_caller: local_plan.params.caller.clone(),
        local_plan_tool: local_plan.params.tool.clone(),
        local_plan_min_posterior: local_plan.effective_params.min_posterior,
        local_plan_min_call_rate: local_plan.effective_params.min_call_rate,
        local_plan_genotyping_bcf_path: genotyping_bcf_path,
        local_plan_genotyping_vcf_path: genotyping_vcf_path,
        local_plan_genotyping_vcf_tbi_path: genotyping_vcf_tbi_path,
        local_plan_genotyping_gl_path: genotyping_gl_path,
        local_plan_summary_path: summary_path,
        local_plan_stage_metrics_path: stage_metrics_path,
        parser_smoke_schema_version: parser_row.parsed_schema_version.clone(),
        parser_smoke_call_rate,
        parser_smoke_mean_posterior,
        parser_smoke_reference_fasta,
        parser_smoke_sites_vcf,
        bam_pipeline_id: bam_pipeline.pipeline_id.clone(),
        bam_pipeline_node_outputs: bam_pipeline_node.outputs.clone(),
        cross_pipeline_id: cross_pipeline.pipeline_id.clone(),
        cross_pipeline_genotyping_outputs: cross_pipeline_genotyping_node.outputs.clone(),
        cross_pipeline_vcf_filter_inputs: cross_pipeline_vcf_filter_node.upstream_inputs.clone(),
        downstream_variant_count: stats_row.smoke_variant_count,
        downstream_sample_count: stats_row.smoke_sample_count,
        downstream_sample_missingness_count: qc_row.smoke_sample_missingness.len(),
        downstream_variant_missingness_count: qc_row.smoke_variant_missingness.len(),
        downstream_sample_missingness_exclusion_threshold: qc_row
            .smoke_sample_missingness_exclusion_threshold,
        downstream_variant_missingness_exclusion_threshold: qc_row
            .smoke_variant_missingness_exclusion_threshold,
        active_scope_ready: readiness_row.active_scope_ready,
        command_ready: readiness_row.command_ready,
        output_ready: readiness_row.output_ready,
        parser_ready: readiness_row.parser_ready,
        expected_result_ready: readiness_row.expected_result_ready,
        report_ready: readiness_row.report_ready,
        schema_ready: readiness_row.schema_ready,
        local_plan_ready,
        producer_contract_ready,
        parser_smoke_ready,
        bam_pipeline_ready,
        cross_pipeline_ready,
        downstream_handoff_ready,
        downstream_variant_metrics_ready,
        downstream_missingness_ready,
        coverage_status: coverage_status.to_string(),
        missing_surfaces,
        reason,
    })
}

#[cfg(feature = "bam_downstream")]
fn find_parser_smoke_row(
    report: &RealOutputParserSmokeReport,
) -> Result<&super::real_output_parser_probe::RealOutputParserSmokeRow> {
    report
        .rows
        .iter()
        .find(|row| {
            row.representative_stage_id == EXPECTED_STAGE_ID
                && row.representative_tool_id == EXPECTED_TOOL_ID
        })
        .ok_or_else(|| {
            anyhow!("missing real-output parser-smoke row for `bam.genotyping` / `angsd`")
        })
}

#[cfg(feature = "bam_downstream")]
fn find_vcf_stats_row(report: &VcfStatsReadyReport) -> Result<&VcfStatsReadyRow> {
    report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.stats" && row.tool_id == EXPECTED_VCF_STATS_TOOL_ID)
        .ok_or_else(|| anyhow!("missing vcf.stats readiness row for `bcftools`"))
}

#[cfg(feature = "bam_downstream")]
fn find_vcf_qc_row(report: &VcfQcReadyReport) -> Result<&VcfQcReadyRow> {
    report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.qc" && row.tool_id == EXPECTED_VCF_QC_TOOL_ID)
        .ok_or_else(|| anyhow!("missing vcf.qc readiness row for `bcftools`"))
}

#[cfg(feature = "bam_downstream")]
fn find_pipeline_node<'a>(
    report: &'a crate::commands::benchmark::local_pipeline_dag::LocalPipelineDagValidationReport,
    stage_id: &str,
) -> Result<&'a LocalPipelineDagValidationNodeReport> {
    report
        .nodes
        .iter()
        .find(|node| node.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing `{stage_id}` node in pipeline `{}`", report.pipeline_id))
}

#[cfg(feature = "bam_downstream")]
fn find_plan_output(outputs: &[LocalGenotypingArtifact], name: &str) -> Result<String> {
    outputs
        .iter()
        .find(|artifact| artifact.name == name)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("missing bam.genotyping local-plan output `{name}`"))
}

#[cfg(feature = "bam_downstream")]
fn declared_output_names(outputs: &[LocalGenotypingArtifact]) -> Vec<String> {
    let mut names = outputs.iter().map(|artifact| artifact.name.clone()).collect::<Vec<_>>();
    names.sort();
    names
}

fn sorted_output_names(output_ids: &[&str]) -> Vec<String> {
    let mut names = output_ids.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    names.sort();
    names
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
