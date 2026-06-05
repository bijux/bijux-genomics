use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    comparable_benchmark_stage_ids, comparable_tool_ids_for_stage,
    stage_comparable_metric_fields_for_stage, BamStage,
};
use bijux_dna_planner_bam::stage_api::default_tool_for_stage;
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_COMPARABLE_METRICS_PATH: &str =
    "target/bench-readiness/bam-comparable-metrics.tsv";
const BAM_COMPARABLE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_comparable_metrics.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamComparableMetricContractStatus {
    Declared,
    MissingSharedMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamComparableMetricsRow {
    pub(crate) stage_id: String,
    pub(crate) comparison_contract_status: BamComparableMetricContractStatus,
    pub(crate) tool_count: usize,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) default_tool_id: String,
    pub(crate) corpus_status: String,
    pub(crate) shared_metric_field_count: usize,
    pub(crate) shared_metric_fields: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamComparableMetricsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) comparable_stage_count: usize,
    pub(crate) multi_tool_stage_count: usize,
    pub(crate) comparable_tool_row_count: usize,
    pub(crate) row_count: usize,
    pub(crate) declared_stage_count: usize,
    pub(crate) missing_shared_metric_stage_count: usize,
    pub(crate) shared_metric_field_count: usize,
    pub(crate) rows: Vec<BamComparableMetricsRow>,
}

pub(crate) fn run_render_bam_comparable_metrics(
    args: &parse::BenchReadinessRenderBamComparableMetricsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_comparable_metrics(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_COMPARABLE_METRICS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_comparable_metrics(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamComparableMetricsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let corpus_status_by_stage = super::tool_serving_map::load_corpus_status_by_stage(repo_root)?;
    let summary = collect_bam_comparable_metric_rows(&corpus_status_by_stage)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_comparable_metrics_tsv(&summary.rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    Ok(BamComparableMetricsReport {
        schema_version: BAM_COMPARABLE_METRICS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        comparable_stage_count: summary.comparable_stage_count,
        multi_tool_stage_count: summary.rows.len(),
        comparable_tool_row_count: summary.comparable_tool_row_count,
        row_count: summary.rows.len(),
        declared_stage_count: summary.declared_stage_count,
        missing_shared_metric_stage_count: summary.missing_shared_metric_stage_count,
        shared_metric_field_count: summary
            .rows
            .iter()
            .map(|row| row.shared_metric_field_count)
            .sum(),
        rows: summary.rows,
    })
}

#[derive(Debug)]
struct ComparableMetricRowSummary {
    comparable_stage_count: usize,
    comparable_tool_row_count: usize,
    declared_stage_count: usize,
    missing_shared_metric_stage_count: usize,
    rows: Vec<BamComparableMetricsRow>,
}

fn collect_bam_comparable_metric_rows(
    corpus_status_by_stage: &std::collections::BTreeMap<String, String>,
) -> Result<ComparableMetricRowSummary> {
    let comparable_stage_ids = comparable_benchmark_stage_ids();
    let mut comparable_tool_row_count = 0;
    let mut rows = Vec::new();

    for stage_id in &comparable_stage_ids {
        let tool_ids = comparable_tool_ids_for_stage(stage_id)
            .into_iter()
            .map(|tool_id| tool_id.to_string())
            .collect::<Vec<_>>();
        comparable_tool_row_count += tool_ids.len();
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
        let shared_metric_fields = stage_comparable_metric_fields_for_stage(stage_id);
        let comparison_contract_status = comparable_metric_contract_status(&shared_metric_fields);
        let reason = comparable_metric_reason(
            stage_key,
            &corpus_status,
            &shared_metric_fields,
            comparison_contract_status,
        );

        rows.push(BamComparableMetricsRow {
            stage_id: stage_key.to_string(),
            comparison_contract_status,
            tool_count: tool_ids.len(),
            tool_ids,
            default_tool_id,
            corpus_status,
            shared_metric_field_count: shared_metric_fields.len(),
            shared_metric_fields,
            reason,
        });
    }

    rows.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));

    Ok(ComparableMetricRowSummary {
        comparable_stage_count: comparable_stage_ids.len(),
        comparable_tool_row_count,
        declared_stage_count: rows
            .iter()
            .filter(|row| row.comparison_contract_status == BamComparableMetricContractStatus::Declared)
            .count(),
        missing_shared_metric_stage_count: rows
            .iter()
            .filter(|row| {
                row.comparison_contract_status
                    == BamComparableMetricContractStatus::MissingSharedMetrics
            })
            .count(),
        rows,
    })
}

fn comparable_metric_contract_status(
    shared_metric_fields: &[String],
) -> BamComparableMetricContractStatus {
    if shared_metric_fields.is_empty() {
        BamComparableMetricContractStatus::MissingSharedMetrics
    } else {
        BamComparableMetricContractStatus::Declared
    }
}

fn comparable_metric_reason(
    stage_id: &str,
    corpus_status: &str,
    shared_metric_fields: &[String],
    status: BamComparableMetricContractStatus,
) -> String {
    match status {
        BamComparableMetricContractStatus::Declared => format!(
            "stage `{stage_id}` publishes governed shared comparable metrics `{}` for same-stage tool comparison while corpus routing remains `{corpus_status}`",
            shared_metric_fields.join(", ")
        ),
        BamComparableMetricContractStatus::MissingSharedMetrics => format!(
            "stage `{stage_id}` has multiple comparable tools but no governed shared comparable metrics while corpus routing remains `{corpus_status}`"
        ),
    }
}

fn render_bam_comparable_metrics_tsv(rows: &[BamComparableMetricsRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tcomparison_contract_status\ttool_count\ttool_ids\tdefault_tool_id\tcorpus_status\tshared_metric_field_count\tshared_metric_fields\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(comparison_contract_status_label(row.comparison_contract_status)),
            row.tool_count,
            sanitize_tsv(&row.tool_ids.join(",")),
            sanitize_tsv(&row.default_tool_id),
            sanitize_tsv(&row.corpus_status),
            row.shared_metric_field_count,
            sanitize_tsv(&row.shared_metric_fields.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn comparison_contract_status_label(status: BamComparableMetricContractStatus) -> &'static str {
    match status {
        BamComparableMetricContractStatus::Declared => "declared",
        BamComparableMetricContractStatus::MissingSharedMetrics => "missing_shared_metrics",
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        comparison_contract_status_label, render_bam_comparable_metrics,
        BAM_COMPARABLE_METRICS_SCHEMA_VERSION, DEFAULT_BAM_COMPARABLE_METRICS_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_comparable_metrics_reports_governed_multi_tool_rows() {
        let root = repo_root();
        let report =
            render_bam_comparable_metrics(&root, PathBuf::from(DEFAULT_BAM_COMPARABLE_METRICS_PATH))
                .expect("render BAM comparable metrics");

        assert_eq!(report.schema_version, BAM_COMPARABLE_METRICS_SCHEMA_VERSION);
        assert_eq!(report.comparable_stage_count, 15);
        assert_eq!(report.multi_tool_stage_count, 15);
        assert_eq!(report.comparable_tool_row_count, 40);
        assert_eq!(report.row_count, 15);
        assert_eq!(report.declared_stage_count, 15);
        assert_eq!(report.missing_shared_metric_stage_count, 0);
        assert_eq!(report.shared_metric_field_count, 65);

        assert!(report.rows.iter().all(|row| {
            comparison_contract_status_label(row.comparison_contract_status) == "declared"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.validate"
                && row.tool_count == 3
                && row.tool_ids
                    == [
                        "bamtools".to_string(),
                        "bedtools".to_string(),
                        "samtools".to_string(),
                    ]
                && row.default_tool_id == "samtools"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
                && row.shared_metric_fields
                    == [
                        "validation_status".to_string(),
                        "validation_errors".to_string(),
                        "validation_warnings".to_string(),
                        "input_bam_identity".to_string(),
                    ]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.coverage"
                && row.tool_count == 3
                && row.default_tool_id == "mosdepth"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
                && row.shared_metric_fields
                    == [
                        "mean_depth".to_string(),
                        "breadth_1x".to_string(),
                        "covered_bases".to_string(),
                        "observed_region_count".to_string(),
                        "region_ids".to_string(),
                    ]
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "bam.damage"
                && row.tool_count == 6
                && row.default_tool_id == "mapdamage2"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
                && row.shared_metric_fields
                    == [
                        "terminal_c_to_t_5p".to_string(),
                        "terminal_g_to_a_3p".to_string(),
                        "damage_signal".to_string(),
                        "runtime_s".to_string(),
                        "memory_mb".to_string(),
                    ]
        }));
    }
}
