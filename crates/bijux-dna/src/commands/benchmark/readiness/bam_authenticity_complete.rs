use std::collections::{BTreeMap, BTreeSet};
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

pub(crate) const DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.authenticity.complete.json";
const BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_authenticity_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.authenticity";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.authenticity.local_smoke.report.v1";
const EXPECTED_AUTHENTICITY_REPORT_SCHEMA_VERSION: &str = "bijux.bam.authenticity.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.authenticity_advisory.v1";
const EXPECTED_COMPOSITION_SCHEMA_VERSION: &str = "bijux.bam.authenticity.composition.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.authenticity.local_smoke.metrics.v1";
const EXPECTED_SAMPLE_ID: &str = "adna_damage_non_udg";
const EXPECTED_METHOD: &str = "authenticct";
const EXPECTED_SCORE: f64 = 0.533_333_333_333_333_3;
const EXPECTED_CONFIDENCE: f64 = 0.813_333_333_333_333_4;
const EXPECTED_STATUS: &str = "pass";
const EXPECTED_PMD_LIKE_SIGNAL_PRESENT: bool = true;
const EXPECTED_CONTAMINATION_ESTIMATE: f64 = 0.03;
const CHECKED_SURFACE_COUNT: usize = 12;
const EXPECTED_TOOL_IDS: [&str; 3] = ["authenticct", "damageprofiler", "pmdtools"];
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["authenticity_report", "summary", "stage_metrics"];
const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 5] =
    ["score", "confidence", "status", "pmd_like_signal_present", "advisory_boundary"];
const REQUIRED_EVIDENCE_METRIC_IDS: [&str; 5] =
    ["damage", "contamination", "complexity", "coverage", "mapping"];
const REQUIRED_REPORT_FIELDS: [&str; 3] = ["schema_version", "summary", "composition"];
const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 15] = [
    "expected_score",
    "score",
    "score_delta",
    "expected_confidence",
    "confidence",
    "confidence_delta",
    "expected_status",
    "status",
    "expected_pmd_like_signal_present",
    "pmd_like_signal_present",
    "contamination_estimate",
    "expected_consumed_metric_ids",
    "consumed_metric_ids",
    "missing_metric_ids",
    "expectation_matched",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct LocalAuthenticitySmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    score: f64,
    confidence: f64,
    status: String,
    pmd_like_signal_present: bool,
    #[serde(default)]
    contamination_estimate: Option<f64>,
    consumed_metrics: Vec<String>,
    missing_metrics: Vec<String>,
    authenticity_report: String,
    authenticity_summary: String,
    authenticity_composite: String,
    advisory_boundary: String,
    stage_metrics: String,
    damage_unified_metrics: String,
    contamination_summary: String,
    complexity_summary: String,
    coverage_regime: String,
    mapping_summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ConsumedMetricEntry {
    available: bool,
    source: String,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct AuthenticityCompositeReport {
    schema_version: String,
    stage_id: String,
    score: f64,
    confidence: f64,
    pmd_like_signal_present: bool,
    contamination_cross_check: String,
    consumed_metrics: BTreeMap<String, ConsumedMetricEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct LocalAuthenticitySmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    method: String,
    expected_score: f64,
    score: f64,
    score_delta: f64,
    expected_confidence: f64,
    confidence: f64,
    confidence_delta: f64,
    expected_status: String,
    status: String,
    expected_pmd_like_signal_present: bool,
    pmd_like_signal_present: bool,
    #[serde(default)]
    contamination_estimate: Option<f64>,
    expected_consumed_metric_ids: Vec<String>,
    consumed_metric_ids: Vec<String>,
    missing_metric_ids: Vec<String>,
    expectation_matched: bool,
}

#[derive(Debug, Clone)]
struct AuthenticityStageProof {
    local_smoke_report_path: PathBuf,
    local_smoke_report: LocalAuthenticitySmokeReport,
    authenticity_report: serde_json::Value,
    summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1,
    composition: AuthenticityCompositeReport,
    normalized_metrics: LocalAuthenticitySmokeMetrics,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamAuthenticityCompleteRow {
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
    pub(crate) required_evidence_metric_ids: Vec<String>,
    pub(crate) required_report_fields: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) authenticity_report_path: String,
    pub(crate) authenticity_summary_path: String,
    pub(crate) authenticity_composite_path: String,
    pub(crate) advisory_boundary_path: String,
    pub(crate) stage_metrics_path: String,
    pub(crate) damage_unified_metrics_path: String,
    pub(crate) contamination_summary_path: String,
    pub(crate) complexity_summary_path: String,
    pub(crate) coverage_regime_path: String,
    pub(crate) mapping_summary_path: String,
    pub(crate) local_smoke_schema_version: String,
    pub(crate) authenticity_report_schema_version: String,
    pub(crate) summary_schema_version: String,
    pub(crate) composition_schema_version: String,
    pub(crate) normalized_metrics_schema_version: String,
    pub(crate) local_smoke_report: LocalAuthenticitySmokeReport,
    pub(crate) summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1,
    pub(crate) composition: AuthenticityCompositeReport,
    pub(crate) normalized_metrics: LocalAuthenticitySmokeMetrics,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) authenticity_report_contract_ready: bool,
    pub(crate) summary_ready: bool,
    pub(crate) composition_ready: bool,
    pub(crate) normalized_metrics_ready: bool,
    pub(crate) evidence_consumption_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamAuthenticityCompleteReport {
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
    pub(crate) required_evidence_metric_ids: Vec<String>,
    pub(crate) required_report_fields: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamAuthenticityCompleteRow>,
    pub(crate) violations: Vec<BamAuthenticityCompleteRow>,
}

pub(crate) fn run_render_bam_authenticity_complete(
    args: &parse::BenchReadinessRenderBamAuthenticityCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_authenticity_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_authenticity_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAuthenticityCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_authenticity_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam.authenticity must keep retained tool coverage, active scope, command, output, parser, expected-result, report, schema, local-smoke, advisory summary, evidence composition, normalized metrics, and upstream evidence-path consumption complete"
        ));
    }
    Ok(report)
}

fn build_bam_authenticity_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamAuthenticityCompleteReport> {
    let readiness_report = render_bam_damage_authenticity_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_DAMAGE_AUTHENTICITY_READY_PATH),
    )?;
    let proof = load_authenticity_stage_proof(repo_root)?;

    let mut rows = readiness_report
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .map(|row| build_bam_authenticity_complete_row(repo_root, row, &proof))
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

    Ok(BamAuthenticityCompleteReport {
        schema_version: BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION,
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
        required_evidence_metric_ids: REQUIRED_EVIDENCE_METRIC_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_report_fields: REQUIRED_REPORT_FIELDS
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

fn build_bam_authenticity_complete_row(
    repo_root: &Path,
    authenticity_row: BamDamageAuthenticityReadyRow,
    proof: &AuthenticityStageProof,
) -> Result<BamAuthenticityCompleteRow> {
    let inherited_artifact_set =
        authenticity_row.local_smoke_artifact_paths.iter().cloned().collect::<BTreeSet<_>>();
    let required_artifact_paths = [
        proof.local_smoke_report.authenticity_report.as_str(),
        proof.local_smoke_report.authenticity_summary.as_str(),
        proof.local_smoke_report.authenticity_composite.as_str(),
        proof.local_smoke_report.advisory_boundary.as_str(),
        proof.local_smoke_report.stage_metrics.as_str(),
        proof.local_smoke_report.damage_unified_metrics.as_str(),
        proof.local_smoke_report.contamination_summary.as_str(),
        proof.local_smoke_report.complexity_summary.as_str(),
        proof.local_smoke_report.coverage_regime.as_str(),
        proof.local_smoke_report.mapping_summary.as_str(),
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<Vec<_>>();

    let authenticity_report_contract_ready = proof.local_smoke_report_path
        == repo_root.join(&authenticity_row.local_smoke_proof_path)
        && proof.authenticity_report.get("schema_version").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_AUTHENTICITY_REPORT_SCHEMA_VERSION)
        && proof.authenticity_report.get("summary").is_some()
        && proof.authenticity_report.get("composition").is_some();

    let summary_ready = proof.summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && proof.summary.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.summary.score, EXPECTED_SCORE)
        && float_matches(proof.summary.confidence, EXPECTED_CONFIDENCE)
        && proof.summary.status == EXPECTED_STATUS
        && proof.summary.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof.summary.advisory_boundary.stage_id == EXPECTED_STAGE_ID
        && proof.summary.advisory_boundary.advisory_only;

    let composition_ready = proof.composition.schema_version == EXPECTED_COMPOSITION_SCHEMA_VERSION
        && proof.composition.stage_id == EXPECTED_STAGE_ID
        && float_matches(proof.composition.score, EXPECTED_SCORE)
        && float_matches(proof.composition.confidence, EXPECTED_CONFIDENCE)
        && proof.composition.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && REQUIRED_EVIDENCE_METRIC_IDS.iter().all(|metric_id| {
            proof
                .composition
                .consumed_metrics
                .get(*metric_id)
                .is_some_and(|entry| entry.available && entry.source == "stage_artifact")
        });

    let expected_consumed_metric_ids =
        REQUIRED_EVIDENCE_METRIC_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let normalized_metrics_ready = proof.normalized_metrics.schema_version
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && proof.normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && proof.normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && proof.normalized_metrics.method == EXPECTED_METHOD
        && float_matches(proof.normalized_metrics.expected_score, EXPECTED_SCORE)
        && float_matches(proof.normalized_metrics.score, EXPECTED_SCORE)
        && float_matches(proof.normalized_metrics.score_delta, 0.0)
        && float_matches(proof.normalized_metrics.expected_confidence, EXPECTED_CONFIDENCE)
        && float_matches(proof.normalized_metrics.confidence, EXPECTED_CONFIDENCE)
        && float_matches(proof.normalized_metrics.confidence_delta, 0.0)
        && proof.normalized_metrics.expected_status == EXPECTED_STATUS
        && proof.normalized_metrics.status == EXPECTED_STATUS
        && proof.normalized_metrics.expected_pmd_like_signal_present
            == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof.normalized_metrics.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof
            .normalized_metrics
            .contamination_estimate
            .is_some_and(|value| float_matches(value, EXPECTED_CONTAMINATION_ESTIMATE))
        && proof.normalized_metrics.expected_consumed_metric_ids == expected_consumed_metric_ids
        && proof.normalized_metrics.consumed_metric_ids == expected_consumed_metric_ids
        && proof.normalized_metrics.missing_metric_ids.is_empty()
        && proof.normalized_metrics.expectation_matched;

    let evidence_consumption_ready = proof.local_smoke_report.schema_version
        == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && proof.local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && proof.local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && proof.local_smoke_report.method == EXPECTED_METHOD
        && proof.local_smoke_report.expectation_matched
        && float_matches(proof.local_smoke_report.score, EXPECTED_SCORE)
        && float_matches(proof.local_smoke_report.confidence, EXPECTED_CONFIDENCE)
        && proof.local_smoke_report.status == EXPECTED_STATUS
        && proof.local_smoke_report.pmd_like_signal_present == EXPECTED_PMD_LIKE_SIGNAL_PRESENT
        && proof
            .local_smoke_report
            .contamination_estimate
            .is_some_and(|value| float_matches(value, EXPECTED_CONTAMINATION_ESTIMATE))
        && proof.local_smoke_report.consumed_metrics == expected_consumed_metric_ids
        && proof.local_smoke_report.missing_metrics.is_empty()
        && required_artifact_paths.iter().all(|path| inherited_artifact_set.contains(path))
        && proof
            .composition
            .consumed_metrics
            .get("damage")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.damage_unified_metrics.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("contamination")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.contamination_summary.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("complexity")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.complexity_summary.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("coverage")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.coverage_regime.as_str())
        && proof
            .composition
            .consumed_metrics
            .get("mapping")
            .and_then(|entry| entry.path.as_deref())
            .map(|path| normalize_path_string(repo_root, path))
            .as_deref()
            == Some(proof.local_smoke_report.mapping_summary.as_str());

    let mut missing_surfaces = authenticity_row.missing_surfaces.clone();
    if !authenticity_report_contract_ready {
        missing_surfaces.push("authenticity_report_contract".to_string());
    }
    if !summary_ready {
        missing_surfaces.push("authenticity_summary_contract".to_string());
    }
    if !composition_ready {
        missing_surfaces.push("authenticity_evidence_composition".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_authenticity_metrics".to_string());
    }
    if !evidence_consumption_ready {
        missing_surfaces.push("authenticity_evidence_paths".to_string());
    }

    let coverage_status =
        if authenticity_row.coverage_status == "complete" && missing_surfaces.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        };
    let reason = if coverage_status == "complete" {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, advisory summary, evidence composition, normalized metrics, and upstream evidence-path consumption complete",
            authenticity_row.tool_id
        )
    } else {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` is missing readiness proof for {}",
            authenticity_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    Ok(BamAuthenticityCompleteRow {
        result_id: authenticity_row.result_id,
        stage_id: authenticity_row.stage_id,
        tool_id: authenticity_row.tool_id,
        sample_scope: authenticity_row.sample_scope,
        benchmark_status: authenticity_row.benchmark_status,
        support_status: authenticity_row.support_status,
        adapter_status: authenticity_row.adapter_status,
        parser_status: authenticity_row.parser_status,
        corpus_status: authenticity_row.corpus_status,
        report_section_id: authenticity_row.report_section_id,
        summary_table_id: authenticity_row.summary_table_id,
        command_readiness_kind: authenticity_row.command_readiness_kind,
        required_output_ids: authenticity_row.required_output_ids,
        stage_output_ids: authenticity_row.stage_output_ids,
        expected_schema_extension_id: authenticity_row.expected_schema_extension_id,
        schema_extension_id: authenticity_row.schema_extension_id,
        required_schema_keys: authenticity_row.required_schema_keys,
        schema_required_keys: authenticity_row.schema_required_keys,
        required_local_smoke_fields: authenticity_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_evidence_metric_ids: REQUIRED_EVIDENCE_METRIC_IDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_report_fields: REQUIRED_REPORT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_scope_proof_path: authenticity_row.active_scope_proof_path,
        command_proof_path: authenticity_row.command_proof_path,
        output_contract_proof_path: authenticity_row.output_contract_proof_path,
        parser_proof_path: authenticity_row.parser_proof_path,
        expected_result_proof_path: authenticity_row.expected_result_proof_path,
        report_map_proof_path: authenticity_row.report_map_proof_path,
        schema_proof_path: authenticity_row.schema_proof_path,
        local_smoke_proof_path: authenticity_row.local_smoke_proof_path,
        authenticity_report_path: proof.local_smoke_report.authenticity_report.clone(),
        authenticity_summary_path: proof.local_smoke_report.authenticity_summary.clone(),
        authenticity_composite_path: proof.local_smoke_report.authenticity_composite.clone(),
        advisory_boundary_path: proof.local_smoke_report.advisory_boundary.clone(),
        stage_metrics_path: proof.local_smoke_report.stage_metrics.clone(),
        damage_unified_metrics_path: proof.local_smoke_report.damage_unified_metrics.clone(),
        contamination_summary_path: proof.local_smoke_report.contamination_summary.clone(),
        complexity_summary_path: proof.local_smoke_report.complexity_summary.clone(),
        coverage_regime_path: proof.local_smoke_report.coverage_regime.clone(),
        mapping_summary_path: proof.local_smoke_report.mapping_summary.clone(),
        local_smoke_schema_version: proof.local_smoke_report.schema_version.clone(),
        authenticity_report_schema_version: proof
            .authenticity_report
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string(),
        summary_schema_version: proof.summary.schema_version.clone(),
        composition_schema_version: proof.composition.schema_version.clone(),
        normalized_metrics_schema_version: proof.normalized_metrics.schema_version.clone(),
        local_smoke_report: proof.local_smoke_report.clone(),
        summary: proof.summary.clone(),
        composition: proof.composition.clone(),
        normalized_metrics: proof.normalized_metrics.clone(),
        active_scope_ready: authenticity_row.active_scope_ready,
        command_ready: authenticity_row.command_ready,
        output_ready: authenticity_row.output_ready,
        parser_ready: authenticity_row.parser_ready,
        expected_result_ready: authenticity_row.expected_result_ready,
        report_ready: authenticity_row.report_ready,
        schema_ready: authenticity_row.schema_ready,
        local_smoke_ready: authenticity_row.local_smoke_ready,
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn load_authenticity_stage_proof(repo_root: &Path) -> Result<AuthenticityStageProof> {
    let local_smoke_report_path =
        bijux_dna_api::v1::api::bam::write_local_authenticity_smoke_report()?;
    let local_smoke_report: LocalAuthenticitySmokeReport = serde_json::from_str(
        &fs::read_to_string(&local_smoke_report_path)
            .with_context(|| format!("read {}", local_smoke_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", local_smoke_report_path.display()))?;

    let authenticity_report_path = repo_root.join(&local_smoke_report.authenticity_report);
    let authenticity_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&authenticity_report_path)
            .with_context(|| format!("read {}", authenticity_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", authenticity_report_path.display()))?;

    let summary_path = repo_root.join(&local_smoke_report.authenticity_summary);
    let summary: bijux_dna_domain_bam::BamAuthenticityAdvisoryV1 = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;

    let composition_path = repo_root.join(&local_smoke_report.authenticity_composite);
    let composition: AuthenticityCompositeReport = serde_json::from_str(
        &fs::read_to_string(&composition_path)
            .with_context(|| format!("read {}", composition_path.display()))?,
    )
    .with_context(|| format!("parse {}", composition_path.display()))?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalAuthenticitySmokeMetrics = serde_json::from_str(
        &fs::read_to_string(&stage_metrics_path)
            .with_context(|| format!("read {}", stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_metrics_path.display()))?;

    Ok(AuthenticityStageProof {
        local_smoke_report_path,
        local_smoke_report,
        authenticity_report,
        summary,
        composition,
        normalized_metrics,
    })
}

fn normalize_path_string(repo_root: &Path, path: &str) -> String {
    let candidate = PathBuf::from(path);
    let absolute = if candidate.is_absolute() { candidate } else { repo_root.join(candidate) };
    path_relative_to_repo(repo_root, &absolute)
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
        render_bam_authenticity_complete, BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_authenticity_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_authenticity_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH),
        )
        .expect("render BAM authenticity completion report");

        assert_eq!(report.schema_version, BAM_AUTHENTICITY_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 3);
        assert_eq!(report.complete_row_count, 3);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 12);
        assert_eq!(report.missing_tool_ids, Vec::<String>::new());
        assert_eq!(report.unexpected_tool_ids, Vec::<String>::new());
        assert_eq!(report.violation_count, 0);
        assert!(report.toolset_ready);
        assert!(report.ok);

        let row =
            report.rows.iter().find(|row| row.tool_id == "authenticct").expect("authenticct row");
        assert_eq!(row.stage_id, "bam.authenticity");
        assert_eq!(row.summary_schema_version, "bijux.bam.authenticity_advisory.v1");
        assert_eq!(row.composition_schema_version, "bijux.bam.authenticity.composition.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.authenticity.local_smoke.metrics.v1"
        );
        assert!((row.summary.score - 0.533_333_333_333_333_3).abs() <= 1e-9);
        assert!((row.summary.confidence - 0.813_333_333_333_333_4).abs() <= 1e-9);
        assert_eq!(row.summary.status, "pass");
        assert!(row.local_smoke_report.expectation_matched);
        assert!(row.authenticity_report_contract_ready);
        assert!(row.summary_ready);
        assert!(row.composition_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.evidence_consumption_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
