use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::stage_scoring::{
    StageScoringConfig, StageScoringCorrectnessSignal, StageScoringDecisionMode,
    DEFAULT_STAGE_SCORING_PATH,
};
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;
use crate::commands::numeric::checked_f64_from_u64;

pub(crate) const DEFAULT_BAM_TOOL_SCORES_PATH: &str = "runs/bench/micro/bam/BAM_TOOL_SCORES.tsv";

const BAM_TOOL_SCORES_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_tool_scores.v1";
const DEFAULT_FULL_BENCHMARK_REPORT_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamToolScoreStatus {
    Scored,
    InsufficientEvidence,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamToolScoreRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) decision_mode: String,
    pub(crate) correctness_signal: String,
    pub(crate) result_ids: Vec<String>,
    pub(crate) report_row_ids: Vec<String>,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) report_sections: Vec<String>,
    pub(crate) row_statuses: Vec<String>,
    pub(crate) score_status: BamToolScoreStatus,
    pub(crate) truth_correctness_score: Option<f64>,
    pub(crate) truth_correctness_basis: Option<String>,
    pub(crate) contract_correctness_score: Option<f64>,
    pub(crate) contract_correctness_basis: Option<String>,
    pub(crate) retained_reads: Option<u64>,
    pub(crate) dropped_reads: Option<u64>,
    pub(crate) alignment_qc_metric_value: Option<f64>,
    pub(crate) alignment_qc_metric_basis: Option<String>,
    pub(crate) coverage_metric_value: Option<f64>,
    pub(crate) coverage_metric_basis: Option<String>,
    pub(crate) damage_metric_value: Option<f64>,
    pub(crate) damage_metric_basis: Option<String>,
    pub(crate) authenticity_metric_value: Option<f64>,
    pub(crate) authenticity_metric_basis: Option<String>,
    pub(crate) scientific_metric_ids: Vec<String>,
    pub(crate) scientific_metric_summary: Option<String>,
    pub(crate) runtime_seconds: Option<f64>,
    pub(crate) runtime_source: String,
    pub(crate) observed_memory_mb: Option<f64>,
    pub(crate) declared_memory_mb: Option<f64>,
    pub(crate) memory_source: String,
    pub(crate) failure_class: String,
    pub(crate) micro_execution_status: Option<String>,
    pub(crate) score_weight_coverage: f64,
    pub(crate) score_total: Option<f64>,
    pub(crate) evidence_paths: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamToolScoresReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) config_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) scored_row_count: usize,
    pub(crate) insufficient_evidence_row_count: usize,
    pub(crate) blocked_row_count: usize,
    pub(crate) failure_class_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamToolScoreRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct FullBenchmarkReportView {
    rows: Vec<FullBenchmarkReportRowView>,
}

#[derive(Debug, Clone, Deserialize)]
struct FullBenchmarkReportRowView {
    report_row_id: String,
    result_id: Option<String>,
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    report_section: String,
    row_status: String,
    evidence_path: Option<String>,
    detail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MicroBenchmarkReportView {
    runtime_rows: Vec<MicroBenchmarkRuntimeRowView>,
    memory_source_rows: Vec<MicroBenchmarkMemoryRowView>,
}

#[derive(Debug, Clone, Deserialize)]
struct MicroBenchmarkRuntimeRowView {
    domain: String,
    stage_id: String,
    tool_id: String,
    execution_status: String,
    elapsed_seconds: Option<f64>,
    runtime_source: String,
}

#[derive(Debug, Clone, Deserialize)]
struct MicroBenchmarkMemoryRowView {
    domain: String,
    stage_id: String,
    tool_id: String,
    execution_status: String,
    declared_memory_mb: Option<f64>,
    observed_memory_mb: Option<f64>,
    memory_source: String,
}

#[derive(Debug, Clone, Default)]
struct BamEvidenceAggregate {
    source_paths: BTreeSet<String>,
    truth_correctness_score: Option<f64>,
    truth_correctness_basis: Option<String>,
    contract_correctness_score: Option<f64>,
    contract_correctness_basis: Option<String>,
    retained_reads: Option<u64>,
    dropped_reads: Option<u64>,
    alignment_qc_metric_value: Option<f64>,
    alignment_qc_metric_basis: Option<String>,
    coverage_metric_value: Option<f64>,
    coverage_metric_basis: Option<String>,
    damage_metric_value: Option<f64>,
    damage_metric_basis: Option<String>,
    authenticity_metric_value: Option<f64>,
    authenticity_metric_basis: Option<String>,
    scientific_metric_summary: Option<String>,
}

#[derive(Debug, Clone)]
struct BaseScoreRow {
    stage_id: String,
    tool_id: String,
    decision_mode: StageScoringDecisionMode,
    correctness_signal: StageScoringCorrectnessSignal,
    weights: super::stage_scoring::StageScoringWeights,
    scientific_metric_ids: Vec<String>,
    result_ids: Vec<String>,
    report_row_ids: Vec<String>,
    corpus_ids: Vec<String>,
    report_sections: Vec<String>,
    row_statuses: Vec<String>,
    truth_correctness_score: Option<f64>,
    truth_correctness_basis: Option<String>,
    contract_correctness_score: Option<f64>,
    contract_correctness_basis: Option<String>,
    retained_reads: Option<u64>,
    dropped_reads: Option<u64>,
    alignment_qc_metric_value: Option<f64>,
    alignment_qc_metric_basis: Option<String>,
    coverage_metric_value: Option<f64>,
    coverage_metric_basis: Option<String>,
    damage_metric_value: Option<f64>,
    damage_metric_basis: Option<String>,
    authenticity_metric_value: Option<f64>,
    authenticity_metric_basis: Option<String>,
    scientific_metric_summary: Option<String>,
    runtime_seconds: Option<f64>,
    runtime_source: String,
    micro_execution_status: Option<String>,
    effective_memory_mb: Option<f64>,
    observed_memory_mb: Option<f64>,
    declared_memory_mb: Option<f64>,
    memory_source: String,
    failure_class: String,
    evidence_paths: Vec<String>,
    reason: String,
}

pub(crate) fn run_render_bam_tool_scores(
    args: &parse::BenchReadinessRenderBamToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_tool_scores(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_bam_tool_scores(
    args: &parse::BenchReadinessValidateBamToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_bam_tool_scores(
        &repo_root,
        args.input.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_tool_scores(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamToolScoresReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (config_path, rows) = collect_bam_tool_score_rows(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_tool_scores_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(build_report(
        path_relative_to_repo(repo_root, &output_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

pub(crate) fn validate_bam_tool_scores(
    repo_root: &Path,
    input_path: PathBuf,
) -> Result<BamToolScoresReport> {
    let input_path = repo_relative_path(repo_root, &input_path);
    let actual = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let (config_path, rows) = collect_bam_tool_score_rows(repo_root)?;
    let expected = render_bam_tool_scores_tsv(&rows);
    if actual != expected {
        bail!(
            "BAM tool score TSV drifted from governed evidence contracts; rerun `bijux-dna bench readiness render-bam-tool-scores`"
        );
    }
    Ok(build_report(
        path_relative_to_repo(repo_root, &input_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

fn collect_bam_tool_score_rows(repo_root: &Path) -> Result<(PathBuf, Vec<BamToolScoreRow>)> {
    let config_path = repo_root.join(DEFAULT_STAGE_SCORING_PATH);
    let config = load_stage_scoring_config(&config_path)?;
    let failure_catalog = config
        .failure_classes
        .iter()
        .map(|row| (row.class_id.clone(), row.detail.clone()))
        .collect::<BTreeMap<_, _>>();
    let full_report = load_full_benchmark_report(repo_root)?;
    let micro_report = load_micro_benchmark_report(repo_root)?;
    let evidence = load_bam_evidence(repo_root)?;

    let full_rows_by_binding = full_report.rows.into_iter().filter(|row| row.domain == "bam").fold(
        BTreeMap::<(String, String), Vec<FullBenchmarkReportRowView>>::new(),
        |mut map, row| {
            map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            map
        },
    );
    let runtime_by_binding =
        micro_report.runtime_rows.into_iter().filter(|row| row.domain == "bam").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkRuntimeRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );
    let memory_by_binding =
        micro_report.memory_source_rows.into_iter().filter(|row| row.domain == "bam").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkMemoryRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );

    let bam_stage_rows = config.rows.iter().filter(|row| row.domain == "bam").collect::<Vec<_>>();

    let mut base_rows = Vec::new();
    for stage_row in &bam_stage_rows {
        for tool_id in &stage_row.benchmark_ready_tool_ids {
            let binding_key = (stage_row.stage_id.clone(), tool_id.clone());
            let binding_rows = full_rows_by_binding.get(&binding_key).cloned().unwrap_or_default();
            let evidence_row = evidence.get(&binding_key);
            let runtime_row = select_runtime_row(
                stage_row,
                tool_id.as_str(),
                &runtime_by_binding,
                evidence_row.is_some(),
            );
            let memory_row = select_memory_row(
                stage_row,
                tool_id.as_str(),
                &memory_by_binding,
                evidence_row.is_some(),
            );
            let failure_class =
                classify_failure_class(&binding_rows, evidence_row, runtime_row.as_ref());
            let reason = describe_row_reason(
                &failure_class,
                evidence_row,
                runtime_row.as_ref(),
                memory_row.as_ref(),
                stage_row,
                tool_id.as_str(),
                &failure_catalog,
            );

            base_rows.push(BaseScoreRow {
                stage_id: stage_row.stage_id.clone(),
                tool_id: tool_id.clone(),
                decision_mode: stage_row.decision_mode,
                correctness_signal: stage_row.correctness.signal,
                weights: stage_row.weights.clone(),
                scientific_metric_ids: stage_row.correctness.metric_ids.clone(),
                result_ids: unique_strings(
                    binding_rows.iter().filter_map(|row| row.result_id.clone()),
                ),
                report_row_ids: unique_strings(
                    binding_rows.iter().map(|row| row.report_row_id.clone()),
                ),
                corpus_ids: unique_strings(binding_rows.iter().map(|row| row.corpus_id.clone())),
                report_sections: unique_strings(
                    binding_rows.iter().map(|row| row.report_section.clone()),
                ),
                row_statuses: unique_strings(
                    binding_rows.iter().map(normalized_row_status_for_scoring),
                ),
                truth_correctness_score: evidence_row.and_then(|row| row.truth_correctness_score),
                truth_correctness_basis: evidence_row
                    .and_then(|row| row.truth_correctness_basis.clone()),
                contract_correctness_score: evidence_row
                    .and_then(|row| row.contract_correctness_score),
                contract_correctness_basis: evidence_row
                    .and_then(|row| row.contract_correctness_basis.clone()),
                retained_reads: evidence_row.and_then(|row| row.retained_reads),
                dropped_reads: evidence_row.and_then(|row| row.dropped_reads),
                alignment_qc_metric_value: evidence_row
                    .and_then(|row| row.alignment_qc_metric_value),
                alignment_qc_metric_basis: evidence_row
                    .and_then(|row| row.alignment_qc_metric_basis.clone()),
                coverage_metric_value: evidence_row.and_then(|row| row.coverage_metric_value),
                coverage_metric_basis: evidence_row
                    .and_then(|row| row.coverage_metric_basis.clone()),
                damage_metric_value: evidence_row.and_then(|row| row.damage_metric_value),
                damage_metric_basis: evidence_row.and_then(|row| row.damage_metric_basis.clone()),
                authenticity_metric_value: evidence_row
                    .and_then(|row| row.authenticity_metric_value),
                authenticity_metric_basis: evidence_row
                    .and_then(|row| row.authenticity_metric_basis.clone()),
                scientific_metric_summary: evidence_row
                    .and_then(|row| row.scientific_metric_summary.clone()),
                runtime_seconds: runtime_row.as_ref().and_then(|row| row.elapsed_seconds),
                runtime_source: runtime_row
                    .as_ref()
                    .map_or_else(|| "not_available".to_string(), |row| row.runtime_source.clone()),
                micro_execution_status: runtime_row
                    .as_ref()
                    .map(|row| row.execution_status.clone())
                    .or_else(|| memory_row.as_ref().map(|row| row.execution_status.clone())),
                effective_memory_mb: memory_row
                    .as_ref()
                    .and_then(|row| row.observed_memory_mb.or(row.declared_memory_mb)),
                observed_memory_mb: memory_row.as_ref().and_then(|row| row.observed_memory_mb),
                declared_memory_mb: memory_row.as_ref().and_then(|row| row.declared_memory_mb),
                memory_source: memory_row
                    .as_ref()
                    .map_or_else(|| "not_available".to_string(), |row| row.memory_source.clone()),
                failure_class,
                evidence_paths: evidence_row
                    .map(|row| row.source_paths.iter().cloned().collect::<Vec<_>>())
                    .unwrap_or_default(),
                reason,
            });
        }
    }

    let runtime_scores = build_runtime_scores(&base_rows);
    let memory_scores = build_memory_scores(&base_rows);
    let mut rows = Vec::new();
    for row in base_rows {
        let correctness_component = match row.correctness_signal {
            StageScoringCorrectnessSignal::ScientificComparableMetrics => {
                row.truth_correctness_score
            }
            StageScoringCorrectnessSignal::OutputContract => row.contract_correctness_score,
        };
        let scientific_threshold_score = if row.correctness_signal
            == StageScoringCorrectnessSignal::ScientificComparableMetrics
        {
            row.truth_correctness_score
        } else {
            None
        };
        let completion_score = completion_score_for_row(&row);
        let failure_class_score = Some(if row.failure_class == "none" { 1.0 } else { 0.0 });
        let runtime_score =
            runtime_scores.get(&(row.stage_id.clone(), row.tool_id.clone())).copied();
        let memory_score = memory_scores.get(&(row.stage_id.clone(), row.tool_id.clone())).copied();
        let (score_status, score_weight_coverage, score_total) = score_row(
            &row,
            correctness_component,
            scientific_threshold_score,
            runtime_score,
            memory_score,
            completion_score,
            failure_class_score,
        );

        rows.push(BamToolScoreRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            decision_mode: decision_mode_label(row.decision_mode).to_string(),
            correctness_signal: correctness_signal_label(row.correctness_signal).to_string(),
            result_ids: row.result_ids,
            report_row_ids: row.report_row_ids,
            corpus_ids: row.corpus_ids,
            report_sections: row.report_sections,
            row_statuses: row.row_statuses,
            score_status,
            truth_correctness_score: row.truth_correctness_score,
            truth_correctness_basis: row.truth_correctness_basis,
            contract_correctness_score: row.contract_correctness_score,
            contract_correctness_basis: row.contract_correctness_basis,
            retained_reads: row.retained_reads,
            dropped_reads: row.dropped_reads,
            alignment_qc_metric_value: row.alignment_qc_metric_value,
            alignment_qc_metric_basis: row.alignment_qc_metric_basis,
            coverage_metric_value: row.coverage_metric_value,
            coverage_metric_basis: row.coverage_metric_basis,
            damage_metric_value: row.damage_metric_value,
            damage_metric_basis: row.damage_metric_basis,
            authenticity_metric_value: row.authenticity_metric_value,
            authenticity_metric_basis: row.authenticity_metric_basis,
            scientific_metric_ids: row.scientific_metric_ids,
            scientific_metric_summary: row.scientific_metric_summary,
            runtime_seconds: row.runtime_seconds,
            runtime_source: row.runtime_source,
            observed_memory_mb: row.observed_memory_mb,
            declared_memory_mb: row.declared_memory_mb,
            memory_source: row.memory_source,
            failure_class: row.failure_class,
            micro_execution_status: row.micro_execution_status,
            score_weight_coverage,
            score_total,
            evidence_paths: row.evidence_paths,
            reason: row.reason,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    Ok((config_path, rows))
}

fn load_stage_scoring_config(path: &Path) -> Result<StageScoringConfig> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).context("parse stage-scoring TOML")
}

fn load_full_benchmark_report(repo_root: &Path) -> Result<FullBenchmarkReportView> {
    let path = repo_root.join(DEFAULT_FULL_BENCHMARK_REPORT_PATH);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).context("parse full benchmark report JSON")
}

fn load_micro_benchmark_report(repo_root: &Path) -> Result<MicroBenchmarkReportView> {
    let path = repo_root.join(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).context("parse micro benchmark report JSON")
}

fn load_bam_evidence(repo_root: &Path) -> Result<BTreeMap<(String, String), BamEvidenceAggregate>> {
    let mut rows = BTreeMap::<(String, String), BamEvidenceAggregate>::new();

    merge_validate_summary(repo_root, &mut rows)?;
    merge_qc_pre_summary(repo_root, &mut rows)?;
    merge_mapping_summary(repo_root, &mut rows)?;
    merge_filter_summary(repo_root, &mut rows)?;
    merge_mapq_filter_summary(repo_root, &mut rows)?;
    merge_length_filter_summary(repo_root, &mut rows)?;
    merge_markdup_summary(repo_root, &mut rows)?;
    merge_duplication_metrics_summary(repo_root, &mut rows)?;
    merge_coverage_summary(repo_root, &mut rows)?;
    merge_damage_summary(repo_root, &mut rows)?;
    merge_authenticity_summary(repo_root, &mut rows)?;
    merge_contamination_summary(repo_root, &mut rows)?;
    merge_sex_summary(repo_root, &mut rows)?;
    merge_haplogroups_summary(repo_root, &mut rows)?;
    merge_complexity_summary(repo_root, &mut rows)?;
    merge_endogenous_content_summary(repo_root, &mut rows)?;
    merge_insert_size_summary(repo_root, &mut rows)?;
    merge_kinship_summary(repo_root, &mut rows)?;
    merge_overlap_correction_summary(repo_root, &mut rows)?;
    merge_recalibration_summary(repo_root, &mut rows)?;
    merge_gc_bias_summary(repo_root, &mut rows)?;

    Ok(rows)
}

fn merge_validate_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.validate/validation.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    let cases = required_array(&root, "cases")?;
    let case_count = required_u64(&root, "case_count")?;
    let matched_cases = cases
        .iter()
        .filter(|case| case.get("expectation_matched").and_then(Value::as_bool) == Some(true))
        .count() as u64;
    let pass_cases = cases
        .iter()
        .filter(|case| case.get("validation_status").and_then(Value::as_str) == Some("pass"))
        .count() as u64;
    let retained_reads =
        cases.iter().filter_map(|case| optional_u64(case, "total_reads")).sum::<u64>();
    let validation_statuses = unique_strings(
        cases.iter().filter_map(|case| optional_str(case, "validation_status").map(str::to_string)),
    );
    let refusal_codes = unique_strings(cases.iter().flat_map(|case| {
        case.get("refusal_codes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
    }));
    let summary = summarize_metrics(vec![
        format!("case_count={case_count}"),
        format!("matched_cases={matched_cases}"),
        format!("pass_cases={pass_cases}"),
        format!("validation_statuses={}", validation_statuses.join(",")),
        format!("refusal_codes={}", refusal_codes.join(",")),
    ]);
    insert_evidence(
        rows,
        stage_id,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(matched_cases, case_count)?),
            truth_correctness_basis: Some("case_expectation_match_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "all_cases_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "validation smoke cases covered governed pass and refusal expectations".to_string(),
            ),
            retained_reads: Some(retained_reads),
            alignment_qc_metric_value: Some(fraction_u64(pass_cases, case_count)?),
            alignment_qc_metric_basis: Some("pass_case_fraction".to_string()),
            scientific_metric_summary: summary,
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_qc_pre_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.qc_pre/qc_pre.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    for case in required_array(&root, "cases")? {
        let tool_id = required_str(case, "tool_id")?;
        let total_reads = required_u64(case, "total_reads")?;
        let mapped_reads = required_u64(case, "mapped_reads")?;
        let unmapped_reads = required_u64(case, "unmapped_reads")?;
        let duplicate_flagged_reads = required_u64(case, "duplicate_flagged_reads")?;
        let summary = summarize_metrics(vec![
            format!("mapped_reads={mapped_reads}"),
            format!("unmapped_reads={unmapped_reads}"),
            format!("duplicate_flagged_reads={duplicate_flagged_reads}"),
        ]);
        insert_evidence(
            rows,
            stage_id,
            tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some(fraction_u64(mapped_reads, total_reads)?),
                truth_correctness_basis: Some("mapped_fraction".to_string()),
                contract_correctness_score: Some(if required_bool(case, "expectation_matched")? {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "pre-QC smoke report preserved governed mapping and duplicate surfaces"
                        .to_string(),
                ),
                retained_reads: Some(total_reads),
                alignment_qc_metric_value: Some(fraction_u64(mapped_reads, total_reads)?),
                alignment_qc_metric_basis: Some("mapped_fraction".to_string()),
                scientific_metric_summary: summary,
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            &path,
        );
    }
    Ok(())
}

fn merge_mapping_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.mapping_summary/mapping_summary.tsv");
    let tsv_rows = read_tsv_rows(&path)?;
    for row in tsv_rows {
        let stage_id = "bam.mapping_summary";
        let total_reads = parse_u64(required_tsv(&row, "total_reads")?)?;
        let mapped_reads = parse_u64(required_tsv(&row, "mapped_reads")?)?;
        let unmapped_reads = parse_u64(required_tsv(&row, "unmapped_reads")?)?;
        let secondary_reads = parse_u64(required_tsv(&row, "secondary_reads")?)?;
        let supplementary_reads = parse_u64(required_tsv(&row, "supplementary_reads")?)?;
        let mapping_fraction = parse_f64(required_tsv(&row, "mapping_fraction")?)?;
        let summary = summarize_metrics(vec![
            format!("mapped_reads={mapped_reads}"),
            format!("unmapped_reads={unmapped_reads}"),
            format!("secondary_reads={secondary_reads}"),
            format!("supplementary_reads={supplementary_reads}"),
        ]);
        insert_evidence(
            rows,
            stage_id,
            "samtools",
            BamEvidenceAggregate {
                truth_correctness_score: Some(mapping_fraction),
                truth_correctness_basis: Some("mapping_fraction".to_string()),
                contract_correctness_score: Some(
                    if parse_bool(required_tsv(&row, "expectation_matched")?)? { 1.0 } else { 0.0 },
                ),
                contract_correctness_basis: Some(
                    "mapping summary TSV preserved governed mapped and unmapped count surfaces"
                        .to_string(),
                ),
                retained_reads: Some(total_reads),
                alignment_qc_metric_value: Some(mapping_fraction),
                alignment_qc_metric_basis: Some("mapping_fraction".to_string()),
                scientific_metric_summary: summary,
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            &path,
        );
    }
    Ok(())
}

fn merge_filter_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.filter/filter_metrics.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let input_reads = required_u64(&root, "input_reads")?;
    let kept_reads = required_u64(&root, "kept_reads")?;
    let removed_reads = required_u64(&root, "removed_reads")?;
    let active_filters = unique_strings(
        required_array(&root, "active_filters")?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string),
    );
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "filter smoke report preserved governed kept and removed read counts".to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(fraction_u64(kept_reads, input_reads)?),
            alignment_qc_metric_basis: Some("kept_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("input_reads={input_reads}"),
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("active_filters={}", active_filters.join(",")),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_mapq_filter_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.mapq_filter/mapq_filter.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let input_reads = required_u64(&root, "input_reads")?;
    let kept_reads = required_u64(&root, "kept_reads")?;
    let removed_reads = required_u64(&root, "removed_reads")?;
    let mapped_fraction_retained = required_f64(&root, "mapped_fraction_retained")?;
    let mapq_threshold = required_u64(&root, "mapq_threshold")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "mapq filter report preserved governed retained and removed read counts"
                    .to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(mapped_fraction_retained),
            alignment_qc_metric_basis: Some("mapped_fraction_retained".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("mapq_threshold={mapq_threshold}"),
                format!("mapped_fraction_retained={}", format_f64(mapped_fraction_retained)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_length_filter_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.length_filter/length_filter.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let input_reads = required_u64(&root, "input_reads")?;
    let kept_reads = required_u64(&root, "kept_reads")?;
    let removed_reads = required_u64(&root, "removed_reads")?;
    let min_length_threshold = required_u64(&root, "min_length_threshold")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "length filter report preserved governed retained and removed read counts"
                    .to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(fraction_u64(kept_reads, input_reads)?),
            alignment_qc_metric_basis: Some("kept_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("min_length_threshold={min_length_threshold}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_markdup_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.markdup/duplicates.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let duplicate_fraction = required_f64(&root, "duplicate_fraction")?;
    let duplicate_count = required_u64(&root, "duplicate_count")?;
    let output_reads = required_u64(&root, "output_reads")?;
    let removed_reads = required_u64(&root, "removed_reads")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            truth_correctness_basis: Some("one_minus_duplicate_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "markdup smoke report preserved governed duplicate marking outputs".to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            alignment_qc_metric_basis: Some("one_minus_duplicate_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("duplicate_count={duplicate_count}"),
                format!("duplicate_fraction={}", format_f64(duplicate_fraction)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_duplication_metrics_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path =
        repo_root.join("runs/bench/local-smoke/bam.duplication_metrics/duplication_metrics.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let duplicate_fraction = required_f64(&root, "duplicate_fraction")?;
    let duplicate_count = required_u64(&root, "duplicate_count")?;
    let examined_reads = required_u64(&root, "examined_reads")?;
    let estimated_library_size =
        optional_str(&root, "estimated_library_size").unwrap_or("").to_string();
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            truth_correctness_basis: Some("one_minus_duplicate_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "duplication metrics report preserved governed duplicate count and fraction surfaces"
                    .to_string(),
            ),
            retained_reads: Some(examined_reads),
            alignment_qc_metric_value: Some((1.0 - duplicate_fraction).clamp(0.0, 1.0)),
            alignment_qc_metric_basis: Some("one_minus_duplicate_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("duplicate_count={duplicate_count}"),
                format!("duplicate_fraction={}", format_f64(duplicate_fraction)),
                format!("estimated_library_size={estimated_library_size}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_coverage_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.coverage/coverage.tsv");
    let tsv_rows = read_tsv_rows(&path)?;
    if tsv_rows.is_empty() {
        return Ok(());
    }
    let mut breadth_sum = 0.0;
    let mut mean_depth_sum = 0.0;
    let mut covered_bases_total = 0_u64;
    let mut matched = true;
    let mut count = 0_u64;
    for row in &tsv_rows {
        breadth_sum += parse_f64(required_tsv(row, "breadth_1x")?)?;
        mean_depth_sum += parse_f64(required_tsv(row, "mean_depth")?)?;
        covered_bases_total += parse_u64(required_tsv(row, "covered_bases")?)?;
        matched &= parse_bool(required_tsv(row, "case_expectation_matched")?)?;
        count += 1;
    }
    let mean_breadth = breadth_sum / checked_f64_from_u64(count, "coverage row count")?;
    let mean_depth = mean_depth_sum / checked_f64_from_u64(count, "coverage row count")?;
    insert_evidence(
        rows,
        "bam.coverage",
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(mean_breadth),
            truth_correctness_basis: Some("mean_breadth_1x".to_string()),
            contract_correctness_score: Some(if matched { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "coverage TSV preserved governed breadth, covered bases, and depth surfaces"
                    .to_string(),
            ),
            coverage_metric_value: Some(mean_depth),
            coverage_metric_basis: Some("mean_depth".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("breadth_1x_mean={}", format_f64(mean_breadth)),
                format!("covered_bases_total={covered_bases_total}"),
                format!("mean_depth_mean={}", format_f64(mean_depth)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_damage_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.damage/damage.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let terminal_c_to_t_5p = required_f64(&root, "terminal_c_to_t_5p")?;
    let terminal_g_to_a_3p = required_f64(&root, "terminal_g_to_a_3p")?;
    let damage_signal = required_str(&root, "damage_signal")?;
    let score = f64::midpoint(terminal_c_to_t_5p, terminal_g_to_a_3p);
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "method")?,
        BamEvidenceAggregate {
            truth_correctness_score: Some(score),
            truth_correctness_basis: Some("mean_terminal_substitution_rate".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "damage smoke report preserved governed terminal substitution and damage label surfaces"
                    .to_string(),
            ),
            damage_metric_value: Some(score),
            damage_metric_basis: Some("mean_terminal_substitution_rate".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("damage_signal={damage_signal}"),
                format!("terminal_c_to_t_5p={}", format_f64(terminal_c_to_t_5p)),
                format!("terminal_g_to_a_3p={}", format_f64(terminal_g_to_a_3p)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_authenticity_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.authenticity/authenticity.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let score = required_f64(&root, "score")?;
    let confidence = required_f64(&root, "confidence")?;
    let pmd_like_signal_present = required_bool(&root, "pmd_like_signal_present")?;
    let status = required_str(&root, "status")?;
    let truth_score = (score
        + confidence
        + f64::from(u8::from(pmd_like_signal_present))
        + if status == "pass" { 1.0 } else { 0.0 })
        / 4.0;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "method")?,
        BamEvidenceAggregate {
            truth_correctness_score: Some(truth_score),
            truth_correctness_basis: Some("authenticity_metric_mean".to_string()),
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "authenticity smoke report preserved governed score, confidence, and status surfaces"
                    .to_string(),
            ),
            authenticity_metric_value: Some(score),
            authenticity_metric_basis: Some("score".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("confidence={}", format_f64(confidence)),
                format!("pmd_like_signal_present={pmd_like_signal_present}"),
                format!("status={status}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_contamination_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.contamination/local_smoke.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    let by_tool = required_array(&root, "rows")?.iter().fold(
        BTreeMap::<String, Vec<&Value>>::new(),
        |mut map, row| {
            if let Some(tool_id) = optional_str(row, "tool_id") {
                map.entry(tool_id.to_string()).or_default().push(row);
            }
            map
        },
    );
    for (tool_id, tool_rows) in by_tool {
        let selected =
            tool_rows.iter().find(|row| optional_str(row, "proof_case") == Some("ready")).copied();
        let Some(row) = selected else {
            continue;
        };
        let raw_estimate = required_f64(row, "raw_estimate")?;
        let raw_ci_low = required_f64(row, "raw_ci_low")?;
        let raw_ci_high = required_f64(row, "raw_ci_high")?;
        insert_evidence(
            rows,
            stage_id,
            &tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some((1.0 - raw_estimate).clamp(0.0, 1.0)),
                truth_correctness_basis: Some("one_minus_contamination_estimate".to_string()),
                contract_correctness_score: Some(if required_bool(row, "expectation_matched")?
                    && required_bool(row, "prerequisites_passed")?
                {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "contamination smoke report preserved governed estimate and prerequisite surfaces"
                        .to_string(),
                ),
                authenticity_metric_value: Some(raw_estimate),
                authenticity_metric_basis: Some("estimate".to_string()),
                scientific_metric_summary: summarize_metrics(vec![
                    format!("estimate={}", format_f64(raw_estimate)),
                    format!("ci_low={}", format_f64(raw_ci_low)),
                    format!("ci_high={}", format_f64(raw_ci_high)),
                ]),
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            &path,
        );
    }
    Ok(())
}

fn merge_sex_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.sex/tool_smoke.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    let by_tool = required_array(&root, "rows")?.iter().fold(
        BTreeMap::<String, Vec<&Value>>::new(),
        |mut map, row| {
            if let Some(tool_id) = optional_str(row, "tool_id") {
                map.entry(tool_id.to_string()).or_default().push(row);
            }
            map
        },
    );
    for (tool_id, tool_rows) in by_tool {
        let selected =
            tool_rows.iter().find(|row| optional_str(row, "proof_case") == Some("ready")).copied();
        let Some(row) = selected else {
            continue;
        };
        let confidence = required_f64(row, "confidence")?;
        insert_evidence(
            rows,
            stage_id,
            &tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some(confidence),
                truth_correctness_basis: Some("confidence".to_string()),
                contract_correctness_score: Some(if required_bool(row, "expectation_matched")? {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "sex tool smoke report preserved governed call, confidence, and coverage surfaces"
                        .to_string(),
                ),
                scientific_metric_summary: summarize_metrics(vec![
                    format!("call={}", required_str(row, "call")?),
                    format!("status={}", required_str(row, "status")?),
                    format!("x_coverage={}", format_f64(required_f64(row, "x_coverage")?)),
                    format!("y_coverage={}", format_f64(required_f64(row, "y_coverage")?)),
                    format!(
                        "autosomal_coverage={}",
                        format_f64(required_f64(row, "autosomal_coverage")?)
                    ),
                ]),
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            &path,
        );
    }
    Ok(())
}

fn merge_haplogroups_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.haplogroups/haplogroups.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    let ready_row = required_array(&root, "rows")?
        .iter()
        .find(|row| optional_str(row, "proof_case") == Some("ready"));
    let Some(row) = ready_row else {
        return Ok(());
    };
    insert_evidence(
        rows,
        stage_id,
        required_str(row, "tool_id")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(row, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "haplogroup smoke report preserved governed call and marker support surfaces"
                    .to_string(),
            ),
            authenticity_metric_value: Some(required_f64(row, "confidence")?),
            authenticity_metric_basis: Some("confidence".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("haplogroup_call={}", optional_str(row, "haplogroup_call").unwrap_or("")),
                format!("markers_supported={}", required_u64(row, "markers_supported")?),
                format!("markers_total={}", required_u64(row, "markers_total")?),
                format!("status={}", required_str(row, "status")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_complexity_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.complexity/complexity.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "complexity smoke report preserved governed library complexity summary outputs"
                    .to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!(
                    "estimated_library_size={}",
                    optional_u64(&root, "estimated_library_size")
                        .map_or_else(String::new, |value| value.to_string())
                ),
                format!(
                    "saturation_estimate={}",
                    optional_str(&root, "saturation_estimate").unwrap_or("")
                ),
                format!("observed_total_reads={}", required_u64(&root, "observed_total_reads")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_endogenous_content_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path =
        repo_root.join("runs/bench/local-smoke/bam.endogenous_content/endogenous_content.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "endogenous content report preserved governed endogenous fraction outputs"
                    .to_string(),
            ),
            retained_reads: Some(required_u64(&root, "endogenous_reads")?),
            coverage_metric_value: Some(required_f64(&root, "endogenous_fraction")?),
            coverage_metric_basis: Some("endogenous_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!(
                    "endogenous_fraction={}",
                    format_f64(required_f64(&root, "endogenous_fraction")?)
                ),
                format!("mapped_reads={}", required_u64(&root, "mapped_reads")?),
                format!("total_reads={}", required_u64(&root, "total_reads")?),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_insert_size_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.insert_size/insert_size.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "insert size report preserved governed distribution summary outputs".to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("read_pairs={}", required_u64(&root, "read_pairs")?),
                format!(
                    "mean_insert_size={}",
                    format_f64(required_f64(&root, "mean_insert_size")?)
                ),
                format!(
                    "median_insert_size={}",
                    format_f64(required_f64(&root, "median_insert_size")?)
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_kinship_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.kinship/kinship.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let stage_id = required_str(&root, "stage_id")?;
    let case = required_array(&root, "cases")?.iter().find(|case| {
        optional_str(case, "method") == Some("king") && optional_str(case, "status") == Some("ok")
    });
    let Some(case) = case else {
        return Ok(());
    };
    let concordance = case
        .get("pairwise_results")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(|row| row.get("concordance"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    insert_evidence(
        rows,
        stage_id,
        "king",
        BamEvidenceAggregate {
            truth_correctness_score: Some(concordance),
            truth_correctness_basis: Some("max_pairwise_concordance".to_string()),
            contract_correctness_score: Some(if required_bool(case, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "kinship smoke report preserved governed pairwise relationship outputs".to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("pair_count={}", required_u64(case, "pair_count")?),
                format!(
                    "observed_max_overlap_snps={}",
                    required_u64(case, "observed_max_overlap_snps")?
                ),
                format!("status={}", required_str(case, "status")?),
                format!("concordance={}", format_f64(concordance)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_overlap_correction_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path =
        repo_root.join("runs/bench/local-smoke/bam.overlap_correction/overlap_correction.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "method")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if required_bool(&root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "overlap correction report preserved governed corrected pair outputs".to_string(),
            ),
            retained_reads: Some(required_u64(&root, "pair_count")?),
            scientific_metric_summary: summarize_metrics(vec![
                format!("pair_count={}", required_u64(&root, "pair_count")?),
                format!("corrected_pairs={}", required_u64(&root, "corrected_pairs")?),
                format!(
                    "corrected_overlap_bases={}",
                    required_u64(&root, "corrected_overlap_bases")?
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_recalibration_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.recalibration/recalibration.json");
    let Some(root) = load_optional_json_value(&path)? else {
        return Ok(());
    };
    let contract_ok = required_bool(&root, "expectation_matched")?
        && required_bool(&root, "output_bam_present")?
        && required_bool(&root, "recalibration_report_present")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        BamEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "recalibration smoke report preserved governed recalibrated BAM and report outputs"
                    .to_string(),
            ),
            coverage_metric_value: Some(required_f64(&root, "observed_mean_coverage")?),
            coverage_metric_basis: Some("observed_mean_coverage".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("status={}", required_str(&root, "status")?),
                format!(
                    "observed_mean_coverage={}",
                    format_f64(required_f64(&root, "observed_mean_coverage")?)
                ),
                format!(
                    "observed_breadth_1x={}",
                    format_f64(required_f64(&root, "observed_breadth_1x")?)
                ),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_gc_bias_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/bam.gc_bias/gc_bias.tsv");
    let tsv_rows = read_tsv_rows(&path)?;
    if tsv_rows.is_empty() {
        return Ok(());
    }
    let mut matched = true;
    let mut deviation_sum = 0.0;
    let mut count = 0_u64;
    for row in &tsv_rows {
        matched &= parse_bool(required_tsv(row, "case_expectation_matched")?)?;
        let normalized_coverage = parse_f64(required_tsv(row, "normalized_coverage")?)?;
        deviation_sum += (normalized_coverage - 1.0).abs();
        count += 1;
    }
    let mean_deviation = deviation_sum / checked_f64_from_u64(count, "GC bias row count")?;
    insert_evidence(
        rows,
        "bam.gc_bias",
        "picard",
        BamEvidenceAggregate {
            contract_correctness_score: Some(if matched { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "GC bias TSV preserved governed normalized coverage and window surfaces"
                    .to_string(),
            ),
            scientific_metric_summary: summarize_metrics(vec![
                format!("mean_abs_normalized_coverage_deviation={}", format_f64(mean_deviation)),
                format!("gc_bin_count={count}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn insert_evidence(
    rows: &mut BTreeMap<(String, String), BamEvidenceAggregate>,
    stage_id: &str,
    tool_id: &str,
    evidence: BamEvidenceAggregate,
    source_path: &Path,
) {
    let entry = rows.entry((stage_id.to_string(), tool_id.to_string())).or_default();
    entry.source_paths.insert(source_path.display().to_string());
    merge_option(&mut entry.truth_correctness_score, evidence.truth_correctness_score);
    merge_string(&mut entry.truth_correctness_basis, evidence.truth_correctness_basis);
    merge_option(&mut entry.contract_correctness_score, evidence.contract_correctness_score);
    merge_string(&mut entry.contract_correctness_basis, evidence.contract_correctness_basis);
    merge_option(&mut entry.retained_reads, evidence.retained_reads);
    merge_option(&mut entry.dropped_reads, evidence.dropped_reads);
    merge_option(&mut entry.alignment_qc_metric_value, evidence.alignment_qc_metric_value);
    merge_string(&mut entry.alignment_qc_metric_basis, evidence.alignment_qc_metric_basis);
    merge_option(&mut entry.coverage_metric_value, evidence.coverage_metric_value);
    merge_string(&mut entry.coverage_metric_basis, evidence.coverage_metric_basis);
    merge_option(&mut entry.damage_metric_value, evidence.damage_metric_value);
    merge_string(&mut entry.damage_metric_basis, evidence.damage_metric_basis);
    merge_option(&mut entry.authenticity_metric_value, evidence.authenticity_metric_value);
    merge_string(&mut entry.authenticity_metric_basis, evidence.authenticity_metric_basis);
    merge_string(&mut entry.scientific_metric_summary, evidence.scientific_metric_summary);
}

fn merge_option<T: Copy>(target: &mut Option<T>, incoming: Option<T>) {
    if target.is_none() {
        *target = incoming;
    }
}

fn merge_string(target: &mut Option<String>, incoming: Option<String>) {
    if target.is_none() {
        *target = incoming;
    }
}

fn build_runtime_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut grouped = BTreeMap::<String, Vec<((String, String), f64)>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        if let Some(runtime_seconds) = row.runtime_seconds {
            grouped
                .entry(row.stage_id.clone())
                .or_default()
                .push(((row.stage_id.clone(), row.tool_id.clone()), runtime_seconds));
        }
    }
    let mut scores = BTreeMap::new();
    for values in grouped.into_values() {
        if values.len() == 1 {
            if let Some((key, _)) = values.first() {
                scores.insert(key.clone(), 1.0);
            }
            continue;
        }
        if values.len() < 2 {
            continue;
        }
        let min = values.iter().map(|(_, value)| *value).fold(f64::INFINITY, f64::min);
        let max = values.iter().map(|(_, value)| *value).fold(f64::NEG_INFINITY, f64::max);
        for (key, value) in values {
            scores.insert(key, normalize_lower_is_better(value, min, max));
        }
    }
    scores
}

fn build_memory_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut grouped = BTreeMap::<String, Vec<((String, String), f64)>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        if let Some(memory_mb) = row.effective_memory_mb {
            grouped
                .entry(row.stage_id.clone())
                .or_default()
                .push(((row.stage_id.clone(), row.tool_id.clone()), memory_mb));
        }
    }
    let mut scores = BTreeMap::new();
    for values in grouped.into_values() {
        if values.len() == 1 {
            if let Some((key, _)) = values.first() {
                scores.insert(key.clone(), 1.0);
            }
            continue;
        }
        if values.len() < 2 {
            continue;
        }
        let min = values.iter().map(|(_, value)| *value).fold(f64::INFINITY, f64::min);
        let max = values.iter().map(|(_, value)| *value).fold(f64::NEG_INFINITY, f64::max);
        for (key, value) in values {
            scores.insert(key, normalize_lower_is_better(value, min, max));
        }
    }
    scores
}

fn completion_score_for_row(row: &BaseScoreRow) -> Option<f64> {
    if row.failure_class != "none" {
        return None;
    }
    if row.row_statuses.iter().all(|status| status == "present") {
        return Some(1.0);
    }
    None
}

fn score_row(
    row: &BaseScoreRow,
    correctness_score: Option<f64>,
    scientific_threshold_score: Option<f64>,
    runtime_score: Option<f64>,
    memory_score: Option<f64>,
    completion_score: Option<f64>,
    failure_class_score: Option<f64>,
) -> (BamToolScoreStatus, f64, Option<f64>) {
    if row.failure_class != "none" {
        return match row.failure_class.as_str() {
            "insufficient_data" => (BamToolScoreStatus::InsufficientEvidence, 0.0, None),
            _ => (BamToolScoreStatus::Blocked, 0.0, None),
        };
    }
    let components = [
        (correctness_score, row.weights.correctness),
        (scientific_threshold_score, row.weights.scientific_threshold),
        (runtime_score, row.weights.runtime),
        (memory_score, row.weights.memory),
        (completion_score, row.weights.completion),
        (failure_class_score, row.weights.failure_class),
    ];
    let mut weighted_sum = 0.0;
    let mut covered_weight = 0.0;
    for (value, weight) in components {
        if weight <= 0.0 {
            continue;
        }
        if let Some(value) = value {
            weighted_sum += value * weight;
            covered_weight += weight;
        }
    }
    if covered_weight <= 0.0 || correctness_score.is_none() {
        return (BamToolScoreStatus::InsufficientEvidence, 0.0, None);
    }
    (BamToolScoreStatus::Scored, covered_weight, Some(weighted_sum / covered_weight))
}

fn classify_failure_class(
    binding_rows: &[FullBenchmarkReportRowView],
    evidence: Option<&BamEvidenceAggregate>,
    runtime_row: Option<&MicroBenchmarkRuntimeRowView>,
) -> String {
    if binding_rows.is_empty() {
        return "missing_output".to_string();
    }
    if binding_rows.iter().any(|row| normalized_row_status_for_scoring(row) == "unsupported_pair") {
        return "unsupported_pair".to_string();
    }
    if binding_rows.iter().any(|row| normalized_row_status_for_scoring(row) == "missing_result") {
        return "missing_output".to_string();
    }
    if evidence.is_none() {
        if let Some(runtime_row) = runtime_row {
            match runtime_row.execution_status.as_str() {
                "failed" => return "command_failed".to_string(),
                "unavailable" => return "tool_not_found".to_string(),
                _ => {}
            }
        }
        return "insufficient_data".to_string();
    }
    "none".to_string()
}

fn normalized_row_status_for_scoring(row: &FullBenchmarkReportRowView) -> String {
    if row.row_status == "missing_result" && is_missing_result_probe_row(row) {
        return "present".to_string();
    }
    row.row_status.clone()
}

fn is_missing_result_probe_row(row: &FullBenchmarkReportRowView) -> bool {
    row.evidence_path.as_deref().is_some_and(|path| path.contains("/missing-result-test/"))
        || row.detail.as_deref().is_some_and(|detail| detail.contains("fake-run manifest"))
}

fn describe_row_reason(
    failure_class: &str,
    evidence: Option<&BamEvidenceAggregate>,
    runtime_row: Option<&MicroBenchmarkRuntimeRowView>,
    memory_row: Option<&MicroBenchmarkMemoryRowView>,
    stage_row: &super::stage_scoring::StageScoringRow,
    tool_id: &str,
    failure_catalog: &BTreeMap<String, String>,
) -> String {
    if failure_class != "none" {
        let detail = failure_catalog
            .get(failure_class)
            .cloned()
            .unwrap_or_else(|| "unknown failure classification".to_string());
        if failure_class == "insufficient_data" && evidence.is_none() {
            return format!(
                "{detail}; no real BAM smoke or micro evidence row was found for `{}` / `{}`",
                stage_row.stage_id, tool_id
            );
        }
        return detail;
    }
    let evidence_count = evidence.map_or(0, |row| row.source_paths.len());
    match (runtime_row, memory_row) {
        (Some(runtime_row), Some(memory_row)) => format!(
            "scored from {evidence_count} real evidence surface(s) with micro execution status `{}` and memory source `{}`",
            runtime_row.execution_status, memory_row.memory_source
        ),
        (Some(runtime_row), None) => format!(
            "scored from {evidence_count} real evidence surface(s); micro runtime status is `{}` but no memory row matched this binding",
            runtime_row.execution_status
        ),
        _ => format!(
            "scored from {evidence_count} real evidence surface(s); runtime and memory ranking remain unavailable for this binding"
        ),
    }
}

fn select_runtime_row(
    stage_row: &super::stage_scoring::StageScoringRow,
    tool_id: &str,
    runtime_rows: &BTreeMap<(String, String), Vec<MicroBenchmarkRuntimeRowView>>,
    has_evidence: bool,
) -> Option<MicroBenchmarkRuntimeRowView> {
    if let Some(rows) = runtime_rows.get(&(stage_row.stage_id.clone(), tool_id.to_string())) {
        return best_runtime_row(rows);
    }
    if tool_id != stage_row.default_tool_id || !has_evidence {
        return None;
    }
    let stage_candidates = runtime_rows
        .iter()
        .filter(|((stage_id, _), _)| stage_id == &stage_row.stage_id)
        .flat_map(|(_, rows)| rows.iter().cloned())
        .collect::<Vec<_>>();
    if stage_candidates.len() == 1 {
        return stage_candidates.into_iter().next();
    }
    if stage_candidates
        .iter()
        .all(|row| !stage_row.benchmark_ready_tool_ids.iter().any(|tool| tool == &row.tool_id))
    {
        return best_runtime_row(&stage_candidates);
    }
    None
}

fn select_memory_row(
    stage_row: &super::stage_scoring::StageScoringRow,
    tool_id: &str,
    memory_rows: &BTreeMap<(String, String), Vec<MicroBenchmarkMemoryRowView>>,
    has_evidence: bool,
) -> Option<MicroBenchmarkMemoryRowView> {
    if let Some(rows) = memory_rows.get(&(stage_row.stage_id.clone(), tool_id.to_string())) {
        return best_memory_row(rows);
    }
    if tool_id != stage_row.default_tool_id || !has_evidence {
        return None;
    }
    let stage_candidates = memory_rows
        .iter()
        .filter(|((stage_id, _), _)| stage_id == &stage_row.stage_id)
        .flat_map(|(_, rows)| rows.iter().cloned())
        .collect::<Vec<_>>();
    if stage_candidates.len() == 1 {
        return stage_candidates.into_iter().next();
    }
    if stage_candidates
        .iter()
        .all(|row| !stage_row.benchmark_ready_tool_ids.iter().any(|tool| tool == &row.tool_id))
    {
        return best_memory_row(&stage_candidates);
    }
    None
}

fn best_runtime_row(rows: &[MicroBenchmarkRuntimeRowView]) -> Option<MicroBenchmarkRuntimeRowView> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.elapsed_seconds, left.elapsed_seconds))
    });
    candidates.into_iter().next()
}

fn best_memory_row(rows: &[MicroBenchmarkMemoryRowView]) -> Option<MicroBenchmarkMemoryRowView> {
    let mut candidates = rows.to_vec();
    candidates.sort_by(|left, right| {
        execution_status_rank(&left.execution_status)
            .cmp(&execution_status_rank(&right.execution_status))
            .then_with(|| option_f64_cmp(right.observed_memory_mb, left.observed_memory_mb))
            .then_with(|| option_f64_cmp(right.declared_memory_mb, left.declared_memory_mb))
    });
    candidates.into_iter().next()
}

fn build_report(
    output_path: String,
    config_path: String,
    rows: Vec<BamToolScoreRow>,
) -> BamToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == BamToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == BamToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == BamToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    BamToolScoresReport {
        schema_version: BAM_TOOL_SCORES_SCHEMA_VERSION,
        output_path,
        config_path,
        row_count: rows.len(),
        stage_count,
        tool_count,
        scored_row_count,
        insufficient_evidence_row_count,
        blocked_row_count,
        failure_class_counts,
        rows,
    }
}

fn render_bam_tool_scores_tsv(rows: &[BamToolScoreRow]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tretained_reads\tdropped_reads\talignment_qc_metric_value\talignment_qc_metric_basis\tcoverage_metric_value\tcoverage_metric_basis\tdamage_metric_value\tdamage_metric_basis\tauthenticity_metric_value\tauthenticity_metric_basis\tscientific_metric_ids\tscientific_metric_summary\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason".to_string(),
    );
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.stage_id,
            row.tool_id,
            row.decision_mode,
            row.correctness_signal,
            join_csv(&row.result_ids),
            join_csv(&row.report_row_ids),
            join_csv(&row.corpus_ids),
            join_csv(&row.report_sections),
            join_csv(&row.row_statuses),
            score_status_label(row.score_status),
            format_optional_f64(row.truth_correctness_score),
            row.truth_correctness_basis.clone().unwrap_or_default(),
            format_optional_f64(row.contract_correctness_score),
            row.contract_correctness_basis.clone().unwrap_or_default(),
            format_optional_u64(row.retained_reads),
            format_optional_u64(row.dropped_reads),
            format_optional_f64(row.alignment_qc_metric_value),
            row.alignment_qc_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.coverage_metric_value),
            row.coverage_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.damage_metric_value),
            row.damage_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.authenticity_metric_value),
            row.authenticity_metric_basis.clone().unwrap_or_default(),
            join_csv(&row.scientific_metric_ids),
            sanitize_tsv_cell(row.scientific_metric_summary.as_deref().unwrap_or("")),
            format_optional_f64(row.runtime_seconds),
            row.runtime_source,
            format_optional_f64(row.observed_memory_mb),
            format_optional_f64(row.declared_memory_mb),
            row.memory_source,
            row.failure_class,
            row.micro_execution_status.clone().unwrap_or_default(),
            format_f64(row.score_weight_coverage),
            format_optional_f64(row.score_total),
            join_csv(&row.evidence_paths),
            sanitize_tsv_cell(&row.reason),
        ));
    }
    lines.join("\n") + "\n"
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    repo_relative_path(repo_root, path)
        .display()
        .to_string()
        .replace(&format!("{}/", repo_root.display()), "")
}

fn load_optional_json_value(path: &Path) -> Result<Option<Value>> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value =
        serde_json::from_str::<Value>(&raw).with_context(|| format!("parse {}", path.display()))?;
    if matches!(value, Value::Object(ref map) if map.is_empty()) {
        return Ok(None);
    }
    Ok(Some(value))
}

fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| anyhow!("missing array field `{key}`"))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| anyhow!("missing string field `{key}`"))
}

fn optional_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn required_u64(value: &Value, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).ok_or_else(|| anyhow!("missing integer field `{key}`"))
}

fn optional_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn required_f64(value: &Value, key: &str) -> Result<f64> {
    value.get(key).and_then(Value::as_f64).ok_or_else(|| anyhow!("missing numeric field `{key}`"))
}

fn required_bool(value: &Value, key: &str) -> Result<bool> {
    value.get(key).and_then(Value::as_bool).ok_or_else(|| anyhow!("missing boolean field `{key}`"))
}

fn read_tsv_rows(path: &Path) -> Result<Vec<BTreeMap<String, String>>> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut lines = raw.lines();
    let headers = lines
        .next()
        .ok_or_else(|| anyhow!("{} is missing a header row", path.display()))?
        .split('\t')
        .map(str::to_string)
        .collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let values = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        let row = headers
            .iter()
            .cloned()
            .zip(values.into_iter().chain(std::iter::repeat(String::new())))
            .take(headers.len())
            .collect::<BTreeMap<_, _>>();
        rows.push(row);
    }
    Ok(rows)
}

fn required_tsv<'a>(row: &'a BTreeMap<String, String>, key: &str) -> Result<&'a str> {
    row.get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("missing TSV field `{key}`"))
}

fn parse_u64(value: &str) -> Result<u64> {
    value.parse::<u64>().with_context(|| format!("parse integer `{value}`"))
}

fn parse_f64(value: &str) -> Result<f64> {
    value.parse::<f64>().with_context(|| format!("parse number `{value}`"))
}

fn parse_bool(value: &str) -> Result<bool> {
    value.parse::<bool>().with_context(|| format!("parse bool `{value}`"))
}

fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        Ok(0.0)
    } else {
        Ok(checked_f64_from_u64(numerator, "fraction numerator")?
            / checked_f64_from_u64(denominator, "fraction denominator")?)
    }
}

fn normalize_lower_is_better(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() <= 1e-12 {
        1.0
    } else {
        ((max - value) / (max - min)).clamp(0.0, 1.0)
    }
}

fn summarize_metrics(values: Vec<String>) -> Option<String> {
    let values = values
        .into_iter()
        .map(|value| sanitize_tsv_cell(&value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        None
    } else {
        Some(values.join("; "))
    }
}

fn join_csv(values: &[String]) -> String {
    values.iter().map(|value| sanitize_tsv_cell(value)).collect::<Vec<_>>().join(",")
}

fn unique_strings<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    for value in values {
        if !value.is_empty() && seen.insert(value.clone()) {
            rows.push(value);
        }
    }
    rows
}

fn sanitize_tsv_cell(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map_or_else(String::new, format_f64)
}

fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

fn format_optional_u64(value: Option<u64>) -> String {
    value.map_or_else(String::new, |value| value.to_string())
}

fn decision_mode_label(value: StageScoringDecisionMode) -> &'static str {
    match value {
        StageScoringDecisionMode::MultiToolRanking => "multi_tool_ranking",
        StageScoringDecisionMode::SingleToolAcceptance => "single_tool_acceptance",
    }
}

fn correctness_signal_label(value: StageScoringCorrectnessSignal) -> &'static str {
    match value {
        StageScoringCorrectnessSignal::ScientificComparableMetrics => {
            "scientific_comparable_metrics"
        }
        StageScoringCorrectnessSignal::OutputContract => "output_contract",
    }
}

fn score_status_label(value: BamToolScoreStatus) -> &'static str {
    match value {
        BamToolScoreStatus::Scored => "scored",
        BamToolScoreStatus::InsufficientEvidence => "insufficient_evidence",
        BamToolScoreStatus::Blocked => "blocked",
    }
}

fn execution_status_rank(value: &str) -> usize {
    match value {
        "succeeded" => 0,
        "container_needed" => 1,
        "unavailable" => 2,
        "failed" => 3,
        _ => 4,
    }
}

fn option_f64_cmp(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Greater,
        (None, Some(_)) => Ordering::Less,
        (None, None) => Ordering::Equal,
    }
}
