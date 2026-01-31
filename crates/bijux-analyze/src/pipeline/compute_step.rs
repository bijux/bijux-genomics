//! Owner: bijux-analyze
//! Compute step for analyze pipeline.

use std::collections::BTreeMap;

use anyhow::Result;
use bijux_core::FactsRowV1;

use crate::decision::score::{build_rankings, RankInput, RankingEntry};
use crate::model::FactRow;
use crate::{AnalyzeMode, AnalyzeOptions};

use super::validate_step::ValidatedFacts;

#[derive(Debug)]
pub(crate) struct AnalysisCore {
    pub(crate) facts_rows: Vec<FactsRowV1>,
    pub(crate) rankings: Option<std::collections::BTreeMap<String, Vec<RankingEntry>>>,
    pub(crate) base_dir: std::path::PathBuf,
}

pub(crate) fn compute_core(
    validated: ValidatedFacts,
    options: &AnalyzeOptions,
) -> Result<AnalysisCore> {
    let facts_rows: Vec<FactsRowV1> = validated
        .facts
        .rows
        .iter()
        .map(FactRow::to_facts_row_v1)
        .collect();

    let rankings = if matches!(options.mode, AnalyzeMode::Rank { .. }) {
        Some(build_rankings(&rank_inputs_from_facts(
            &validated.facts.rows,
        ))?)
    } else {
        None
    };

    Ok(AnalysisCore {
        facts_rows,
        rankings,
        base_dir: validated.base_dir,
    })
}

#[allow(clippy::cast_precision_loss)]
fn rank_inputs_from_facts(facts: &[FactRow]) -> Vec<RankInput> {
    let mut by_tool: BTreeMap<String, Vec<&FactRow>> = BTreeMap::new();
    for row in facts {
        by_tool.entry(row.tool_id.clone()).or_default().push(row);
    }
    by_tool
        .into_iter()
        .map(|(tool, rows)| {
            let n = rows.len() as f64;
            let runtime_s = rows.iter().map(|row| row.runtime_s).sum::<f64>() / n.max(1.0);
            let memory_mb = rows.iter().map(|row| row.memory_mb).sum::<f64>() / n.max(1.0);
            let read_retention = retention_from_rows(rows.iter().copied());
            let base_retention = base_retention_from_rows(rows.iter().copied());
            RankInput {
                tool,
                runtime_s,
                memory_mb,
                read_retention,
                base_retention,
                error_reduction_proxy: None,
            }
        })
        .collect()
}

#[allow(clippy::cast_precision_loss)]
fn retention_from_rows<'a, I: Iterator<Item = &'a FactRow>>(rows: I) -> Option<f64> {
    let mut values = Vec::new();
    for row in rows {
        if let (Some(ri), Some(ro)) = (row.reads_in, row.reads_out) {
            if ri > 0 {
                values.push(ro as f64 / ri as f64);
            }
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

#[allow(clippy::cast_precision_loss)]
fn base_retention_from_rows<'a, I: Iterator<Item = &'a FactRow>>(rows: I) -> Option<f64> {
    let mut values = Vec::new();
    for row in rows {
        if let (Some(bi), Some(bo)) = (row.bases_in, row.bases_out) {
            if bi > 0 {
                values.push(bo as f64 / bi as f64);
            }
        }
    }
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}
