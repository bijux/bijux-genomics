use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_damage_authenticity_ready::{
    render_bam_damage_authenticity_ready, BamDamageAuthenticityReadyRow,
    DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_DAMAGE_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.damage.complete.json";
const BAM_DAMAGE_COMPLETE_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_damage_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.damage";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.damage.local_smoke.report.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.damage_evidence.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.damage.stage_metrics.v1";
const EXPECTED_PARSER_OUTPUT_SCHEMA_VERSION: &str = "bijux.bam.damage.parser_output.v1";
const EXPECTED_SAMPLE_ID: &str = "adna_damage_non_udg";
const EXPECTED_METHOD: &str = "ngsbriggs";
const EXPECTED_TERMINAL_C_TO_T_5P: f64 = 0.18;
const EXPECTED_TERMINAL_G_TO_A_3P: f64 = 0.11;
const EXPECTED_SHORT_FRAGMENT_FRACTION: f64 = 1.0;
const EXPECTED_DAMAGE_SIGNAL: &str = "moderate";
const EXPECTED_STRICT_PROFILE_UPGRADED: bool = false;
const CHECKED_SURFACE_COUNT: usize = 13;
const EXPECTED_TOOL_IDS: [&str; 6] =
    ["addeam", "damageprofiler", "mapdamage2", "ngsbriggs", "pmdtools", "pydamage"];
const REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"];
const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 7] = [
    "terminal_c_to_t_5p",
    "terminal_g_to_a_3p",
    "short_fragment_fraction",
    "damage_signal",
    "strict_profile_upgraded",
    "advisory_boundary",
    "notes",
];
const REQUIRED_UNIFIED_METRIC_NAMES: [&str; 3] = ["canonical", "tools_seen", "comparison"];
const REQUIRED_PARSER_OUTPUT_FIELDS: [&str; 3] = ["schema_version", "stage_id", "parsed_tools"];
const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 16] = [
    "expected_terminal_c_to_t_5p",
    "terminal_c_to_t_5p",
    "terminal_c_to_t_5p_delta",
    "expected_terminal_g_to_a_3p",
    "terminal_g_to_a_3p",
    "terminal_g_to_a_3p_delta",
    "expected_short_fragment_fraction",
    "short_fragment_fraction",
    "short_fragment_fraction_delta",
    "expected_damage_signal",
    "damage_signal",
    "expected_strict_profile_upgraded",
    "strict_profile_upgraded",
    "expectation_matched",
    "tool_id",
    "tools_seen",
];

#[derive(Debug, Clone, Deserialize)]
struct LocalDamageSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    tools_seen: Vec<String>,
    terminal_c_to_t_5p: f64,
    terminal_g_to_a_3p: f64,
    short_fragment_fraction: f64,
    damage_signal: String,
    strict_profile_upgraded: bool,
    damage_report: String,
    terminal_position_metrics: String,
    parser_output: String,
    advisory_boundary: String,
    udg_regime: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DamageUnifiedMetricsReport {
    canonical: bijux_dna_domain_bam::metrics::DamageMetricsV1,
    tools_seen: Vec<String>,
    #[serde(default)]
    comparison: Option<bijux_dna_domain_bam::metrics::DamageComparisonV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct ParsedDamageToolOutput {
    tool_id: String,
    metrics: bijux_dna_domain_bam::metrics::DamageMetricsV1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DamageParserOutputReport {
    schema_version: String,
    stage_id: String,
    parsed_tools: Vec<ParsedDamageToolOutput>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct LocalDamageSmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_id: String,
    tools_seen: Vec<String>,
    expected_terminal_c_to_t_5p: f64,
    terminal_c_to_t_5p: f64,
    terminal_c_to_t_5p_delta: f64,
    expected_terminal_g_to_a_3p: f64,
    terminal_g_to_a_3p: f64,
    terminal_g_to_a_3p_delta: f64,
    expected_short_fragment_fraction: f64,
    short_fragment_fraction: f64,
    short_fragment_fraction_delta: f64,
    expected_damage_signal: String,
    damage_signal: String,
    expected_strict_profile_upgraded: bool,
    strict_profile_upgraded: bool,
    expectation_matched: bool,
}

#[derive(Debug, Clone)]
struct DamageStageProof {
    local_smoke_report_path: PathBuf,
    local_smoke_report: LocalDamageSmokeReport,
    summary: bijux_dna_domain_bam::BamDamageEvidenceV1,
    unified_metrics: DamageUnifiedMetricsReport,
    parser_output: DamageParserOutputReport,
    normalized_metrics: LocalDamageSmokeMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamDamageCompleteRow {
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
    pub(crate) required_schema_keys: Vec<String>,
    pub(crate) schema_required_keys: Vec<String>,
    pub(crate) required_local_smoke_fields: Vec<String>,
    pub(crate) required_summary_metric_names: Vec<String>,
    pub(crate) required_unified_metric_names: Vec<String>,
    pub(crate) required_parser_output_fields: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) damage_report_path: String,
    pub(crate) terminal_position_metrics_path: String,
    pub(crate) parser_output_path: String,
    pub(crate) advisory_boundary_path: String,
    pub(crate) udg_regime_path: String,
    pub(crate) stage_metrics_path: String,
    pub(crate) local_smoke_schema_version: String,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_method: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_expectation_matched: bool,
    pub(crate) local_smoke_tools_seen: Vec<String>,
    pub(crate) local_smoke_terminal_c_to_t_5p: f64,
    pub(crate) local_smoke_terminal_g_to_a_3p: f64,
    pub(crate) local_smoke_short_fragment_fraction: f64,
    pub(crate) local_smoke_damage_signal: String,
    pub(crate) local_smoke_strict_profile_upgraded: bool,
    pub(crate) summary_schema_version: String,
    pub(crate) summary_terminal_c_to_t_5p: f64,
    pub(crate) summary_terminal_g_to_a_3p: f64,
    pub(crate) summary_short_fragment_fraction: f64,
    pub(crate) summary_damage_signal: String,
    pub(crate) summary_strict_profile_upgraded: bool,
    pub(crate) summary_notes: Vec<String>,
    pub(crate) unified_canonical_metrics: bijux_dna_domain_bam::metrics::DamageMetricsV1,
    pub(crate) unified_tools_seen: Vec<String>,
    pub(crate) unified_comparison: Option<bijux_dna_domain_bam::metrics::DamageComparisonV1>,
    pub(crate) parser_output_schema_version: String,
    pub(crate) parsed_tool_ids: Vec<String>,
    pub(crate) parsed_tools: Vec<ParsedDamageToolOutput>,
    pub(crate) normalized_metrics_schema_version: String,
    pub(crate) normalized_metrics_sample_id: String,
    pub(crate) normalized_metrics_tool_id: String,
    pub(crate) normalized_metrics: LocalDamageSmokeMetrics,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) summary_ready: bool,
    pub(crate) unified_metrics_ready: bool,
    pub(crate) parser_output_contract_ready: bool,
    pub(crate) normalized_metrics_ready: bool,
    pub(crate) damage_metric_consistency_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamDamageCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) observed_tool_ids: Vec<String>,
    pub(crate) missing_tool_ids: Vec<String>,
    pub(crate) unexpected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) required_summary_metric_names: Vec<String>,
    pub(crate) required_unified_metric_names: Vec<String>,
    pub(crate) required_parser_output_fields: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamDamageCompleteRow>,
    pub(crate) violations: Vec<BamDamageCompleteRow>,
}

pub(crate) fn run_render_bam_damage_complete(
    args: &parse::BenchReadinessRenderBamDamageCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_damage_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_DAMAGE_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_damage_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamDamageCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_damage_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam.damage must keep retained tool coverage, active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, unified metrics, parser-output contract, and normalized damage metrics complete"
        ));
    }
    Ok(report)
}

fn build_bam_damage_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamDamageCompleteReport> {
    let readiness_report = render_bam_damage_authenticity_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH),
    )?;
    let proof = load_damage_stage_proof(repo_root)?;

    let mut rows = readiness_report
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .map(|row| build_bam_damage_complete_row(repo_root, row, &proof))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    let expected_tool_id_set = expected_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let observed_tool_id_set = observed_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let missing_tool_ids =
        expected_tool_id_set.difference(&observed_tool_id_set).cloned().collect::<Vec<_>>();
    let unexpected_tool_ids =
        observed_tool_id_set.difference(&expected_tool_id_set).cloned().collect::<Vec<_>>();
    let toolset_ready = missing_tool_ids.is_empty() && unexpected_tool_ids.is_empty();

    let complete_row_count = rows.iter().filter(|row| row.coverage_status == "complete").count();
    let violations =
        rows.iter().filter(|row| row.coverage_status != "complete").cloned().collect::<Vec<_>>();

    Ok(BamDamageCompleteReport {
        schema_version: BAM_DAMAGE_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len().saturating_sub(complete_row_count),
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        observed_tool_ids,
        missing_tool_ids,
        unexpected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_unified_metric_names: REQUIRED_UNIFIED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_parser_output_fields: REQUIRED_PARSER_OUTPUT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        toolset_ready,
        violation_count: violations.len() + usize::from(!toolset_ready),
        ok: toolset_ready && violations.is_empty(),
        rows,
        violations,
    })
}

fn build_bam_damage_complete_row(
    repo_root: &Path,
    damage_row: BamDamageAuthenticityReadyRow,
    proof: &DamageStageProof,
) -> Result<BamDamageCompleteRow> {
    let parsed_tool_ids = proof
        .parser_output
        .parsed_tools
        .iter()
        .map(|tool| tool.tool_id.clone())
        .collect::<Vec<_>>();
    let parsed_tool_id_set = parsed_tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    let local_tool_set =
        proof.local_smoke_report.tools_seen.iter().cloned().collect::<BTreeSet<_>>();
    let inherited_artifact_set =
        damage_row.local_smoke_artifact_paths.iter().cloned().collect::<BTreeSet<_>>();
    let required_artifact_paths = [
        proof.local_smoke_report.damage_report.as_str(),
        proof.local_smoke_report.terminal_position_metrics.as_str(),
        proof.local_smoke_report.parser_output.as_str(),
        proof.local_smoke_report.advisory_boundary.as_str(),
        proof.local_smoke_report.udg_regime.as_str(),
        proof.local_smoke_report.stage_metrics.as_str(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let summary_ready = proof.summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && proof.summary.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.summary.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.summary.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(proof.summary.short_fragment_fraction, EXPECTED_SHORT_FRAGMENT_FRACTION)
        && proof.summary.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.summary.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.summary.advisory_boundary.stage_id == EXPECTED_STAGE_ID
        && proof.summary.advisory_boundary.advisory_only;
    let unified_metrics_ready =
        float_matches(proof.unified_metrics.canonical.c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
            && float_matches(
                proof.unified_metrics.canonical.g_to_a_3p,
                EXPECTED_TERMINAL_G_TO_A_3P,
            )
            && proof.unified_metrics.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
                == local_tool_set
            && proof.unified_metrics.comparison.is_some();
    let parser_output_contract_ready = proof.parser_output.schema_version
        == EXPECTED_PARSER_OUTPUT_SCHEMA_VERSION
        && proof.parser_output.stage_id == EXPECTED_STAGE_ID
        && !proof.parser_output.parsed_tools.is_empty()
        && parsed_tool_id_set == local_tool_set
        && proof.parser_output.parsed_tools.iter().all(|tool| {
            !tool.tool_id.is_empty()
                && tool.metrics.c_to_t_5p.is_finite()
                && tool.metrics.g_to_a_3p.is_finite()
        });
    let normalized_metrics_ready = proof.normalized_metrics.schema_version
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && proof.normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && proof.normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && proof.normalized_metrics.tool_id == EXPECTED_METHOD
        && proof.normalized_metrics.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
            == local_tool_set
        && float_matches(
            proof.normalized_metrics.expected_terminal_c_to_t_5p,
            EXPECTED_TERMINAL_C_TO_T_5P,
        )
        && float_matches(proof.normalized_metrics.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.normalized_metrics.terminal_c_to_t_5p_delta, 0.0)
        && float_matches(
            proof.normalized_metrics.expected_terminal_g_to_a_3p,
            EXPECTED_TERMINAL_G_TO_A_3P,
        )
        && float_matches(proof.normalized_metrics.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(proof.normalized_metrics.terminal_g_to_a_3p_delta, 0.0)
        && float_matches(
            proof.normalized_metrics.expected_short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && float_matches(
            proof.normalized_metrics.short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && float_matches(proof.normalized_metrics.short_fragment_fraction_delta, 0.0)
        && proof.normalized_metrics.expected_damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.normalized_metrics.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.normalized_metrics.expected_strict_profile_upgraded
            == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.normalized_metrics.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && proof.normalized_metrics.expectation_matched;
    let damage_metric_consistency_ready = proof.local_smoke_report_path
        == repo_root.join(&damage_row.local_smoke_proof_path)
        && proof.local_smoke_report.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && proof.local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && proof.local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && proof.local_smoke_report.method == EXPECTED_METHOD
        && proof.local_smoke_report.expectation_matched
        && proof.local_smoke_report.tools_seen.iter().cloned().collect::<BTreeSet<_>>()
            == local_tool_set
        && float_matches(proof.local_smoke_report.terminal_c_to_t_5p, EXPECTED_TERMINAL_C_TO_T_5P)
        && float_matches(proof.local_smoke_report.terminal_g_to_a_3p, EXPECTED_TERMINAL_G_TO_A_3P)
        && float_matches(
            proof.local_smoke_report.short_fragment_fraction,
            EXPECTED_SHORT_FRAGMENT_FRACTION,
        )
        && proof.local_smoke_report.damage_signal == EXPECTED_DAMAGE_SIGNAL
        && proof.local_smoke_report.strict_profile_upgraded == EXPECTED_STRICT_PROFILE_UPGRADED
        && required_artifact_paths.iter().all(|path| inherited_artifact_set.contains(path))
        && proof.local_smoke_report.damage_report.ends_with("damage.summary.json")
        && proof
            .local_smoke_report
            .terminal_position_metrics
            .ends_with("damage.unified_metrics.json")
        && proof.local_smoke_report.parser_output.ends_with("damage.parser_output.json")
        && proof.local_smoke_report.stage_metrics.ends_with("stage.metrics.json")
        && float_matches(
            proof.summary.terminal_c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.summary.terminal_g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.summary.short_fragment_fraction,
            proof.local_smoke_report.short_fragment_fraction,
        )
        && proof.summary.damage_signal == proof.local_smoke_report.damage_signal
        && proof.summary.strict_profile_upgraded
            == proof.local_smoke_report.strict_profile_upgraded
        && float_matches(
            proof.unified_metrics.canonical.c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.unified_metrics.canonical.g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.normalized_metrics.terminal_c_to_t_5p,
            proof.local_smoke_report.terminal_c_to_t_5p,
        )
        && float_matches(
            proof.normalized_metrics.terminal_g_to_a_3p,
            proof.local_smoke_report.terminal_g_to_a_3p,
        )
        && float_matches(
            proof.normalized_metrics.short_fragment_fraction,
            proof.local_smoke_report.short_fragment_fraction,
        )
        && proof.normalized_metrics.damage_signal == proof.local_smoke_report.damage_signal
        && proof.normalized_metrics.strict_profile_upgraded
            == proof.local_smoke_report.strict_profile_upgraded;

    let mut missing_surfaces = damage_row.missing_surfaces.clone();
    if !summary_ready {
        missing_surfaces.push("damage_summary_contract".to_string());
    }
    if !unified_metrics_ready {
        missing_surfaces.push("damage_unified_metrics".to_string());
    }
    if !parser_output_contract_ready {
        missing_surfaces.push("damage_parser_output_contract".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_damage_metrics".to_string());
    }
    if !damage_metric_consistency_ready {
        missing_surfaces.push("damage_metric_consistency".to_string());
    }

    let coverage_status = if damage_row.coverage_status == "complete" && missing_surfaces.is_empty()
    {
        "complete".to_string()
    } else {
        "incomplete".to_string()
    };
    let reason = if coverage_status == "complete" {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, unified metrics, parser-output contract, and normalized damage metrics complete",
            damage_row.tool_id
        )
    } else {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` is missing readiness proof for {}",
            damage_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    Ok(BamDamageCompleteRow {
        result_id: damage_row.result_id,
        stage_id: damage_row.stage_id,
        tool_id: damage_row.tool_id,
        sample_scope: damage_row.sample_scope,
        benchmark_status: damage_row.benchmark_status,
        support_status: damage_row.support_status,
        adapter_status: damage_row.adapter_status,
        parser_status: damage_row.parser_status,
        corpus_status: damage_row.corpus_status,
        report_section_id: damage_row.report_section_id,
        summary_table_id: damage_row.summary_table_id,
        command_readiness_kind: damage_row.command_readiness_kind,
        required_output_ids: damage_row.required_output_ids,
        stage_output_ids: damage_row.stage_output_ids,
        expected_schema_extension_id: damage_row.expected_schema_extension_id,
        schema_extension_id: damage_row.schema_extension_id,
        required_schema_keys: damage_row.required_schema_keys,
        schema_required_keys: damage_row.schema_required_keys,
        required_local_smoke_fields: damage_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_unified_metric_names: REQUIRED_UNIFIED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_parser_output_fields: REQUIRED_PARSER_OUTPUT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_scope_proof_path: damage_row.active_scope_proof_path,
        command_proof_path: damage_row.command_proof_path,
        output_contract_proof_path: damage_row.output_contract_proof_path,
        parser_proof_path: damage_row.parser_proof_path,
        expected_result_proof_path: damage_row.expected_result_proof_path,
        report_map_proof_path: damage_row.report_map_proof_path,
        schema_proof_path: damage_row.schema_proof_path,
        local_smoke_proof_path: damage_row.local_smoke_proof_path,
        damage_report_path: proof.local_smoke_report.damage_report.clone(),
        terminal_position_metrics_path: proof.local_smoke_report.terminal_position_metrics.clone(),
        parser_output_path: proof.local_smoke_report.parser_output.clone(),
        advisory_boundary_path: proof.local_smoke_report.advisory_boundary.clone(),
        udg_regime_path: proof.local_smoke_report.udg_regime.clone(),
        stage_metrics_path: proof.local_smoke_report.stage_metrics.clone(),
        local_smoke_schema_version: proof.local_smoke_report.schema_version.clone(),
        local_smoke_sample_id: proof.local_smoke_report.sample_id.clone(),
        local_smoke_method: proof.local_smoke_report.method.clone(),
        local_smoke_input_bam: proof.local_smoke_report.input_bam.clone(),
        local_smoke_expectation_matched: proof.local_smoke_report.expectation_matched,
        local_smoke_tools_seen: proof.local_smoke_report.tools_seen.clone(),
        local_smoke_terminal_c_to_t_5p: proof.local_smoke_report.terminal_c_to_t_5p,
        local_smoke_terminal_g_to_a_3p: proof.local_smoke_report.terminal_g_to_a_3p,
        local_smoke_short_fragment_fraction: proof.local_smoke_report.short_fragment_fraction,
        local_smoke_damage_signal: proof.local_smoke_report.damage_signal.clone(),
        local_smoke_strict_profile_upgraded: proof.local_smoke_report.strict_profile_upgraded,
        summary_schema_version: proof.summary.schema_version.clone(),
        summary_terminal_c_to_t_5p: proof.summary.terminal_c_to_t_5p,
        summary_terminal_g_to_a_3p: proof.summary.terminal_g_to_a_3p,
        summary_short_fragment_fraction: proof.summary.short_fragment_fraction,
        summary_damage_signal: proof.summary.damage_signal.clone(),
        summary_strict_profile_upgraded: proof.summary.strict_profile_upgraded,
        summary_notes: proof.summary.notes.clone(),
        unified_canonical_metrics: proof.unified_metrics.canonical.clone(),
        unified_tools_seen: proof.unified_metrics.tools_seen.clone(),
        unified_comparison: proof.unified_metrics.comparison.clone(),
        parser_output_schema_version: proof.parser_output.schema_version.clone(),
        parsed_tool_ids,
        parsed_tools: proof.parser_output.parsed_tools.clone(),
        normalized_metrics_schema_version: proof.normalized_metrics.schema_version.clone(),
        normalized_metrics_sample_id: proof.normalized_metrics.sample_id.clone(),
        normalized_metrics_tool_id: proof.normalized_metrics.tool_id.clone(),
        normalized_metrics: proof.normalized_metrics.clone(),
        active_scope_ready: damage_row.active_scope_ready,
        command_ready: damage_row.command_ready,
        output_ready: damage_row.output_ready,
        parser_ready: damage_row.parser_ready,
        expected_result_ready: damage_row.expected_result_ready,
        report_ready: damage_row.report_ready,
        schema_ready: damage_row.schema_ready,
        local_smoke_ready: damage_row.local_smoke_ready,
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn load_damage_stage_proof(repo_root: &Path) -> Result<DamageStageProof> {
    let local_smoke_report_path = bijux_dna_api::v1::api::bam::write_local_damage_smoke_report()?;
    let local_smoke_report: LocalDamageSmokeReport = serde_json::from_str(
        &fs::read_to_string(&local_smoke_report_path)
            .with_context(|| format!("read {}", local_smoke_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", local_smoke_report_path.display()))?;

    let summary_path = repo_root.join(&local_smoke_report.damage_report);
    let summary: bijux_dna_domain_bam::BamDamageEvidenceV1 = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;

    let unified_metrics_path = repo_root.join(&local_smoke_report.terminal_position_metrics);
    let unified_metrics: DamageUnifiedMetricsReport = serde_json::from_str(
        &fs::read_to_string(&unified_metrics_path)
            .with_context(|| format!("read {}", unified_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", unified_metrics_path.display()))?;

    let parser_output_path = repo_root.join(&local_smoke_report.parser_output);
    let parser_output: DamageParserOutputReport = serde_json::from_str(
        &fs::read_to_string(&parser_output_path)
            .with_context(|| format!("read {}", parser_output_path.display()))?,
    )
    .with_context(|| format!("parse {}", parser_output_path.display()))?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalDamageSmokeMetrics = serde_json::from_str(
        &fs::read_to_string(&stage_metrics_path)
            .with_context(|| format!("read {}", stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_metrics_path.display()))?;

    Ok(DamageStageProof {
        local_smoke_report_path,
        local_smoke_report,
        summary,
        unified_metrics,
        parser_output,
        normalized_metrics,
    })
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_damage_complete, BAM_DAMAGE_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_DAMAGE_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_damage_complete_reports_governed_metrics() {
        let root = repo_root();
        let report =
            render_bam_damage_complete(&root, PathBuf::from(DEFAULT_BAM_DAMAGE_COMPLETE_PATH))
                .expect("render BAM damage completion report");

        assert_eq!(report.schema_version, BAM_DAMAGE_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 6);
        assert_eq!(report.complete_row_count, 6);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 13);
        assert_eq!(report.missing_tool_ids, Vec::<String>::new());
        assert_eq!(report.unexpected_tool_ids, Vec::<String>::new());
        assert_eq!(report.violation_count, 0);
        assert!(report.toolset_ready);
        assert!(report.ok);

        let row =
            report.rows.iter().find(|row| row.tool_id == "mapdamage2").expect("mapdamage2 row");
        assert_eq!(row.stage_id, "bam.damage");
        assert_eq!(row.summary_schema_version, "bijux.bam.damage_evidence.v1");
        assert_eq!(row.parser_output_schema_version, "bijux.bam.damage.parser_output.v1");
        assert_eq!(row.normalized_metrics_schema_version, "bijux.bam.damage.stage_metrics.v1");
        assert!((row.summary_terminal_c_to_t_5p - 0.18).abs() <= 1e-9);
        assert!((row.summary_terminal_g_to_a_3p - 0.11).abs() <= 1e-9);
        assert!((row.summary_short_fragment_fraction - 1.0).abs() <= 1e-9);
        assert_eq!(row.summary_damage_signal, "moderate");
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.unified_metrics_ready);
        assert!(row.parser_output_contract_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.damage_metric_consistency_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
