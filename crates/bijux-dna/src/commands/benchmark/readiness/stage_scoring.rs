use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::BamStage;
use bijux_dna_domain_fastq::default_execution_tool_for_stage;
use bijux_dna_planner_bam::stage_api::default_tool_for_stage as default_bam_tool_for_stage;
use serde::Serialize;

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
use super::tool_probe_support::path_relative_to_repo;
use crate::commands::benchmark::local_micro_benchmark_report::DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH;
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_SCORING_PATH: &str = "configs/bench/local/stage-scoring.toml";
const STAGE_SCORING_REPORT_SCHEMA_VERSION: &str = "bijux.bench.readiness.stage_scoring.v1";
const STAGE_SCORING_CONFIG_HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-bench\n\
# purpose = Governed scoring policy for active benchmark stage comparisons.\n\
# authority = bijux-dna-bench\n\
# stability = evolving\n\
# last_updated = 2026-08-12\n\n";
const STAGE_SCORING_FULL_BENCHMARK_SOURCE: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
const STAGE_SCORING_MICRO_SOURCE: &str =
    "artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_REPORT.json";
const STAGE_SCORING_FAILURE_CLASS_SOURCE: &str =
    "artifacts/benchmark-smoke/readiness/failure-classification-all-domains.json";
const STAGE_SCORING_ACTIVE_STAGE_SOURCE: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-catalog.tsv";
const STAGE_SCORING_THRESHOLD_SOURCE: &str =
    "configs/bench/local/scientific-acceptance-thresholds.toml";

use bijux_dna_api::v1::api::bench::{validate_stage_scoring_config, STAGE_SCORING_SCHEMA_VERSION};
pub(crate) use bijux_dna_api::v1::api::bench::{
    StageScoringApplicability, StageScoringCompletion, StageScoringConfig, StageScoringCorrectness,
    StageScoringCorrectnessSignal, StageScoringDecisionMode, StageScoringDirection,
    StageScoringFailureClass, StageScoringFailureClassCatalogEntry, StageScoringFailureEffect,
    StageScoringMemory, StageScoringMissingBehavior, StageScoringRow, StageScoringRuntime,
    StageScoringScientificThreshold, StageScoringWeights,
};

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
    let body = toml::to_string_pretty(&config).context("serialize stage-scoring TOML")?;
    let rendered = format!("{STAGE_SCORING_CONFIG_HEADER}{body}");
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
                "blocked_external_access" => {
                    "the supported tool requires a checksum-admitted operator artifact before recommendation"
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

    validate_stage_scoring_config(config)
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
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, OnceLock};

    use super::{
        render_stage_scoring, StageScoringApplicability, StageScoringCorrectnessSignal,
        StageScoringDecisionMode, STAGE_SCORING_CONFIG_HEADER,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn artifact_tempdir(repo_root: &Path) -> tempfile::TempDir {
        let artifact_root = repo_root.join("artifacts");
        std::fs::create_dir_all(&artifact_root).expect("create test artifact root");
        tempfile::tempdir_in(artifact_root).expect("temporary artifact directory")
    }

    #[test]
    fn render_stage_scoring_covers_all_active_stages() {
        let _guard = test_lock().lock().expect("stage scoring unit lock");
        let repo_root = repo_root();
        let temp_dir = artifact_tempdir(&repo_root);
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report = render_stage_scoring(&repo_root, relative_output_path.clone())
            .expect("render stage scoring");
        let rendered = std::fs::read_to_string(&output_path).expect("read stage scoring config");

        assert_eq!(report.config_path, relative_output_path.to_string_lossy());
        assert!(rendered.starts_with(STAGE_SCORING_CONFIG_HEADER));
        assert_eq!(report.row_count, 71);
        assert_eq!(
            report.multi_tool_stage_count,
            report
                .rows
                .iter()
                .filter(|row| row.decision_mode == StageScoringDecisionMode::MultiToolRanking)
                .count()
        );
        assert_eq!(
            report.single_tool_stage_count,
            report.row_count - report.multi_tool_stage_count
        );
        assert_eq!(
            report.scientific_stage_count,
            report
                .rows
                .iter()
                .filter(|row| {
                    row.scientific_threshold.applicability == StageScoringApplicability::Required
                })
                .count()
        );
        assert_eq!(report.failure_class_count, 8);
        assert_eq!(report.domain_counts.get("fastq"), Some(&27));
        assert_eq!(report.domain_counts.get("bam"), Some(&24));
        assert_eq!(report.domain_counts.get("vcf"), Some(&20));
        assert!(report.rows.iter().all(|row| !row.report_section_ids.is_empty()));
    }

    #[test]
    fn render_stage_scoring_preserves_multi_tool_ranking_contracts() {
        let _guard = test_lock().lock().expect("stage scoring unit lock");
        let repo_root = repo_root();
        let temp_dir = artifact_tempdir(&repo_root);
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report =
            render_stage_scoring(&repo_root, relative_output_path).expect("render stage scoring");

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
        let temp_dir = artifact_tempdir(&repo_root);
        let output_path = temp_dir.path().join("stage-scoring.toml");
        let relative_output_path =
            output_path.strip_prefix(&repo_root).expect("relative output path").to_path_buf();
        let report =
            render_stage_scoring(&repo_root, relative_output_path).expect("render stage scoring");

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
