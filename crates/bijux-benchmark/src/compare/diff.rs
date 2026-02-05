//! Owner: bijux-benchmark
//! Typed diffs and effect sizes between runs.
//! Must not perform IO.

use std::collections::BTreeMap;

use anyhow::Result;

use crate::compare::stratify::CompareStratum;
use crate::model::{BenchmarkSummary, MetricSummary};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricDiff {
    pub metric_id: String,
    pub absolute: f64,
    pub relative: Option<f64>,
    pub practical: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CompareReport {
    pub suite_a: String,
    pub suite_b: String,
    pub diffs: Vec<MetricDiff>,
    pub strata: Vec<CompareStratum>,
}

pub fn compare_summaries(
    summary_a: &BenchmarkSummary,
    summary_b: &BenchmarkSummary,
) -> Result<CompareReport> {
    let mut diffs = Vec::new();
    let mut strata = Vec::new();

    let mut index_b: BTreeMap<(String, String, String, String), &crate::model::SummaryRow> =
        BTreeMap::new();
    for row in &summary_b.rows {
        index_b.insert(
            (
                row.dataset_id.clone(),
                row.stage_id.clone(),
                row.tool_id.clone(),
                row.params_hash.clone(),
            ),
            row,
        );
    }

    for row_a in &summary_a.rows {
        let key = (
            row_a.dataset_id.clone(),
            row_a.stage_id.clone(),
            row_a.tool_id.clone(),
            row_a.params_hash.clone(),
        );
        let Some(row_b) = index_b.get(&key) else {
            continue;
        };
        strata.push(CompareStratum {
            dataset_id: row_a.dataset_id.clone(),
            dataset_class: row_a.dataset_class.clone(),
            read_layout: row_a.read_layout.clone(),
            stage_id: row_a.stage_id.clone(),
            tool_id: row_a.tool_id.clone(),
            params_hash: row_a.params_hash.clone(),
        });

        diffs.extend(metric_diffs(&row_a.runtime, &row_b.runtime, 0.05));
        diffs.extend(metric_diffs(&row_a.memory, &row_b.memory, 0.05));

        let mut map_b: BTreeMap<&str, &MetricSummary> = BTreeMap::new();
        for metric in &row_b.metrics {
            map_b.insert(metric.metric_id.as_str(), metric);
        }
        for metric_a in &row_a.metrics {
            if let Some(metric_b) = map_b.get(metric_a.metric_id.as_str()) {
                diffs.extend(metric_diffs(metric_a, metric_b, 0.05));
            }
        }
    }

    diffs.sort_by(|a, b| a.metric_id.cmp(&b.metric_id));
    Ok(CompareReport {
        suite_a: summary_a.suite_id.clone(),
        suite_b: summary_b.suite_id.clone(),
        diffs,
        strata,
    })
}

fn metric_diffs(a: &MetricSummary, b: &MetricSummary, practical_threshold: f64) -> Vec<MetricDiff> {
    let mut diffs = Vec::new();
    let a_val = a.stats.median;
    let b_val = b.stats.median;
    let absolute = b_val - a_val;
    let relative = if a_val.abs() > f64::EPSILON {
        Some(absolute / a_val)
    } else {
        None
    };
    let practical = absolute.abs() >= practical_threshold;
    diffs.push(MetricDiff {
        metric_id: a.metric_id.clone(),
        absolute,
        relative,
        practical,
    });
    diffs
}
