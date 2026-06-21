use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::stage_scoring::{
    StageScoringConfig, StageScoringCorrectnessSignal, StageScoringDecisionMode,
    DEFAULT_STAGE_SCORING_PATH,
};
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_TOOL_SCORES_PATH: &str = "runs/bench/micro/vcf/VCF_TOOL_SCORES.tsv";

const VCF_TOOL_SCORES_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_tool_scores.v1";
const DEFAULT_FULL_BENCHMARK_REPORT_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfToolScoreStatus {
    Scored,
    InsufficientEvidence,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfToolScoreRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) decision_mode: String,
    pub(crate) correctness_signal: String,
    pub(crate) result_ids: Vec<String>,
    pub(crate) report_row_ids: Vec<String>,
    pub(crate) corpus_ids: Vec<String>,
    pub(crate) report_sections: Vec<String>,
    pub(crate) row_statuses: Vec<String>,
    pub(crate) score_status: VcfToolScoreStatus,
    pub(crate) truth_correctness_score: Option<f64>,
    pub(crate) truth_correctness_basis: Option<String>,
    pub(crate) contract_correctness_score: Option<f64>,
    pub(crate) contract_correctness_basis: Option<String>,
    pub(crate) genotype_truth_metric_value: Option<f64>,
    pub(crate) genotype_truth_metric_basis: Option<String>,
    pub(crate) missingness_metric_value: Option<f64>,
    pub(crate) missingness_metric_basis: Option<String>,
    pub(crate) phasing_imputation_metric_value: Option<f64>,
    pub(crate) phasing_imputation_metric_basis: Option<String>,
    pub(crate) population_metric_value: Option<f64>,
    pub(crate) population_metric_basis: Option<String>,
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
pub(crate) struct VcfToolScoresReport {
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
    pub(crate) rows: Vec<VcfToolScoreRow>,
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
struct VcfEvidenceAggregate {
    source_paths: BTreeSet<String>,
    truth_correctness_score: Option<f64>,
    truth_correctness_basis: Option<String>,
    contract_correctness_score: Option<f64>,
    contract_correctness_basis: Option<String>,
    genotype_truth_metric_value: Option<f64>,
    genotype_truth_metric_basis: Option<String>,
    missingness_metric_value: Option<f64>,
    missingness_metric_basis: Option<String>,
    phasing_imputation_metric_value: Option<f64>,
    phasing_imputation_metric_basis: Option<String>,
    population_metric_value: Option<f64>,
    population_metric_basis: Option<String>,
    scientific_metric_summary: Option<String>,
    runtime_seconds: Option<f64>,
    runtime_source: Option<String>,
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
    genotype_truth_metric_value: Option<f64>,
    genotype_truth_metric_basis: Option<String>,
    missingness_metric_value: Option<f64>,
    missingness_metric_basis: Option<String>,
    phasing_imputation_metric_value: Option<f64>,
    phasing_imputation_metric_basis: Option<String>,
    population_metric_value: Option<f64>,
    population_metric_basis: Option<String>,
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

pub(crate) fn run_render_vcf_tool_scores(
    args: &parse::BenchReadinessRenderVcfToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_tool_scores(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_vcf_tool_scores(
    args: &parse::BenchReadinessValidateVcfToolScoresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_vcf_tool_scores(
        &repo_root,
        args.input.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_TOOL_SCORES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_tool_scores(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfToolScoresReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (config_path, rows) = collect_vcf_tool_score_rows(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_tool_scores_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(build_report(
        path_relative_to_repo(repo_root, &output_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

pub(crate) fn validate_vcf_tool_scores(
    repo_root: &Path,
    input_path: PathBuf,
) -> Result<VcfToolScoresReport> {
    let input_path = repo_relative_path(repo_root, &input_path);
    let actual = fs::read_to_string(&input_path)
        .with_context(|| format!("read {}", input_path.display()))?;
    let (config_path, rows) = collect_vcf_tool_score_rows(repo_root)?;
    let expected = render_vcf_tool_scores_tsv(&rows);
    if actual != expected {
        bail!(
            "VCF tool score TSV drifted from governed evidence contracts; rerun `bijux-dna bench readiness render-vcf-tool-scores`"
        );
    }
    Ok(build_report(
        path_relative_to_repo(repo_root, &input_path),
        path_relative_to_repo(repo_root, &config_path),
        rows,
    ))
}

fn collect_vcf_tool_score_rows(repo_root: &Path) -> Result<(PathBuf, Vec<VcfToolScoreRow>)> {
    let config_path = repo_root.join(DEFAULT_STAGE_SCORING_PATH);
    let config = load_stage_scoring_config(&config_path)?;
    let failure_catalog = config
        .failure_classes
        .iter()
        .map(|row| (row.class_id.clone(), row.detail.clone()))
        .collect::<BTreeMap<_, _>>();
    let full_report = load_full_benchmark_report(repo_root)?;
    let micro_report = load_micro_benchmark_report(repo_root)?;
    let evidence = load_vcf_evidence(repo_root)?;

    let full_rows_by_binding = full_report.rows.into_iter().filter(|row| row.domain == "vcf").fold(
        BTreeMap::<(String, String), Vec<FullBenchmarkReportRowView>>::new(),
        |mut map, row| {
            map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
            map
        },
    );
    let runtime_by_binding =
        micro_report.runtime_rows.into_iter().filter(|row| row.domain == "vcf").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkRuntimeRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );
    let memory_by_binding =
        micro_report.memory_source_rows.into_iter().filter(|row| row.domain == "vcf").fold(
            BTreeMap::<(String, String), Vec<MicroBenchmarkMemoryRowView>>::new(),
            |mut map, row| {
                map.entry((row.stage_id.clone(), row.tool_id.clone())).or_default().push(row);
                map
            },
        );

    let vcf_stage_rows = config.rows.iter().filter(|row| row.domain == "vcf").collect::<Vec<_>>();

    let mut base_rows = Vec::new();
    for stage_row in &vcf_stage_rows {
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
                genotype_truth_metric_value: evidence_row
                    .and_then(|row| row.genotype_truth_metric_value),
                genotype_truth_metric_basis: evidence_row
                    .and_then(|row| row.genotype_truth_metric_basis.clone()),
                missingness_metric_value: evidence_row.and_then(|row| row.missingness_metric_value),
                missingness_metric_basis: evidence_row
                    .and_then(|row| row.missingness_metric_basis.clone()),
                phasing_imputation_metric_value: evidence_row
                    .and_then(|row| row.phasing_imputation_metric_value),
                phasing_imputation_metric_basis: evidence_row
                    .and_then(|row| row.phasing_imputation_metric_basis.clone()),
                population_metric_value: evidence_row.and_then(|row| row.population_metric_value),
                population_metric_basis: evidence_row
                    .and_then(|row| row.population_metric_basis.clone()),
                scientific_metric_summary: evidence_row
                    .and_then(|row| row.scientific_metric_summary.clone()),
                runtime_seconds: runtime_row
                    .as_ref()
                    .and_then(|row| row.elapsed_seconds)
                    .or_else(|| evidence_row.and_then(|row| row.runtime_seconds)),
                runtime_source: runtime_row.as_ref().map_or_else(
                    || {
                        evidence_row
                            .and_then(|row| row.runtime_source.clone())
                            .unwrap_or_else(|| "not_available".to_string())
                    },
                    |row| row.runtime_source.clone(),
                ),
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

        rows.push(VcfToolScoreRow {
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
            genotype_truth_metric_value: row.genotype_truth_metric_value,
            genotype_truth_metric_basis: row.genotype_truth_metric_basis,
            missingness_metric_value: row.missingness_metric_value,
            missingness_metric_basis: row.missingness_metric_basis,
            phasing_imputation_metric_value: row.phasing_imputation_metric_value,
            phasing_imputation_metric_basis: row.phasing_imputation_metric_basis,
            population_metric_value: row.population_metric_value,
            population_metric_basis: row.population_metric_basis,
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

fn load_vcf_evidence(repo_root: &Path) -> Result<BTreeMap<(String, String), VcfEvidenceAggregate>> {
    let mut rows = BTreeMap::<(String, String), VcfEvidenceAggregate>::new();

    merge_admixture_summary(repo_root, &mut rows)?;
    merge_call_summary(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/vcf.call/bcftools/metrics.json",
        "variant_count",
        "variant_count",
    )?;
    merge_call_summary(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/vcf.call_diploid/bcftools/metrics.json",
        "called_genotypes",
        "called_genotypes",
    )?;
    merge_call_gl_summary(repo_root, &mut rows)?;
    merge_call_pseudohaploid_summary(repo_root, &mut rows)?;
    merge_damage_filter_summary(repo_root, &mut rows)?;
    merge_filter_summary(repo_root, &mut rows)?;
    merge_gl_propagation_summary(repo_root, &mut rows)?;
    merge_imputation_metrics_summary(repo_root, &mut rows)?;
    merge_impute_summary(repo_root, &mut rows)?;
    merge_pca_summary(repo_root, &mut rows, "runs/bench/local-smoke/vcf.pca/plink2/pca.json")?;
    merge_pca_summary(repo_root, &mut rows, "runs/bench/local-smoke/vcf.pca/eigensoft/pca.json")?;
    merge_phasing_summary(repo_root, &mut rows)?;
    merge_population_structure_summary(repo_root, &mut rows)?;
    merge_postprocess_summary(repo_root, &mut rows)?;
    merge_prepare_reference_panel_summary(repo_root, &mut rows)?;
    merge_qc_summary(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/vcf.qc/bcftools/qc_summary.json",
        "bcftools",
    )?;
    merge_qc_summary(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/vcf.qc/plink/qc_summary.json",
        "plink",
    )?;
    merge_qc_summary(
        repo_root,
        &mut rows,
        "runs/bench/local-smoke/vcf.qc/plink2/qc_summary.json",
        "plink2",
    )?;
    merge_roh_summary(repo_root, &mut rows)?;
    merge_stats_summary(repo_root, &mut rows)?;

    Ok(rows)
}

fn merge_admixture_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.admixture/plink2/admixture.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let sample_count = required_u64(&root, "sample_count")?;
    let population_count = required_u64(&root, "population_count")?;
    let selected_k = required_u64(&root, "selected_k")?;
    let status = required_str(&root, "status")?;
    let tool_ok = required_bool(&root, "tool_ok")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                selected_k.min(population_count),
                population_count.max(1),
            )),
            truth_correctness_basis: Some("selected_k_over_population_count".to_string()),
            contract_correctness_score: Some(if tool_ok && status == "complete" {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "admixture smoke report preserved governed selected_k and cluster outputs"
                    .to_string(),
            ),
            population_metric_value: Some(population_count as f64),
            population_metric_basis: Some("population_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("selected_k={selected_k}"),
                format!("sample_count={sample_count}"),
                format!("population_count={population_count}"),
                format!("status={status}"),
            ])),
            runtime_seconds: optional_f64(&root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_call_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
    relative_path: &str,
    genotype_key: &str,
    genotype_basis: &str,
) -> Result<()> {
    let path = repo_root.join(relative_path);
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let genotype_value = required_u64(&root, genotype_key)? as f64;
    let exit_code = required_i64(&root, "exit_code")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if exit_code == 0 && genotype_value > 0.0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "VCF call smoke preserved governed output payloads and call counts".to_string(),
            ),
            genotype_truth_metric_value: Some(genotype_value),
            genotype_truth_metric_basis: Some(genotype_basis.to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(&root, "sample_count")?),
                format!("{genotype_key}={}", format_u64_as_string(genotype_value as u64)),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_call_gl_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.call_gl/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let sites_with_likelihoods = required_u64(&root, "sites_with_likelihoods")?;
    let missing_likelihoods = required_u64(&root, "missing_likelihoods")?;
    let total_sites = sites_with_likelihoods + missing_likelihoods;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(sites_with_likelihoods, total_sites.max(1))),
            truth_correctness_basis: Some("likelihood_site_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "genotype-likelihood smoke preserved governed likelihood fields".to_string(),
            ),
            genotype_truth_metric_value: Some(sites_with_likelihoods as f64),
            genotype_truth_metric_basis: Some("sites_with_likelihoods".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(&root, "sample_count")?),
                format!(
                    "samples_with_likelihoods={}",
                    required_u64(&root, "samples_with_likelihoods")?
                ),
                format!("missing_likelihoods={missing_likelihoods}"),
                format!("likelihood_field={}", required_str(&root, "likelihood_field")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_call_pseudohaploid_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path =
        repo_root.join("runs/bench/local-smoke/vcf.call_pseudohaploid/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let called_sites = required_u64(&root, "called_sites")?;
    let target_sites = required_u64(&root, "target_sites")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(called_sites, target_sites.max(1))),
            truth_correctness_basis: Some("called_site_fraction".to_string()),
            contract_correctness_score: Some(
                if required_i64(&root, "exit_code")? == 0
                    && required_bool(&root, "deterministic_replay_match")?
                {
                    1.0
                } else {
                    0.0
                },
            ),
            contract_correctness_basis: Some(
                "pseudohaploid smoke preserved governed deterministic replay and site counts"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(called_sites as f64),
            genotype_truth_metric_basis: Some("called_sites".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("covered_sites={}", required_u64(&root, "covered_sites")?),
                format!("missing_sites={}", required_u64(&root, "missing_sites")?),
                format!("sample_count={}", required_u64(&root, "sample_count")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_damage_filter_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.damage_filter/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let retained_variants = required_u64(&root, "retained_variants")?;
    let input_variants = required_u64(&root, "input_variants")?;
    let removed_variants = required_u64(&root, "removed_variants")?;
    let terminal_damage_filtered_variants =
        required_u64(&root, "terminal_damage_filtered_variants")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(retained_variants, input_variants.max(1))),
            truth_correctness_basis: Some("retained_variant_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "damage filter smoke preserved governed damage-aware variant filtering outputs"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(retained_variants as f64),
            genotype_truth_metric_basis: Some("retained_variants".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("removed_variants={removed_variants}"),
                format!("retained_variants={retained_variants}"),
                format!("terminal_damage_filtered_variants={terminal_damage_filtered_variants}"),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_filter_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.filter/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let input_variants = required_u64(&root, "input_variants")?;
    let pass_variants = required_u64(&root, "pass_variants")?;
    let failed_variants = required_u64(&root, "failed_variants")?;
    let contract_ok =
        required_i64(&root, "exit_code")? == 0 && input_variants == pass_variants + failed_variants;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "VCF filter smoke preserved governed pass and failed variant counts".to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("pass_variants={pass_variants}"),
                format!("failed_variants={failed_variants}"),
                format!(
                    "quality_threshold={}",
                    format_f64(required_f64(&root, "quality_threshold")?)
                ),
                format!("depth_threshold={}", format_f64(required_f64(&root, "depth_threshold")?)),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_gl_propagation_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.gl_propagation/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let sample_count = required_u64(&root, "sample_count")?;
    let site_count_before = required_u64(&root, "site_count_before")?;
    let site_count_after = required_u64(&root, "site_count_after")?;
    let lost_fields = required_array(&root, "lost_fields")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(site_count_after, site_count_before.max(1))),
            truth_correctness_basis: Some("retained_site_fraction".to_string()),
            contract_correctness_score: Some(
                if required_i64(&root, "exit_code")? == 0 && lost_fields.is_empty() {
                    1.0
                } else {
                    0.0
                },
            ),
            contract_correctness_basis: Some(
                "genotype-likelihood propagation preserved governed sample and site surfaces"
                    .to_string(),
            ),
            genotype_truth_metric_value: Some(site_count_after as f64),
            genotype_truth_metric_basis: Some("site_count_after".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={sample_count}"),
                format!("site_count_before={site_count_before}"),
                format!("site_count_after={site_count_after}"),
                format!(
                    "output_likelihood_fields={}",
                    unique_strings(
                        required_array(&root, "output_likelihood_fields")?
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string),
                    )
                    .join(",")
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_imputation_metrics_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root
        .join("runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let concordance = required_f64(&root, "concordance")?;
    let dosage_r2 = required_f64(&root, "dosage_r2")?;
    let mean_info_score = required_f64(&root, "mean_info_score")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(concordance),
            truth_correctness_basis: Some("concordance".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "imputation metrics smoke preserved governed concordance and dosage quality outputs".to_string(),
            ),
            phasing_imputation_metric_value: Some(dosage_r2),
            phasing_imputation_metric_basis: Some("dosage_r2".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("concordance={}", format_f64(concordance)),
                format!("dosage_r2={}", format_f64(dosage_r2)),
                format!("mean_info_score={}", format_f64(mean_info_score)),
                format!("low_confidence_sites={}", required_u64(&root, "low_confidence_sites")?),
                format!("masked_truth_sites={}", required_u64(&root, "masked_truth_sites")?),
            ])),
            runtime_seconds: optional_f64(&root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_impute_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.impute/beagle/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let masked_truth_match_count = required_u64(&root, "masked_truth_match_count")?;
    let masked_truth_site_count = required_u64(&root, "masked_truth_site_count")?;
    let imputed_genotypes = required_u64(&root, "imputed_genotypes")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(
                masked_truth_match_count,
                masked_truth_site_count.max(1),
            )),
            truth_correctness_basis: Some("masked_truth_match_fraction".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "imputation smoke preserved governed masked-truth and completion outputs"
                    .to_string(),
            ),
            phasing_imputation_metric_value: Some(fraction_u64(
                masked_truth_match_count,
                masked_truth_site_count.max(1),
            )),
            phasing_imputation_metric_basis: Some("masked_truth_match_fraction".to_string()),
            genotype_truth_metric_value: Some(imputed_genotypes as f64),
            genotype_truth_metric_basis: Some("imputed_genotypes".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("missing_before={}", required_u64(&root, "missing_before")?),
                format!("missing_after={}", required_u64(&root, "missing_after")?),
                format!("low_confidence_count={}", required_u64(&root, "low_confidence_count")?),
                format!("unresolved_count={}", required_u64(&root, "unresolved_count")?),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_pca_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
    relative_path: &str,
) -> Result<()> {
    let path = repo_root.join(relative_path);
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let excluded_samples = required_array(&root, "excluded_samples")?;
    let unexpected_samples = required_array(&root, "unexpected_samples")?;
    let tool_ok = required_bool(&root, "tool_ok")?;
    let variant_count = required_u64(&root, "variant_count")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(
                if tool_ok && excluded_samples.is_empty() && unexpected_samples.is_empty() {
                    1.0
                } else {
                    0.0
                },
            ),
            truth_correctness_basis: Some("sample_complete_projection".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "PCA smoke preserved governed eigenvalue and coordinate outputs".to_string(),
            ),
            population_metric_value: Some(variant_count as f64),
            population_metric_basis: Some("variant_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={}", required_u64(&root, "sample_count")?),
                format!("variant_count={variant_count}"),
                format!("eigenvalue_count={}", required_array(&root, "eigenvalues")?.len()),
                format!("execution_mode={}", required_str(&root, "execution_mode")?),
            ])),
            runtime_seconds: optional_f64(&root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_phasing_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let metrics_path = repo_root.join("runs/bench/local-smoke/vcf.phasing/shapeit5/metrics.json");
    let metrics = match load_optional_json_value(&metrics_path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let stage_result_path =
        repo_root.join("runs/bench/local-smoke/vcf/vcf.phasing/shapeit5/stage-result.json");
    let stage_result = load_optional_json_value(&stage_result_path)?;
    let input_genotypes = required_u64(&metrics, "input_genotypes")?;
    let phased_genotypes = required_u64(&metrics, "phased_genotypes")?;
    let contract_ok = if let Some(stage_result) = stage_result.as_ref() {
        stage_result.get("runtime").and_then(|value| value.get("status")).and_then(Value::as_str)
            == Some("succeeded")
    } else {
        required_i64(&metrics, "exit_code")? == 0
    };
    let runtime_seconds = stage_result
        .as_ref()
        .and_then(|value| value.get("runtime"))
        .and_then(|value| value.get("elapsed_seconds"))
        .and_then(Value::as_f64);
    insert_evidence(
        rows,
        required_str(&metrics, "stage_id")?,
        required_str(&metrics, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(phased_genotypes, input_genotypes.max(1))),
            truth_correctness_basis: Some("phased_genotype_fraction".to_string()),
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "phasing smoke preserved governed phased VCF, QC, and manifest outputs".to_string(),
            ),
            phasing_imputation_metric_value: Some(fraction_u64(
                phased_genotypes,
                input_genotypes.max(1),
            )),
            phasing_imputation_metric_basis: Some("phased_genotype_fraction".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("phase_set_count={}", required_u64(&metrics, "phase_set_count")?),
                format!("sample_count={}", required_u64(&metrics, "sample_count")?),
                format!("unphased_genotypes={}", required_u64(&metrics, "unphased_genotypes")?),
            ])),
            runtime_seconds,
            runtime_source: runtime_seconds.map(|_| "stage_result_manifest".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &metrics_path,
    );
    if stage_result.is_some() {
        insert_evidence(
            rows,
            required_str(&metrics, "stage_id")?,
            required_str(&metrics, "tool_id")?,
            VcfEvidenceAggregate {
                source_paths: BTreeSet::new(),
                ..VcfEvidenceAggregate::default()
            },
            &stage_result_path,
        );
    }
    Ok(())
}

fn merge_population_structure_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root
        .join("runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let distance_summary = required_object(&root, "distance_summary")?;
    let pair_count = required_u64_from_object(distance_summary, "pair_count")?;
    let sample_count = required_u64_from_object(distance_summary, "sample_count")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(if required_str(&root, "status")? == "complete" { 1.0 } else { 0.0 }),
            truth_correctness_basis: Some("status_complete".to_string()),
            contract_correctness_score: Some(if required_i64(&root, "exit_code")? == 0 { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "population-structure smoke preserved governed distance summaries and consumed PCA/admixture evidence".to_string(),
            ),
            population_metric_value: Some(pair_count as f64),
            population_metric_basis: Some("pair_count".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("sample_count={sample_count}"),
                format!("pair_count={pair_count}"),
                format!(
                    "within_population_pair_count={}",
                    required_u64_from_object(distance_summary, "within_population_pair_count")?
                ),
                format!(
                    "cross_population_pair_count={}",
                    required_u64_from_object(distance_summary, "cross_population_pair_count")?
                ),
            ])),
            runtime_seconds: optional_f64(&root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_postprocess_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.postprocess/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let contract_ok = required_i64(&root, "exit_code")? == 0
        && required_bool(&root, "readable_vcf")?
        && required_bool(&root, "tabix_present")?
        && required_bool(&root, "contigs_consistent_with_species_context")?;
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "postprocess smoke preserved governed readability, contig, and indexing outputs"
                    .to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("record_count={}", required_u64(&root, "record_count")?),
                format!(
                    "multiallelic_records_split={}",
                    required_u64(&root, "multiallelic_records_split")?
                ),
                format!("indels_normalized={}", required_u64(&root, "indels_normalized")?),
                format!(
                    "variant_ids_normalized={}",
                    required_u64(&root, "variant_ids_normalized")?
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_prepare_reference_panel_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path =
        repo_root.join("runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let contract_ok = required_i64(&root, "exit_code")? == 0
        && required_str(&root, "normalization_status")? == "sorted_indexed_deduplicated";
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "reference-panel preparation smoke preserved governed normalized panel outputs"
                    .to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("panel_id={}", required_str(&root, "panel_id")?),
                format!("map_id={}", required_str(&root, "map_id")?),
                format!("input_variants={}", required_u64(&root, "input_variants")?),
                format!("output_variants={}", required_u64(&root, "output_variants")?),
                format!(
                    "duplicate_sites_removed={}",
                    required_u64(&root, "duplicate_sites_removed")?
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_qc_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
    relative_path: &str,
    tool_id: &str,
) -> Result<()> {
    let path = repo_root.join(relative_path);
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let missingness_post = required_f64(&root, "missingness_post")?;
    let imputation_info_mean = required_f64(&root, "imputation_info_mean")?;
    let rsq_mean = required_f64(&root, "rsq_mean")?;
    let truth_score = ((1.0 - missingness_post).clamp(0.0, 1.0)
        + imputation_info_mean.clamp(0.0, 1.0)
        + rsq_mean.clamp(0.0, 1.0))
        / 3.0;
    insert_evidence(
        rows,
        "vcf.qc",
        tool_id,
        VcfEvidenceAggregate {
            truth_correctness_score: Some(truth_score),
            truth_correctness_basis: Some("mean_missingness_info_rsq_score".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "QC smoke summary preserved governed missingness and imputation-quality outputs"
                    .to_string(),
            ),
            missingness_metric_value: Some((1.0 - missingness_post).clamp(0.0, 1.0)),
            missingness_metric_basis: Some("one_minus_missingness_post".to_string()),
            phasing_imputation_metric_value: Some(rsq_mean.clamp(0.0, 1.0)),
            phasing_imputation_metric_basis: Some("rsq_mean".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("missingness_pre={}", format_f64(required_f64(&root, "missingness_pre")?)),
                format!("missingness_post={}", format_f64(missingness_post)),
                format!("imputation_info_mean={}", format_f64(imputation_info_mean)),
                format!("rsq_mean={}", format_f64(rsq_mean)),
                format!("excluded_samples={}", required_array(&root, "excluded_samples")?.len()),
                format!("excluded_variants={}", required_array(&root, "excluded_variants")?.len()),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_roh_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.roh/plink2/roh.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let contract_ok =
        required_i64(&root, "exit_code")? == 0 && required_str(&root, "status")? == "complete";
    insert_evidence(
        rows,
        required_str(&root, "stage_id")?,
        required_str(&root, "tool_id")?,
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "ROH smoke preserved governed segment and per-sample summary outputs".to_string(),
            ),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("segment_count={}", required_u64(&root, "segment_count")?),
                format!("sample_count={}", required_u64(&root, "sample_count")?),
                format!("total_length={}", format_f64(required_f64(&root, "total_length")?)),
            ])),
            runtime_seconds: optional_f64(&root, "elapsed_seconds"),
            runtime_source: Some("local_smoke_report".to_string()),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn merge_stats_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
) -> Result<()> {
    let path = repo_root.join("runs/bench/local-smoke/vcf.stats/bcftools/stats.json");
    let root = match load_optional_json_value(&path)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let variants_total = required_u64(&root, "variants_total")?;
    let snps = required_u64(&root, "snps")?;
    let indels = required_u64(&root, "indels")?;
    let missingness_post = required_f64(&root, "missingness_post")?;
    let contract_ok =
        required_f64(&root, "annotation_coverage")? == 1.0 && variants_total == snps + indels;
    insert_evidence(
        rows,
        "vcf.stats",
        "bcftools",
        VcfEvidenceAggregate {
            contract_correctness_score: Some(if contract_ok { 1.0 } else { 0.0 }),
            contract_correctness_basis: Some(
                "stats smoke preserved governed variant totals and annotation coverage".to_string(),
            ),
            missingness_metric_value: Some((1.0 - missingness_post).clamp(0.0, 1.0)),
            missingness_metric_basis: Some("one_minus_missingness_post".to_string()),
            scientific_metric_summary: Some(summarize_metrics(vec![
                format!("variants_total={variants_total}"),
                format!("snps={snps}"),
                format!("indels={indels}"),
                format!("ti_tv={}", format_f64(required_f64(&root, "ti_tv")?)),
                format!(
                    "heterozygosity_ratio={}",
                    format_f64(required_f64(&root, "heterozygosity_ratio")?)
                ),
            ])),
            source_paths: BTreeSet::new(),
            ..VcfEvidenceAggregate::default()
        },
        &path,
    );
    Ok(())
}

fn insert_evidence(
    rows: &mut BTreeMap<(String, String), VcfEvidenceAggregate>,
    stage_id: &str,
    tool_id: &str,
    evidence: VcfEvidenceAggregate,
    source_path: &Path,
) {
    let entry = rows.entry((stage_id.to_string(), tool_id.to_string())).or_default();
    entry.source_paths.insert(source_path.display().to_string());
    merge_option(&mut entry.truth_correctness_score, evidence.truth_correctness_score);
    merge_string(&mut entry.truth_correctness_basis, evidence.truth_correctness_basis);
    merge_option(&mut entry.contract_correctness_score, evidence.contract_correctness_score);
    merge_string(&mut entry.contract_correctness_basis, evidence.contract_correctness_basis);
    merge_option(&mut entry.genotype_truth_metric_value, evidence.genotype_truth_metric_value);
    merge_string(&mut entry.genotype_truth_metric_basis, evidence.genotype_truth_metric_basis);
    merge_option(&mut entry.missingness_metric_value, evidence.missingness_metric_value);
    merge_string(&mut entry.missingness_metric_basis, evidence.missingness_metric_basis);
    merge_option(
        &mut entry.phasing_imputation_metric_value,
        evidence.phasing_imputation_metric_value,
    );
    merge_string(
        &mut entry.phasing_imputation_metric_basis,
        evidence.phasing_imputation_metric_basis,
    );
    merge_option(&mut entry.population_metric_value, evidence.population_metric_value);
    merge_string(&mut entry.population_metric_basis, evidence.population_metric_basis);
    merge_string(&mut entry.scientific_metric_summary, evidence.scientific_metric_summary);
    merge_option(&mut entry.runtime_seconds, evidence.runtime_seconds);
    merge_string(&mut entry.runtime_source, evidence.runtime_source);
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
) -> (VcfToolScoreStatus, f64, Option<f64>) {
    if row.failure_class != "none" {
        return match row.failure_class.as_str() {
            "insufficient_data" => (VcfToolScoreStatus::InsufficientEvidence, 0.0, None),
            _ => (VcfToolScoreStatus::Blocked, 0.0, None),
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
        return (VcfToolScoreStatus::InsufficientEvidence, 0.0, None);
    }
    (VcfToolScoreStatus::Scored, covered_weight, Some(weighted_sum / covered_weight))
}

fn classify_failure_class(
    binding_rows: &[FullBenchmarkReportRowView],
    evidence: Option<&VcfEvidenceAggregate>,
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
    evidence: Option<&VcfEvidenceAggregate>,
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
                "{detail}; no real VCF smoke or micro evidence row was found for `{}` / `{}`",
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
    rows: Vec<VcfToolScoreRow>,
) -> VcfToolScoresReport {
    let stage_count = rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len();
    let tool_count = rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len();
    let scored_row_count =
        rows.iter().filter(|row| row.score_status == VcfToolScoreStatus::Scored).count();
    let insufficient_evidence_row_count = rows
        .iter()
        .filter(|row| row.score_status == VcfToolScoreStatus::InsufficientEvidence)
        .count();
    let blocked_row_count =
        rows.iter().filter(|row| row.score_status == VcfToolScoreStatus::Blocked).count();
    let failure_class_counts =
        rows.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, row| {
            *counts.entry(row.failure_class.clone()).or_default() += 1;
            counts
        });
    VcfToolScoresReport {
        schema_version: VCF_TOOL_SCORES_SCHEMA_VERSION,
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

fn render_vcf_tool_scores_tsv(rows: &[VcfToolScoreRow]) -> String {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(
        "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tgenotype_truth_metric_value\tgenotype_truth_metric_basis\tmissingness_metric_value\tmissingness_metric_basis\tphasing_imputation_metric_value\tphasing_imputation_metric_basis\tpopulation_metric_value\tpopulation_metric_basis\tscientific_metric_ids\tscientific_metric_summary\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason".to_string(),
    );
    for row in rows {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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
            format_optional_f64(row.genotype_truth_metric_value),
            row.genotype_truth_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.missingness_metric_value),
            row.missingness_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.phasing_imputation_metric_value),
            row.phasing_imputation_metric_basis.clone().unwrap_or_default(),
            format_optional_f64(row.population_metric_value),
            row.population_metric_basis.clone().unwrap_or_default(),
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

fn score_status_label(status: VcfToolScoreStatus) -> &'static str {
    match status {
        VcfToolScoreStatus::Scored => "scored",
        VcfToolScoreStatus::InsufficientEvidence => "insufficient_evidence",
        VcfToolScoreStatus::Blocked => "blocked",
    }
}

fn decision_mode_label(mode: StageScoringDecisionMode) -> &'static str {
    match mode {
        StageScoringDecisionMode::SingleToolAcceptance => "single_tool_acceptance",
        StageScoringDecisionMode::MultiToolRanking => "multi_tool_ranking",
    }
}

fn correctness_signal_label(signal: StageScoringCorrectnessSignal) -> &'static str {
    match signal {
        StageScoringCorrectnessSignal::ScientificComparableMetrics => {
            "scientific_comparable_metrics"
        }
        StageScoringCorrectnessSignal::OutputContract => "output_contract",
    }
}

fn normalize_lower_is_better(value: f64, min: f64, max: f64) -> f64 {
    if !min.is_finite() || !max.is_finite() || (max - min).abs() < f64::EPSILON {
        return 1.0;
    }
    ((max - value) / (max - min)).clamp(0.0, 1.0)
}

fn execution_status_rank(status: &str) -> usize {
    match status {
        "succeeded" => 0,
        "unavailable" => 1,
        "container_needed" => 2,
        "failed" => 3,
        _ => 4,
    }
}

fn option_f64_cmp(left: Option<f64>, right: Option<f64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values.into_iter().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>()
}

fn join_csv(values: &[String]) -> String {
    values.join(",")
}

fn format_optional_f64(value: Option<f64>) -> String {
    value.map(format_f64).unwrap_or_default()
}

fn format_f64(value: f64) -> String {
    format!("{value:.6}")
}

fn format_optional_u64(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn format_u64_as_string(value: u64) -> String {
    value.to_string()
}

fn sanitize_tsv_cell(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ")
}

fn summarize_metrics(values: Vec<String>) -> String {
    values.into_iter().filter(|value| !value.is_empty()).collect::<Vec<_>>().join("; ")
}

fn fraction_u64(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    numerator as f64 / denominator as f64
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn load_optional_json_value(path: &Path) -> Result<Option<Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display())).map(Some)
}

fn required_object<'a>(value: &'a Value, key: &str) -> Result<&'a serde_json::Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .with_context(|| format!("missing object field `{key}`"))
}

fn required_array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value]> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .with_context(|| format!("missing array field `{key}`"))
}

fn required_str<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).with_context(|| format!("missing string field `{key}`"))
}

fn required_u64(value: &Value, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).with_context(|| format!("missing integer field `{key}`"))
}

fn required_u64_from_object(value: &serde_json::Map<String, Value>, key: &str) -> Result<u64> {
    value.get(key).and_then(Value::as_u64).with_context(|| format!("missing integer field `{key}`"))
}

fn required_i64(value: &Value, key: &str) -> Result<i64> {
    value.get(key).and_then(Value::as_i64).with_context(|| format!("missing integer field `{key}`"))
}

fn required_f64(value: &Value, key: &str) -> Result<f64> {
    value.get(key).and_then(Value::as_f64).with_context(|| format!("missing float field `{key}`"))
}

fn required_bool(value: &Value, key: &str) -> Result<bool> {
    value.get(key).and_then(Value::as_bool).with_context(|| format!("missing bool field `{key}`"))
}

fn optional_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}
