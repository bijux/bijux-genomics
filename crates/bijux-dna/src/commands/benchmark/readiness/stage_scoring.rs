use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::default_execution_tool_for_stage;
use bijux_dna_planner_bam::stage_api::default_tool_for_stage as default_bam_tool_for_stage;
use serde::{Deserialize, Serialize};

use super::all_domain_active_stage_catalog::{
    collect_all_domain_active_stage_catalog_rows, AllDomainActiveStageCatalogRow,
    DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH,
};
use super::all_domain_failure_classification::{
    DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH, REQUIRED_FAILURE_CLASSES,
};
use super::full_benchmark_report::DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH;
use super::scientific_acceptance_thresholds::{
    render_scientific_acceptance_thresholds, ScientificAcceptanceThresholdRow,
    DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH,
};
use super::tool_smoke_support::path_relative_to_repo;
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_SCORING_PATH: &str = "configs/bench/local/stage-scoring.toml";
const STAGE_SCORING_SCHEMA_VERSION: &str = "bijux.bench.local_stage_scoring.v1";
const STAGE_SCORING_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.stage_scoring.v1";
const STAGE_SCORING_FULL_BENCHMARK_SOURCE: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
const STAGE_SCORING_MICRO_SOURCE: &str = "runs/bench/micro/MICRO_BENCHMARK_REPORT.json";
const STAGE_SCORING_FAILURE_CLASS_SOURCE: &str =
    "benchmarks/readiness/failure-classification-all-domains.json";
const STAGE_SCORING_ACTIVE_STAGE_SOURCE: &str =
    "benchmarks/readiness/all-domains/active-stage-catalog.tsv";
const STAGE_SCORING_THRESHOLD_SOURCE: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringDecisionMode {
    MultiToolRanking,
    SingleToolAcceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringCorrectnessSignal {
    ScientificComparableMetrics,
    OutputContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringApplicability {
    Required,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringDirection {
    LowerIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringMissingBehavior {
    ExcludeFromRanking,
    FallbackToDeclaredMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StageScoringFailureEffect {
    BlockRecommendation,
    MarkInsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringWeights {
    pub(crate) correctness: f64,
    pub(crate) scientific_threshold: f64,
    pub(crate) runtime: f64,
    pub(crate) memory: f64,
    pub(crate) completion: f64,
    pub(crate) failure_class: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringCorrectness {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) signal: StageScoringCorrectnessSignal,
    pub(crate) metric_ids: Vec<String>,
    pub(crate) minimum_present_result_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringScientificThreshold {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) applicability: StageScoringApplicability,
    pub(crate) metric_ids: Vec<String>,
    pub(crate) gate: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringRuntime {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) metric_id: String,
    pub(crate) direction: StageScoringDirection,
    pub(crate) missing_behavior: StageScoringMissingBehavior,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringMemory {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) metric_id: String,
    pub(crate) fallback_metric_id: String,
    pub(crate) direction: StageScoringDirection,
    pub(crate) missing_behavior: StageScoringMissingBehavior,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringCompletion {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) required_row_statuses: Vec<String>,
    pub(crate) required_execution_statuses: Vec<String>,
    pub(crate) non_scoring_execution_statuses: Vec<String>,
    pub(crate) required_success_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringFailureClass {
    pub(crate) source_report_paths: Vec<String>,
    pub(crate) blocking_class_ids: Vec<String>,
    pub(crate) insufficient_class_ids: Vec<String>,
    pub(crate) allowed_row_statuses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringFailureClassCatalogEntry {
    pub(crate) class_id: String,
    pub(crate) effect: StageScoringFailureEffect,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) decision_mode: StageScoringDecisionMode,
    pub(crate) active_tool_count: usize,
    pub(crate) benchmark_ready_tool_count: usize,
    pub(crate) default_tool_id: String,
    pub(crate) benchmark_ready_tool_ids: Vec<String>,
    pub(crate) report_section_ids: Vec<String>,
    pub(crate) recommendation_gate: String,
    pub(crate) weights: StageScoringWeights,
    pub(crate) correctness: StageScoringCorrectness,
    pub(crate) scientific_threshold: StageScoringScientificThreshold,
    pub(crate) runtime: StageScoringRuntime,
    pub(crate) memory: StageScoringMemory,
    pub(crate) completion: StageScoringCompletion,
    pub(crate) failure_class: StageScoringFailureClass,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageScoringConfig {
    pub(crate) schema_version: String,
    pub(crate) active_stage_catalog_path: String,
    pub(crate) scientific_threshold_config_path: String,
    pub(crate) full_benchmark_report_path: String,
    pub(crate) micro_benchmark_report_path: String,
    pub(crate) failure_class_report_path: String,
    pub(crate) failure_classes: Vec<StageScoringFailureClassCatalogEntry>,
    pub(crate) rows: Vec<StageScoringRow>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageScoringReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) multi_tool_stage_count: usize,
    pub(crate) single_tool_stage_count: usize,
    pub(crate) scientific_stage_count: usize,
    pub(crate) failure_class_count: usize,
    pub(crate) rows: Vec<StageScoringRow>,
}

pub(crate) fn run_render_stage_scoring(
    args: &parse::BenchReadinessRenderStageScoringArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_scoring(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_SCORING_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn run_validate_stage_scoring(
    args: &parse::BenchReadinessValidateStageScoringArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_stage_scoring(
        &repo_root,
        args.config.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_SCORING_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_stage_scoring(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageScoringReport> {
    let output_path = repo_root.join(output_path);
    let config = build_stage_scoring_config(repo_root)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = toml::to_string_pretty(&config).context("serialize stage-scoring TOML")?;
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;
    build_stage_scoring_report(repo_root, &output_path, config.rows)
}

pub(crate) fn validate_stage_scoring(
    repo_root: &Path,
    config_path: PathBuf,
) -> Result<StageScoringReport> {
    let config_path = repo_root.join(config_path);
    let raw = fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let parsed: StageScoringConfig = toml::from_str(&raw).context("parse stage-scoring TOML")?;
    if parsed.schema_version != STAGE_SCORING_SCHEMA_VERSION {
        bail!(
            "stage scoring schema drift: expected `{STAGE_SCORING_SCHEMA_VERSION}`, found `{}`",
            parsed.schema_version
        );
    }
    ensure_stage_scoring_config_contract(repo_root, &parsed)?;
    let expected = build_stage_scoring_config(repo_root)?;
    if parsed != expected {
        let drift = first_stage_scoring_drift(&parsed, &expected);
        bail!(
            "stage scoring config drifted from owned benchmark contracts; {drift}; rerun `bijux-dna bench readiness render-stage-scoring`"
        );
    }
    build_stage_scoring_report(repo_root, &config_path, parsed.rows)
}

fn build_stage_scoring_config(repo_root: &Path) -> Result<StageScoringConfig> {
    let threshold_rows = render_scientific_acceptance_threshold_rows(repo_root)?;
    let threshold_metrics_by_stage = threshold_rows.into_iter().fold(
        BTreeMap::<(String, String), Vec<String>>::new(),
        |mut rows, threshold| {
            rows.entry((threshold.domain, threshold.stage_id))
                .or_default()
                .push(threshold.metric_id);
            rows
        },
    );
    let default_tool_by_stage = collect_default_tool_ids_by_stage(repo_root)?;
    let rows = collect_all_domain_active_stage_catalog_rows(repo_root)?
        .into_iter()
        .map(|stage| {
            build_stage_scoring_row(&stage, &threshold_metrics_by_stage, &default_tool_by_stage)
        })
        .collect::<Result<Vec<_>>>()?;
    let failure_classes = build_failure_class_catalog();
    let config = StageScoringConfig {
        schema_version: STAGE_SCORING_SCHEMA_VERSION.to_string(),
        active_stage_catalog_path: DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH.to_string(),
        scientific_threshold_config_path: DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH.to_string(),
        full_benchmark_report_path: DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH.to_string(),
        micro_benchmark_report_path: DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH.to_string(),
        failure_class_report_path: DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH.to_string(),
        failure_classes,
        rows,
    };
    ensure_stage_scoring_config_contract(repo_root, &config)?;
    Ok(config)
}

fn build_stage_scoring_row(
    stage: &AllDomainActiveStageCatalogRow,
    threshold_metrics_by_stage: &BTreeMap<(String, String), Vec<String>>,
    default_tool_by_stage: &BTreeMap<(String, String), String>,
) -> Result<StageScoringRow> {
    let metrics = threshold_metrics_by_stage
        .get(&(stage.domain.clone(), stage.stage_id.clone()))
        .cloned()
        .unwrap_or_default();
    let decision_mode = if stage.benchmark_ready_tool_count >= 2 {
        StageScoringDecisionMode::MultiToolRanking
    } else {
        StageScoringDecisionMode::SingleToolAcceptance
    };
    let default_tool_id = default_tool_by_stage
        .get(&(stage.domain.clone(), stage.stage_id.clone()))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "stage scoring config is missing a default tool for `{}` / `{}`",
                stage.domain,
                stage.stage_id
            )
        })?;
    let has_scientific_thresholds = !metrics.is_empty();
    let weights = if has_scientific_thresholds {
        StageScoringWeights {
            correctness: 0.35,
            scientific_threshold: 0.25,
            runtime: 0.15,
            memory: 0.10,
            completion: 0.10,
            failure_class: 0.05,
        }
    } else {
        StageScoringWeights {
            correctness: 0.55,
            scientific_threshold: 0.0,
            runtime: 0.15,
            memory: 0.10,
            completion: 0.10,
            failure_class: 0.10,
        }
    };
    let recommendation_gate = match decision_mode {
        StageScoringDecisionMode::MultiToolRanking => {
            "rank benchmark-ready tools only after completion and failure-class gates pass"
        }
        StageScoringDecisionMode::SingleToolAcceptance => {
            "accept the governed default tool only after completion and failure-class gates pass"
        }
    };

    Ok(StageScoringRow {
        domain: stage.domain.clone(),
        stage_id: stage.stage_id.clone(),
        readiness_kind: stage.readiness_kind.clone(),
        decision_mode,
        active_tool_count: stage.active_tool_count,
        benchmark_ready_tool_count: stage.benchmark_ready_tool_count,
        default_tool_id,
        benchmark_ready_tool_ids: stage.benchmark_ready_tool_ids.clone(),
        report_section_ids: stage.report_section_ids.clone(),
        recommendation_gate: recommendation_gate.to_string(),
        weights,
        correctness: StageScoringCorrectness {
            source_report_paths: vec![
                STAGE_SCORING_FULL_BENCHMARK_SOURCE.to_string(),
                STAGE_SCORING_ACTIVE_STAGE_SOURCE.to_string(),
            ],
            signal: if has_scientific_thresholds {
                StageScoringCorrectnessSignal::ScientificComparableMetrics
            } else {
                StageScoringCorrectnessSignal::OutputContract
            },
            metric_ids: metrics.clone(),
            minimum_present_result_rows: stage.benchmark_ready_tool_count.max(1),
        },
        scientific_threshold: StageScoringScientificThreshold {
            source_report_paths: vec![STAGE_SCORING_THRESHOLD_SOURCE.to_string()],
            applicability: if has_scientific_thresholds {
                StageScoringApplicability::Required
            } else {
                StageScoringApplicability::NotApplicable
            },
            metric_ids: metrics,
            gate: if has_scientific_thresholds {
                "must_pass_all_required_stage_thresholds".to_string()
            } else {
                "not_applicable".to_string()
            },
        },
        runtime: StageScoringRuntime {
            source_report_paths: vec![STAGE_SCORING_MICRO_SOURCE.to_string()],
            metric_id: "elapsed_seconds".to_string(),
            direction: StageScoringDirection::LowerIsBetter,
            missing_behavior: StageScoringMissingBehavior::ExcludeFromRanking,
        },
        memory: StageScoringMemory {
            source_report_paths: vec![STAGE_SCORING_MICRO_SOURCE.to_string()],
            metric_id: "observed_memory_mb".to_string(),
            fallback_metric_id: "declared_memory_mb".to_string(),
            direction: StageScoringDirection::LowerIsBetter,
            missing_behavior: StageScoringMissingBehavior::FallbackToDeclaredMemory,
        },
        completion: StageScoringCompletion {
            source_report_paths: vec![
                STAGE_SCORING_FULL_BENCHMARK_SOURCE.to_string(),
                STAGE_SCORING_MICRO_SOURCE.to_string(),
            ],
            required_row_statuses: vec!["present".to_string()],
            required_execution_statuses: vec!["succeeded".to_string()],
            non_scoring_execution_statuses: vec![
                "container_needed".to_string(),
                "unavailable".to_string(),
            ],
            required_success_ratio: 1.0,
        },
        failure_class: StageScoringFailureClass {
            source_report_paths: vec![STAGE_SCORING_FAILURE_CLASS_SOURCE.to_string()],
            blocking_class_ids: blocking_failure_class_ids(),
            insufficient_class_ids: vec!["insufficient_data".to_string()],
            allowed_row_statuses: vec!["present".to_string()],
        },
    })
}

fn build_stage_scoring_report(
    repo_root: &Path,
    output_path: &Path,
    rows: Vec<StageScoringRow>,
) -> Result<StageScoringReport> {
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let multi_tool_stage_count = rows
        .iter()
        .filter(|row| row.decision_mode == StageScoringDecisionMode::MultiToolRanking)
        .count();
    let scientific_stage_count = rows
        .iter()
        .filter(|row| row.scientific_threshold.applicability == StageScoringApplicability::Required)
        .count();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }
    Ok(StageScoringReport {
        schema_version: STAGE_SCORING_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, output_path),
        row_count: rows.len(),
        domain_counts,
        multi_tool_stage_count,
        single_tool_stage_count: rows.len() - multi_tool_stage_count,
        scientific_stage_count,
        failure_class_count: REQUIRED_FAILURE_CLASSES.len(),
        rows,
    })
}

fn render_scientific_acceptance_threshold_rows(
    repo_root: &Path,
) -> Result<Vec<ScientificAcceptanceThresholdRow>> {
    let scratch_root = std::env::temp_dir().join(format!(
        "bijux-stage-scoring-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;
    let result = render_scientific_acceptance_thresholds(
        repo_root,
        scratch_root.join(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
    )
    .map(|report| report.rows);
    let _ = fs::remove_dir_all(&scratch_root);
    result
}

fn collect_default_tool_ids_by_stage(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), String>> {
    let mut rows = BTreeMap::<(String, String), String>::new();

    for stage in build_vcf_stage_matrix_rows()? {
        rows.insert(("vcf".to_string(), stage.stage_id), stage.tool_id);
    }

    for domain in ["fastq", "bam"] {
        for stage_id in active_stage_ids_for_domain(repo_root, domain)? {
            let tool_id = match domain {
                "fastq" => default_execution_tool_for_stage(&StageId::new(stage_id.clone()))
                    .map(|tool_id| tool_id.to_string()),
                "bam" => Some(
                    default_bam_tool_for_stage(
                        BamStage::try_from(stage_id.as_str())
                            .map_err(|error| anyhow!("unknown BAM stage `{stage_id}`: {error}"))?,
                    )
                    .to_string(),
                ),
                _ => None,
            }
            .ok_or_else(|| {
                anyhow!(
                    "stage scoring config is missing a default tool for `{domain}` / `{stage_id}`"
                )
            })?;
            rows.insert((domain.to_string(), stage_id), tool_id);
        }
    }

    Ok(rows)
}

fn active_stage_ids_for_domain(repo_root: &Path, domain: &str) -> Result<Vec<String>> {
    let mut stage_ids = collect_all_domain_active_stage_catalog_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == domain)
        .map(|row| row.stage_id)
        .collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    Ok(stage_ids)
}

fn build_failure_class_catalog() -> Vec<StageScoringFailureClassCatalogEntry> {
    REQUIRED_FAILURE_CLASSES
        .into_iter()
        .map(|class_id| StageScoringFailureClassCatalogEntry {
            class_id: class_id.to_string(),
            effect: if class_id == "insufficient_data" {
                StageScoringFailureEffect::MarkInsufficientEvidence
            } else {
                StageScoringFailureEffect::BlockRecommendation
            },
            detail: match class_id {
                "missing_input" => {
                    "required benchmark inputs are absent, so the tool cannot be compared honestly"
                }
                "tool_not_found" => {
                    "the executable is unresolved, so the tool must be excluded from recommendation"
                }
                "command_failed" => {
                    "the tool invocation failed before governed outputs were materialized"
                }
                "missing_output" => {
                    "the stage did not produce all governed outputs, so completeness is broken"
                }
                "parser_failed" => {
                    "raw tool output could not be normalized into the governed report contract"
                }
                "insufficient_data" => {
                    "the stage completed but the benchmark evidence is not strong enough to rank tools"
                }
                "unsupported_pair" => {
                    "the stage/tool binding is outside the governed benchmark surface"
                }
                _ => "unknown failure classification",
            }
            .to_string(),
        })
        .collect()
}

fn blocking_failure_class_ids() -> Vec<String> {
    REQUIRED_FAILURE_CLASSES
        .into_iter()
        .filter(|class_id| *class_id != "insufficient_data")
        .map(str::to_string)
        .collect()
}

fn ensure_stage_scoring_config_contract(
    repo_root: &Path,
    config: &StageScoringConfig,
) -> Result<()> {
    if config.active_stage_catalog_path != DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH {
        bail!(
            "stage scoring config must reference `{DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_CATALOG_PATH}`, found `{}`",
            config.active_stage_catalog_path
        );
    }
    if config.scientific_threshold_config_path != DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH {
        bail!(
            "stage scoring config must reference `{DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH}`, found `{}`",
            config.scientific_threshold_config_path
        );
    }
    if config.full_benchmark_report_path != DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH {
        bail!(
            "stage scoring config must reference `{DEFAULT_FULL_BENCHMARK_REPORT_JSON_PATH}`, found `{}`",
            config.full_benchmark_report_path
        );
    }
    if config.micro_benchmark_report_path != DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH {
        bail!(
            "stage scoring config must reference `{DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH}`, found `{}`",
            config.micro_benchmark_report_path
        );
    }
    if config.failure_class_report_path != DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH {
        bail!(
            "stage scoring config must reference `{DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH}`, found `{}`",
            config.failure_class_report_path
        );
    }

    let expected_stage_count = collect_all_domain_active_stage_catalog_rows(repo_root)?.len();
    if config.rows.len() != expected_stage_count {
        bail!(
            "stage scoring config covers {} active stages but expected {expected_stage_count}",
            config.rows.len()
        );
    }

    let failure_class_ids =
        config.failure_classes.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>();
    for class_id in REQUIRED_FAILURE_CLASSES {
        if !failure_class_ids.contains(class_id) {
            bail!("stage scoring config is missing failure class `{class_id}`");
        }
    }

    let mut seen_stage_ids = BTreeSet::<(&str, &str)>::new();
    for row in &config.rows {
        if !seen_stage_ids.insert((row.domain.as_str(), row.stage_id.as_str())) {
            bail!(
                "stage scoring config contains duplicate row `{}` / `{}`",
                row.domain,
                row.stage_id
            );
        }
        if !row.benchmark_ready_tool_ids.iter().any(|tool_id| tool_id == &row.default_tool_id) {
            bail!(
                "stage scoring row `{}` / `{}` keeps default tool `{}` outside benchmark-ready tool ids",
                row.domain,
                row.stage_id,
                row.default_tool_id
            );
        }
        if row.report_section_ids.is_empty() {
            bail!(
                "stage scoring row `{}` / `{}` is missing report_section_ids",
                row.domain,
                row.stage_id
            );
        }
        ensure_weight_contract(row)?;
        ensure_scientific_threshold_contract(row)?;
        ensure_runtime_memory_completion_contract(row)?;
        ensure_failure_class_contract(row, &failure_class_ids)?;
        ensure_decision_mode_contract(row)?;
    }
    Ok(())
}

fn ensure_weight_contract(row: &StageScoringRow) -> Result<()> {
    let sum = row.weights.correctness
        + row.weights.scientific_threshold
        + row.weights.runtime
        + row.weights.memory
        + row.weights.completion
        + row.weights.failure_class;
    if (sum - 1.0).abs() > 1e-9 {
        bail!(
            "stage scoring row `{}` / `{}` has weights summing to {sum} instead of 1.0",
            row.domain,
            row.stage_id
        );
    }
    Ok(())
}

fn ensure_scientific_threshold_contract(row: &StageScoringRow) -> Result<()> {
    match row.scientific_threshold.applicability {
        StageScoringApplicability::Required => {
            if row.scientific_threshold.metric_ids.is_empty() {
                bail!(
                    "stage scoring row `{}` / `{}` requires scientific thresholds but has no metric_ids",
                    row.domain,
                    row.stage_id
                );
            }
            if row.weights.scientific_threshold <= 0.0 {
                bail!(
                    "stage scoring row `{}` / `{}` requires scientific thresholds but gives them zero weight",
                    row.domain,
                    row.stage_id
                );
            }
        }
        StageScoringApplicability::NotApplicable => {
            if !row.scientific_threshold.metric_ids.is_empty() {
                bail!(
                    "stage scoring row `{}` / `{}` marks scientific thresholds not_applicable but still declares metric_ids",
                    row.domain,
                    row.stage_id
                );
            }
            if row.weights.scientific_threshold != 0.0 {
                bail!(
                    "stage scoring row `{}` / `{}` marks scientific thresholds not_applicable but still assigns non-zero weight",
                    row.domain,
                    row.stage_id
                );
            }
        }
    }
    Ok(())
}

fn ensure_runtime_memory_completion_contract(row: &StageScoringRow) -> Result<()> {
    if row.runtime.metric_id != "elapsed_seconds" {
        bail!(
            "stage scoring row `{}` / `{}` must use runtime metric `elapsed_seconds`, found `{}`",
            row.domain,
            row.stage_id,
            row.runtime.metric_id
        );
    }
    if row.memory.metric_id != "observed_memory_mb" {
        bail!(
            "stage scoring row `{}` / `{}` must use memory metric `observed_memory_mb`, found `{}`",
            row.domain,
            row.stage_id,
            row.memory.metric_id
        );
    }
    if row.memory.fallback_metric_id != "declared_memory_mb" {
        bail!(
            "stage scoring row `{}` / `{}` must fall back to `declared_memory_mb`, found `{}`",
            row.domain,
            row.stage_id,
            row.memory.fallback_metric_id
        );
    }
    if row.completion.required_row_statuses != ["present".to_string()] {
        bail!(
            "stage scoring row `{}` / `{}` must require full-benchmark row_status `present`",
            row.domain,
            row.stage_id
        );
    }
    if row.completion.required_execution_statuses != ["succeeded".to_string()] {
        bail!(
            "stage scoring row `{}` / `{}` must require micro execution_status `succeeded`",
            row.domain,
            row.stage_id
        );
    }
    if (row.completion.required_success_ratio - 1.0).abs() > 1e-9 {
        bail!(
            "stage scoring row `{}` / `{}` must require success ratio 1.0, found {}",
            row.domain,
            row.stage_id,
            row.completion.required_success_ratio
        );
    }
    Ok(())
}

fn ensure_failure_class_contract(
    row: &StageScoringRow,
    failure_class_ids: &BTreeSet<&str>,
) -> Result<()> {
    if row.failure_class.blocking_class_ids.is_empty() {
        bail!(
            "stage scoring row `{}` / `{}` is missing blocking failure classes",
            row.domain,
            row.stage_id
        );
    }
    for class_id in row
        .failure_class
        .blocking_class_ids
        .iter()
        .chain(row.failure_class.insufficient_class_ids.iter())
    {
        if !failure_class_ids.contains(class_id.as_str()) {
            bail!(
                "stage scoring row `{}` / `{}` references unknown failure class `{}`",
                row.domain,
                row.stage_id,
                class_id
            );
        }
    }
    Ok(())
}

fn ensure_decision_mode_contract(row: &StageScoringRow) -> Result<()> {
    match row.decision_mode {
        StageScoringDecisionMode::MultiToolRanking => {
            if row.benchmark_ready_tool_count < 2 {
                bail!(
                    "stage scoring row `{}` / `{}` claims multi_tool_ranking with only {} benchmark-ready tools",
                    row.domain,
                    row.stage_id,
                    row.benchmark_ready_tool_count
                );
            }
            let expected_signal =
                if row.scientific_threshold.applicability == StageScoringApplicability::Required {
                    StageScoringCorrectnessSignal::ScientificComparableMetrics
                } else {
                    StageScoringCorrectnessSignal::OutputContract
                };
            if row.correctness.signal != expected_signal {
                bail!(
                    "stage scoring row `{}` / `{}` must use correctness signal `{:?}` for its multi-tool ranking contract",
                    row.domain,
                    row.stage_id,
                    expected_signal
                );
            }
        }
        StageScoringDecisionMode::SingleToolAcceptance => {
            if row.benchmark_ready_tool_count != 1 {
                bail!(
                    "stage scoring row `{}` / `{}` claims single_tool_acceptance with {} benchmark-ready tools",
                    row.domain,
                    row.stage_id,
                    row.benchmark_ready_tool_count
                );
            }
        }
    }
    Ok(())
}

fn first_stage_scoring_drift(parsed: &StageScoringConfig, expected: &StageScoringConfig) -> String {
    if parsed.failure_classes != expected.failure_classes {
        return "failure-class catalog changed".to_string();
    }
    for (index, (left, right)) in parsed.rows.iter().zip(expected.rows.iter()).enumerate() {
        if left != right {
            return format!("row {} drifted at `{}` / `{}`", index, left.domain, left.stage_id);
        }
    }
    format!("row count changed from {} to {}", parsed.rows.len(), expected.rows.len())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    use super::{
        render_stage_scoring, StageScoringApplicability, StageScoringCorrectnessSignal,
        StageScoringDecisionMode,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn render_stage_scoring_covers_all_active_stages() {
        let _guard = test_lock().lock().expect("stage scoring unit lock");
        let repo_root = repo_root();
        let temp_dir = tempfile::tempdir_in(repo_root.join("artifacts")).expect("tempdir");
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report =
            render_stage_scoring(&repo_root, relative_output_path.clone()).expect("render stage scoring");

        assert_eq!(report.config_path, relative_output_path.to_string_lossy());
        assert_eq!(report.row_count, 69);
        assert_eq!(report.multi_tool_stage_count, 31);
        assert_eq!(report.single_tool_stage_count, 38);
        assert_eq!(report.scientific_stage_count, 29);
        assert_eq!(report.failure_class_count, 7);
        assert_eq!(report.domain_counts.get("fastq"), Some(&27));
        assert_eq!(report.domain_counts.get("bam"), Some(&24));
        assert_eq!(report.domain_counts.get("vcf"), Some(&18));
        assert!(report.rows.iter().all(|row| !row.report_section_ids.is_empty()));
    }

    #[test]
    fn render_stage_scoring_preserves_multi_tool_ranking_contracts() {
        let _guard = test_lock().lock().expect("stage scoring unit lock");
        let repo_root = repo_root();
        let temp_dir = tempfile::tempdir_in(repo_root.join("artifacts")).expect("tempdir");
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report = render_stage_scoring(&repo_root, relative_output_path)
            .expect("render stage scoring");

        let row = report
            .rows
            .iter()
            .find(|row| row.domain == "fastq" && row.stage_id == "fastq.validate_reads")
            .expect("fastq.validate_reads row");
        assert_eq!(row.decision_mode, StageScoringDecisionMode::MultiToolRanking);
        assert_eq!(row.default_tool_id, "fastqvalidator");
        assert_eq!(row.benchmark_ready_tool_count, 5);
        assert_eq!(
            row.correctness.signal,
            StageScoringCorrectnessSignal::ScientificComparableMetrics
        );
        assert_eq!(row.scientific_threshold.applicability, StageScoringApplicability::Required);
        assert_eq!(
            row.scientific_threshold.metric_ids,
            vec!["format_validation_pass_rate".to_string()]
        );
    }

    #[test]
    fn render_stage_scoring_preserves_single_tool_acceptance_contracts() {
        let _guard = test_lock().lock().expect("stage scoring unit lock");
        let repo_root = repo_root();
        let temp_dir = tempfile::tempdir_in(repo_root.join("artifacts")).expect("tempdir");
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report = render_stage_scoring(&repo_root, relative_output_path)
            .expect("render stage scoring");

        let row = report
            .rows
            .iter()
            .find(|row| row.domain == "bam" && row.stage_id == "bam.complexity")
            .expect("bam.complexity row");
        assert_eq!(row.decision_mode, StageScoringDecisionMode::SingleToolAcceptance);
        assert_eq!(row.default_tool_id, "preseq");
        assert_eq!(row.benchmark_ready_tool_count, 1);
        assert_eq!(row.correctness.signal, StageScoringCorrectnessSignal::OutputContract);
        assert_eq!(
            row.scientific_threshold.applicability,
            StageScoringApplicability::NotApplicable
        );
        assert!(row.scientific_threshold.metric_ids.is_empty());
        assert_eq!(row.weights.scientific_threshold, 0.0);
    }
}
