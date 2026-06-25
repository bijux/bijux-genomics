use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    comparable_benchmark_stage_ids, comparable_tool_ids_for_stage,
    stage_comparable_metric_contracts_for_stage, BamComparableMetricContract,
    BamScientificInsufficiencyPolicy, BamScientificPassDirection, BamScientificToleranceKind,
    BamStage,
};
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH: &str =
    "benchmarks/readiness/bam/BAM_SCIENCE_THRESHOLDS_READY.json";
const BAM_SCIENCE_THRESHOLDS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_science_thresholds_ready.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamScienceThresholdStatus {
    Declared,
    MissingThresholds,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct BamScienceThresholdMetricRow {
    pub(crate) metric_name: String,
    pub(crate) meaning: String,
    pub(crate) pass_direction: Option<BamScientificPassDirection>,
    pub(crate) tolerance_kind: Option<BamScientificToleranceKind>,
    pub(crate) tolerance_value: Option<f64>,
    pub(crate) insufficiency_policy: Option<BamScientificInsufficiencyPolicy>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct BamScienceThresholdStageRow {
    pub(crate) stage_id: String,
    pub(crate) threshold_status: BamScienceThresholdStatus,
    pub(crate) tool_count: usize,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) default_tool_id: String,
    pub(crate) corpus_status: String,
    pub(crate) metric_count: usize,
    pub(crate) missing_threshold_metric_names: Vec<String>,
    pub(crate) metrics: Vec<BamScienceThresholdMetricRow>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamScienceThresholdsReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) comparable_stage_count: usize,
    pub(crate) stage_row_count: usize,
    pub(crate) governed_metric_count: usize,
    pub(crate) threshold_declared_stage_count: usize,
    pub(crate) missing_threshold_stage_count: usize,
    pub(crate) threshold_declared_metric_count: usize,
    pub(crate) missing_threshold_metric_count: usize,
    pub(crate) rows: Vec<BamScienceThresholdStageRow>,
}

pub(crate) fn run_render_bam_science_thresholds_ready(
    args: &parse::BenchReadinessRenderBamScienceThresholdsReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_science_thresholds_ready(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_science_thresholds_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamScienceThresholdsReadyReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let corpus_status_by_stage = super::tool_serving_map::load_corpus_status_by_stage(repo_root)?;
    let summary = collect_bam_science_threshold_rows(&corpus_status_by_stage)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = BamScienceThresholdsReadyReport {
        schema_version: BAM_SCIENCE_THRESHOLDS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        comparable_stage_count: summary.comparable_stage_count,
        stage_row_count: summary.rows.len(),
        governed_metric_count: summary.threshold_declared_metric_count,
        threshold_declared_stage_count: summary.threshold_declared_stage_count,
        missing_threshold_stage_count: summary.missing_threshold_stage_count,
        threshold_declared_metric_count: summary.threshold_declared_metric_count,
        missing_threshold_metric_count: summary.missing_threshold_metric_count,
        rows: summary.rows,
    };
    fs::write(&output_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(report)
}

#[derive(Debug)]
struct ScienceThresholdRowSummary {
    comparable_stage_count: usize,
    threshold_declared_stage_count: usize,
    missing_threshold_stage_count: usize,
    threshold_declared_metric_count: usize,
    missing_threshold_metric_count: usize,
    rows: Vec<BamScienceThresholdStageRow>,
}

fn collect_bam_science_threshold_rows(
    corpus_status_by_stage: &std::collections::BTreeMap<String, String>,
) -> Result<ScienceThresholdRowSummary> {
    let comparable_stage_ids = comparable_benchmark_stage_ids();
    let mut threshold_declared_stage_count = 0;
    let mut missing_threshold_stage_count = 0;
    let mut threshold_declared_metric_count = 0;
    let mut missing_threshold_metric_count = 0;
    let mut rows = Vec::new();

    for stage_id in &comparable_stage_ids {
        let tool_ids = comparable_tool_ids_for_stage(stage_id)
            .into_iter()
            .map(|tool_id| tool_id.to_string())
            .collect::<Vec<_>>();
        if tool_ids.len() < 2 {
            continue;
        }

        let stage_key = stage_id.as_str();
        let corpus_status = corpus_status_by_stage.get(stage_key).cloned().ok_or_else(|| {
            anyhow!("BAM local corpus compatibility report is missing stage `{stage_key}`")
        })?;
        let stage = BamStage::try_from(stage_key)
            .with_context(|| format!("resolve BAM comparable stage `{stage_key}`"))?;
        let default_tool_id = default_tool_for_stage(stage).to_string();
        let metric_contracts = stage_comparable_metric_contracts_for_stage(stage_id);
        let missing_threshold_metric_names = metric_contracts
            .iter()
            .filter(|metric| metric.scientific_threshold.is_none())
            .map(|metric| metric.name.clone())
            .collect::<Vec<_>>();
        let threshold_status = if missing_threshold_metric_names.is_empty() {
            threshold_declared_stage_count += 1;
            BamScienceThresholdStatus::Declared
        } else {
            missing_threshold_stage_count += 1;
            BamScienceThresholdStatus::MissingThresholds
        };

        let mut metrics = Vec::with_capacity(metric_contracts.len());
        for metric in &metric_contracts {
            if metric.scientific_threshold.is_some() {
                threshold_declared_metric_count += 1;
            } else {
                missing_threshold_metric_count += 1;
            }
            metrics.push(metric_row(metric));
        }

        let reason = science_threshold_reason(
            stage_key,
            &corpus_status,
            threshold_status,
            &missing_threshold_metric_names,
            &metric_contracts,
        );
        rows.push(BamScienceThresholdStageRow {
            stage_id: stage_key.to_string(),
            threshold_status,
            tool_count: tool_ids.len(),
            tool_ids,
            default_tool_id,
            corpus_status,
            metric_count: metrics.len(),
            missing_threshold_metric_names,
            metrics,
            reason,
        });
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    Ok(ScienceThresholdRowSummary {
        comparable_stage_count: comparable_stage_ids.len(),
        threshold_declared_stage_count,
        missing_threshold_stage_count,
        threshold_declared_metric_count,
        missing_threshold_metric_count,
        rows,
    })
}

fn metric_row(metric: &BamComparableMetricContract) -> BamScienceThresholdMetricRow {
    let threshold = metric.scientific_threshold.as_ref();
    BamScienceThresholdMetricRow {
        metric_name: metric.name.clone(),
        meaning: metric.meaning.clone(),
        pass_direction: threshold.map(|value| value.pass_direction),
        tolerance_kind: threshold.map(|value| value.tolerance_kind),
        tolerance_value: threshold.map(|value| value.tolerance_value),
        insufficiency_policy: threshold.map(|value| value.insufficiency_policy),
    }
}

fn science_threshold_reason(
    stage_id: &str,
    corpus_status: &str,
    status: BamScienceThresholdStatus,
    missing_threshold_metric_names: &[String],
    metric_contracts: &[BamComparableMetricContract],
) -> String {
    match status {
        BamScienceThresholdStatus::Declared => format!(
            "stage `{stage_id}` governs scientific threshold semantics for BAM comparable metrics `{}` while corpus routing remains `{corpus_status}`",
            metric_contracts
                .iter()
                .map(|metric| metric.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        BamScienceThresholdStatus::MissingThresholds => format!(
            "stage `{stage_id}` is missing scientific threshold semantics for BAM comparable metrics `{}` while corpus routing remains `{corpus_status}`",
            missing_threshold_metric_names.join(", ")
        ),
    }
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

    use bijux_dna_domain_bam::{
        BamScientificInsufficiencyPolicy, BamScientificPassDirection, BamScientificToleranceKind,
    };

    use super::{
        render_bam_science_thresholds_ready, BAM_SCIENCE_THRESHOLDS_READY_SCHEMA_VERSION,
        DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_science_thresholds_ready_reports_governed_metric_thresholds() {
        let root = repo_root();
        let report = render_bam_science_thresholds_ready(
            &root,
            PathBuf::from(DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH),
        )
        .expect("render BAM science thresholds");

        assert_eq!(report.schema_version, BAM_SCIENCE_THRESHOLDS_READY_SCHEMA_VERSION);
        assert_eq!(report.comparable_stage_count, 15);
        assert_eq!(report.stage_row_count, 15);
        assert_eq!(report.governed_metric_count, 51);
        assert_eq!(report.threshold_declared_stage_count, 15);
        assert_eq!(report.missing_threshold_stage_count, 0);
        assert_eq!(report.threshold_declared_metric_count, 51);
        assert_eq!(report.missing_threshold_metric_count, 0);

        let damage =
            report.rows.iter().find(|row| row.stage_id == "bam.damage").expect("damage row");
        assert_eq!(damage.metric_count, 3);
        assert!(damage.missing_threshold_metric_names.is_empty());
        assert!(damage.metrics.iter().any(|metric| {
            metric.metric_name == "damage_signal"
                && metric.pass_direction == Some(BamScientificPassDirection::ExactMatch)
                && metric.tolerance_kind == Some(BamScientificToleranceKind::ExactMatch)
                && metric.tolerance_value == Some(0.0)
        }));

        let validate =
            report.rows.iter().find(|row| row.stage_id == "bam.validate").expect("validate row");
        assert_eq!(validate.metric_count, 3);
        assert!(validate.metrics.iter().any(|metric| {
            metric.metric_name == "validation_errors"
                && metric.tolerance_value == Some(1.0)
                && metric.insufficiency_policy
                    == Some(BamScientificInsufficiencyPolicy::RefuseStageComparison)
        }));
    }
}
