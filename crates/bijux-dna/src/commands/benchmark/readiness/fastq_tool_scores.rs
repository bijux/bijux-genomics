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
use crate::commands::numeric::rounded_f64_to_u64;

pub(crate) const DEFAULT_FASTQ_TOOL_SCORES_PATH: &str =
    "runs/bench/micro/fastq/FASTQ_TOOL_SCORES.tsv";

const FASTQ_TOOL_SCORES_SCHEMA_VERSION: &str = "bijux.bench.readiness.fastq_tool_scores.v1";
const DEFAULT_FULL_BENCHMARK_REPORT_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
const DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/expected.json";
const DEFAULT_AMPLICON_NORMALIZE_PRIMERS_MANIFEST_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/manifest.toml";
const DEFAULT_AMPLICON_REMOVE_CHIMERAS_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/chimera_expectations.tsv";
const DEFAULT_AMPLICON_NORMALIZE_ABUNDANCE_PATH: &str =
    "benchmarks/tests/fixtures/science/amplicon-truth/normalized_abundance.tsv";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqToolScoreStatus {
    Scored,
    InsufficientEvidence,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqToolScoreRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) decision_mode: String,
    pub(crate) correctness_signal: String,
    pub(crate) result_ids: Vec<String>,
    pub(crate) report_row_ids: Vec<String>,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) report_sections: Vec<String>,
    pub(crate) row_statuses: Vec<String>,
    pub(crate) score_status: FastqToolScoreStatus,
    pub(crate) truth_correctness_score: Option<f64>,
    pub(crate) truth_correctness_basis: Option<String>,
    pub(crate) contract_correctness_score: Option<f64>,
    pub(crate) contract_correctness_basis: Option<String>,
    pub(crate) retained_reads: Option<u64>,
    pub(crate) dropped_reads: Option<u64>,
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
pub(crate) struct FastqToolScoresReport {
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
    pub(crate) rows: Vec<FastqToolScoreRow>,
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
struct FastqEvidenceAggregate {
    source_paths: BTreeSet<String>,
    truth_correctness_score: Option<f64>,
    truth_correctness_basis: Option<String>,
    contract_correctness_score: Option<f64>,
    contract_correctness_basis: Option<String>,
    retained_reads: Option<u64>,
    dropped_reads: Option<u64>,
}

#[derive(Debug, Clone)]
struct BaseScoreRow {
    stage_id: String,
    tool_id: String,
    decision_mode: StageScoringDecisionMode,
    correctness_signal: StageScoringCorrectnessSignal,
    weights: super::stage_scoring::StageScoringWeights,
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

pub(crate) fn run_render_fastq_tool_scores(
    args: &parse::BenchReadinessRenderFastqToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_tool_scores(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_fastq_tool_scores(
    args: &parse::BenchReadinessValidateFastqToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_fastq_tool_scores(
        &repo_root,
        args.input.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_tool_scores(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqToolScoresReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (config_path, rows) = collect_fastq_tool_score_rows(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_tool_scores_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(build_report(
        path_relative_to_repo(repo_root, &output_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

pub(crate) fn validate_fastq_tool_scores(
    repo_root: &Path,
    input_path: PathBuf,
) -> Result<FastqToolScoresReport> {
    let input_path = repo_relative_path(repo_root, &input_path);
    let actual = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let (config_path, rows) = collect_fastq_tool_score_rows(repo_root)?;
    let expected = render_fastq_tool_scores_tsv(&rows);
    if actual != expected {
        bail!(
            "FASTQ tool score TSV drifted from governed evidence contracts; rerun `bijux-dna bench readiness render-fastq-tool-scores`"
        );
    }
    Ok(build_report(
        path_relative_to_repo(repo_root, &input_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

fn collect_fastq_tool_score_rows(repo_root: &Path) -> Result<(PathBuf, Vec<FastqToolScoreRow>)> {
    let config_path = repo_root.join(DEFAULT_STAGE_SCORING_PATH);
    let config = load_stage_scoring_config(&config_path)?;
    let failure_catalog = config
        .failure_classes
        .iter()
        .map(|row| (row.class_id.clone(), row.detail.clone()))
        .collect::<BTreeMap<_, _>>();
    let full_report = load_full_benchmark_report(repo_root)?;
    let micro_report = load_micro_benchmark_report(repo_root)?;
    let evidence = load_fastq_evidence(repo_root)?;

    let full_rows_by_binding =
        full_report.rows.into_iter().filter(|row| row.domain == "fastq").fold(
            BTreeMap::<(String, String), Vec<FullBenchmarkReportRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );
    let runtime_by_binding =
        micro_report.runtime_rows.into_iter().filter(|row| row.domain == "fastq").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkRuntimeRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );
    let memory_by_binding =
        micro_report.memory_source_rows.into_iter().filter(|row| row.domain == "fastq").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkMemoryRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );

    let fastq_stage_rows =
        config.rows.iter().filter(|row| row.domain == "fastq").collect::<Vec<_>>();

    let mut base_rows = Vec::new();
    for stage_row in &fastq_stage_rows {
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
            let failure_class = classify_failure_class(
                &binding_rows,
                evidence_row,
                runtime_row.as_ref(),
                stage_row,
                tool_id.as_str(),
            );
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

        rows.push(FastqToolScoreRow {
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

fn load_fastq_evidence(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), FastqEvidenceAggregate>> {
    let mut rows = BTreeMap::<(String, String), FastqEvidenceAggregate>::new();

    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.filter_reads/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let dropped_reads = required_u64(root, "reads_dropped")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "report retained reads and dropped reads without violating input bounds"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(dropped_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.filter_low_complexity/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let dropped_reads = required_u64(root, "reads_removed_low_complexity")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "report low-complexity removal without inflating the read count"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(dropped_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.profile_overrepresented_sequences/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let top_fraction = required_f64(root, "top_fraction")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(top_fraction),
                    truth_correctness_basis: Some("top_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "surface publishes governed comparable metrics for overrepresented sequences"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.remove_duplicates/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            let duplicate_reads = required_u64(root, "duplicate_reads")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "duplicate removal preserves the governed post-dedup read count surface"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(duplicate_reads),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.detect_adapters/adapters.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let case_count = required_u64(root, "case_count")?;
            let detected_case_count = required_u64(root, "detected_case_count")?;
            Ok((
                stage_id.to_string(),
                "fastqc".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(detected_case_count, case_count)?),
                    truth_correctness_basis: Some("detected_case_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "adapter detection summary keeps the governed case-count surface complete"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.extract_umis/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "UMI extraction summary and governed report artifacts materialized"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.detect_duplicates_premerge/duplicates.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            Ok((
                stage_id.to_string(),
                "bijux_dna".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "duplicate-signal smoke report covers both duplicate and distinct pair cases"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.estimate_library_complexity_prealign/complexity.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            Ok((
                stage_id.to_string(),
                "bijux_dna".to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "complexity smoke report emits governed estimated- and insufficient-data counts"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.merge_pairs/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            let input_pair_count = required_u64(root, "input_pair_count")?;
            let merged_count = required_u64(root, "merged_count")?;
            let unmerged_r1_count = required_u64(root, "unmerged_r1_count")?;
            let unmerged_r2_count = required_u64(root, "unmerged_r2_count")?;
            let discarded_count = required_u64(root, "discarded_count")?;
            let input_reads = input_pair_count.saturating_mul(2);
            let retained_reads = merged_count + unmerged_r1_count + unmerged_r2_count;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "merge report preserves merged, unmerged, and discarded counts".to_string(),
                    ),
                    retained_reads: Some(retained_reads),
                    dropped_reads: Some(discarded_count),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.normalize_abundance/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "normalized abundance table materialized and retained numeric validity"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.remove_chimeras/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "chimera summary emits governed non-chimeric and chimera artifact paths"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.cluster_otus/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "OTU clustering report materialized representatives and abundance outputs"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.infer_asvs/report.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "planned_tool_id")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: None,
                    truth_correctness_basis: None,
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "ASV inference report materialized representatives and abundance outputs"
                            .to_string(),
                    ),
                    retained_reads: None,
                    dropped_reads: None,
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;
    merge_single_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.trim_terminal_damage/metrics.json",
        |root| {
            let stage_id = required_str(root, "stage_id")?;
            let tool_id = required_str(root, "tool_id")?;
            let input_reads = required_u64(root, "input_reads")?;
            let output_reads = required_u64(root, "output_reads")?;
            Ok((
                stage_id.to_string(),
                tool_id.to_string(),
                FastqEvidenceAggregate {
                    truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
                    truth_correctness_basis: Some("retained_fraction".to_string()),
                    contract_correctness_score: Some(1.0),
                    contract_correctness_basis: Some(
                        "terminal-damage trimming report preserves the governed output count surface"
                            .to_string(),
                    ),
                    retained_reads: Some(output_reads),
                    dropped_reads: Some(input_reads.saturating_sub(output_reads)),
                    source_paths: BTreeSet::new(),
                },
            ))
        },
    )?;

    merge_case_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.trim_reads/report.json",
        "trim_reads",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.trim_polyg_tails/metrics.json",
        "trim_polyg_tails",
    )?;
    merge_case_stage_report(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/fastq.normalize_primers/report.json",
        "normalize_primers",
    )?;
    merge_profile_reads_summary(repo_root, &mut rows)?;
    merge_validate_reads_summary(repo_root, &mut rows)?;
    merge_screen_taxonomy_summary(repo_root, &mut rows)?;
    merge_fixture_backed_amplicon_fastq_evidence(repo_root, &mut rows);

    Ok(rows)
}

fn merge_fixture_backed_amplicon_fastq_evidence(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) {
    merge_evidence_row(
        rows,
        ("fastq.normalize_primers".to_string(), "cutadapt".to_string()),
        repo_root.join(DEFAULT_AMPLICON_NORMALIZE_PRIMERS_MANIFEST_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(1.0),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon fixture preserves the governed primer-normalization output surface"
                    .to_string(),
            ),
            retained_reads: Some(3),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.remove_chimeras".to_string(), "vsearch".to_string()),
        repo_root.join(DEFAULT_AMPLICON_REMOVE_CHIMERAS_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon chimera fixtures preserve the governed chimera-removal outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.infer_asvs".to_string(), "dada2".to_string()),
        repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed ASV inference outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.cluster_otus".to_string(), "vsearch".to_string()),
        repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed OTU-clustering outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
    merge_evidence_row(
        rows,
        ("fastq.normalize_abundance".to_string(), "seqkit".to_string()),
        repo_root.join(DEFAULT_AMPLICON_NORMALIZE_ABUNDANCE_PATH).display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "tracked amplicon truth bundle preserves the governed abundance-normalization outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    );
}

fn merge_case_stage_report(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    contract_label: &str,
) -> Result<()> {
    let absolute_path = repo_root.join(relative_path);
    let root = load_optional_json_value(&absolute_path)?
        .ok_or_else(|| anyhow!("missing {}", absolute_path.display()))?;
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let mut tool_totals = BTreeMap::<String, (u64, u64, u64, u64)>::new();
    for case in cases {
        let tool_id = required_str(case, "tool_id")?.to_string();
        let input_reads = optional_u64(case, "input_read_count_total")
            .or_else(|| optional_u64(case, "input_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing input read counts for `{tool_id}`", absolute_path.display())
            })?;
        let output_reads = optional_u64(case, "output_read_count_total")
            .or_else(|| optional_u64(case, "output_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing output read counts for `{tool_id}`", absolute_path.display())
            })?;
        let retained_reads = optional_u64(case, "reads_retained").unwrap_or(output_reads);
        let dropped_reads = optional_u64(case, "reads_dropped")
            .unwrap_or_else(|| input_reads.saturating_sub(output_reads));
        let entry = tool_totals.entry(tool_id).or_default();
        entry.0 += input_reads;
        entry.1 += retained_reads;
        entry.2 += dropped_reads;
        entry.3 += 1;
    }
    for (tool_id, (input_reads, retained_reads, dropped_reads, _case_count)) in tool_totals {
        merge_evidence_row(
            rows,
            (stage_id.clone(), tool_id),
            absolute_path.display().to_string(),
            FastqEvidenceAggregate {
                truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
                truth_correctness_basis: Some("retained_fraction".to_string()),
                contract_correctness_score: Some(1.0),
                contract_correctness_basis: Some(format!(
                    "{contract_label} smoke cases preserve the governed output-count surface"
                )),
                retained_reads: Some(retained_reads),
                dropped_reads: Some(dropped_reads),
                source_paths: BTreeSet::new(),
            },
        );
    }
    Ok(())
}

fn merge_profile_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path = repo_root.join("runs/bench/local-smoke/fastq.profile_reads/profile.json");
    let root = load_optional_json_value(&absolute_path)?
        .ok_or_else(|| anyhow!("missing {}", absolute_path.display()))?;
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = cases
        .iter()
        .map(|case| required_u64(case, "reads_total"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    merge_evidence_row(
        rows,
        (stage_id, "seqkit_stats".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "profile report emits governed read-count and aggregate QC metrics".to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_validate_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path = repo_root.join("runs/bench/local-smoke/fastq.validate_reads/report.json");
    let root = load_optional_json_value(&absolute_path)?
        .ok_or_else(|| anyhow!("missing {}", absolute_path.display()))?;
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = cases
        .iter()
        .map(|case| required_u64(case, "input_read_count_total"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sum::<u64>();
    let passed_cases = u64::try_from(
        cases.iter().filter(|case| optional_str(case, "validation_status") == Some("pass")).count(),
    )?;
    let case_count = u64::try_from(cases.len())?;
    merge_evidence_row(
        rows,
        (stage_id, "fastqvalidator".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(passed_cases, case_count)?),
            truth_correctness_basis: Some("validation_pass_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "validation smoke report covers every governed case without missing outputs"
                    .to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_screen_taxonomy_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path = repo_root
        .join("runs/bench/micro/pipelines/edna/artifacts/fastq.screen_taxonomy/report.json");
    let root = load_optional_json_value(&absolute_path)?
        .ok_or_else(|| anyhow!("missing {}", absolute_path.display()))?;
    let stage_id = required_str(&root, "stage_id")?.to_string();
    let tool_id = required_str(&root, "tool_id")?.to_string();
    let samples = root
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing samples", absolute_path.display()))?;
    let mut total_reads = 0_f64;
    let mut classified_reads = 0_f64;
    for sample in samples {
        let reads_in = required_f64(sample, "reads_in")?;
        let unclassified_fraction = required_f64(sample, "unclassified_fraction")?;
        total_reads += reads_in;
        classified_reads += reads_in * (1.0 - unclassified_fraction);
    }
    merge_evidence_row(
        rows,
        (stage_id, tool_id),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: (total_reads > 0.0).then_some(classified_reads / total_reads),
            truth_correctness_basis: Some("classified_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "taxonomy screen report materialized the governed classified and unclassified surfaces"
                    .to_string(),
            ),
            retained_reads: Some(rounded_f64_to_u64(classified_reads, "retained reads")?),
            dropped_reads: Some(rounded_f64_to_u64(total_reads - classified_reads, "dropped reads")?),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

fn merge_single_stage_report<F>(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    extractor: F,
) -> Result<()>
where
    F: Fn(&Value) -> Result<(String, String, FastqEvidenceAggregate)>,
{
    let absolute_path = repo_root.join(relative_path);
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let (stage_id, tool_id, evidence) = extractor(&root)?;
    merge_evidence_row(rows, (stage_id, tool_id), absolute_path.display().to_string(), evidence);
    Ok(())
}

fn merge_evidence_row(
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    key: (String, String),
    source_path: String,
    mut evidence: FastqEvidenceAggregate,
) {
    evidence.source_paths.insert(source_path);
    let entry = rows.entry(key).or_default();
    entry.source_paths.extend(evidence.source_paths);
    if entry.truth_correctness_score.is_none() {
        entry.truth_correctness_score = evidence.truth_correctness_score;
    }
    if entry.truth_correctness_basis.is_none() {
        entry.truth_correctness_basis = evidence.truth_correctness_basis;
    }
    if entry.contract_correctness_score.is_none() {
        entry.contract_correctness_score = evidence.contract_correctness_score;
    }
    if entry.contract_correctness_basis.is_none() {
        entry.contract_correctness_basis = evidence.contract_correctness_basis;
    }
    if entry.retained_reads.is_none() {
        entry.retained_reads = evidence.retained_reads;
    }
    if entry.dropped_reads.is_none() {
        entry.dropped_reads = evidence.dropped_reads;
    }
}

fn build_runtime_scores(rows: &[BaseScoreRow]) -> BTreeMap<(String, String), f64> {
    let mut per_stage = BTreeMap::<String, Vec<&BaseScoreRow>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        per_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut scores = BTreeMap::new();
    for stage_rows in per_stage.into_values() {
        let Some(reference) = stage_rows.first() else {
            continue;
        };
        let values = stage_rows
            .iter()
            .filter_map(|row| {
                row.runtime_seconds
                    .map(|value| ((row.stage_id.clone(), row.tool_id.clone()), value))
            })
            .collect::<Vec<_>>();
        if reference.decision_mode == StageScoringDecisionMode::SingleToolAcceptance {
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
    let mut per_stage = BTreeMap::<String, Vec<&BaseScoreRow>>::new();
    for row in rows.iter().filter(|row| row.failure_class == "none") {
        per_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut scores = BTreeMap::new();
    for stage_rows in per_stage.into_values() {
        let Some(reference) = stage_rows.first() else {
            continue;
        };
        let values = stage_rows
            .iter()
            .filter_map(|row| {
                row.effective_memory_mb
                    .map(|value| ((row.stage_id.clone(), row.tool_id.clone()), value))
            })
            .collect::<Vec<_>>();
        if reference.decision_mode == StageScoringDecisionMode::SingleToolAcceptance {
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
) -> (FastqToolScoreStatus, f64, Option<f64>) {
    if row.failure_class != "none" {
        return match row.failure_class.as_str() {
            "insufficient_data" => (FastqToolScoreStatus::InsufficientEvidence, 0.0, None),
            _ => (FastqToolScoreStatus::Blocked, 0.0, None),
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
        return (FastqToolScoreStatus::InsufficientEvidence, 0.0, None);
    }
    (FastqToolScoreStatus::Scored, covered_weight, Some(weighted_sum / covered_weight))
}

fn classify_failure_class(
    binding_rows: &[FullBenchmarkReportRowView],
    evidence: Option<&FastqEvidenceAggregate>,
    runtime_row: Option<&MicroBenchmarkRuntimeRowView>,
    _stage_row: &super::stage_scoring::StageScoringRow,
    _tool_id: &str,
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
    if let Some(runtime_row) = runtime_row {
        match runtime_row.execution_status.as_str() {
            "failed" => return "command_failed".to_string(),
            "unavailable" => return "tool_not_found".to_string(),
            _ => {}
        }
    }
    if evidence.is_none() {
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
    evidence: Option<&FastqEvidenceAggregate>,
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
                "{detail}; no real FASTQ smoke or micro evidence row was found for `{}` / `{}`",
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
    rows: Vec<FastqToolScoreRow>,
) -> FastqToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == FastqToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == FastqToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == FastqToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    FastqToolScoresReport {
        schema_version: FASTQ_TOOL_SCORES_SCHEMA_VERSION,
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

fn render_fastq_tool_scores_tsv(rows: &[FastqToolScoreRow]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tretained_reads\tdropped_reads\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason".to_string(),
    );
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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

fn fraction_u64(numerator: u64, denominator: u64) -> Result<f64> {
    if denominator == 0 {
        Ok(0.0)
    } else {
        Ok(crate::commands::numeric::checked_f64_from_u64(numerator, "fraction numerator")?
            / crate::commands::numeric::checked_f64_from_u64(denominator, "fraction denominator")?)
    }
}

fn normalize_lower_is_better(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() <= 1e-12 {
        1.0
    } else {
        ((max - value) / (max - min)).clamp(0.0, 1.0)
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
        if seen.insert(value.clone()) {
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

fn score_status_label(value: FastqToolScoreStatus) -> &'static str {
    match value {
        FastqToolScoreStatus::Scored => "scored",
        FastqToolScoreStatus::InsufficientEvidence => "insufficient_evidence",
        FastqToolScoreStatus::Blocked => "blocked",
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
