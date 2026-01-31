//! Owner: bijux-analyze
//! Canonical analyze pipeline entrypoint.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use bijux_core::selection::{objective_spec, Objective};
use bijux_core::FactsRowV1;

use crate::decision::compare::compare_runs;
use crate::decision::score::{build_rankings, RankInput};
use crate::facts::write_run_summary_json;
use crate::load::{load_facts, load_facts_parquet, load_run_summary, AnalyzeError};
use crate::model::{FactRow, FactTable};
use crate::report::write_run_report_from_facts;
use crate::{AnalyzeInput, AnalyzeMode, AnalyzeOutput, AnalyzeSources};

pub fn analyze_run_pipeline(input: &AnalyzeInput) -> Result<AnalyzeOutput> {
    let mut output = AnalyzeOutput {
        run_id: input.run_id.clone(),
        report_json: None,
        report_html: None,
        summary_json: None,
        compare_json: None,
        ranking_json: None,
        decision_trace_json: None,
    };

    if let AnalyzeMode::Compare {
        ref run_a,
        ref run_b,
    } = input.options.mode
    {
        let objective = objective_spec(Objective::Balanced);
        let comparison = compare_runs(Path::new(run_a), Path::new(run_b), &objective)?;
        let base_dir = std::env::current_dir().context("resolve current_dir")?;
        let path = base_dir.join("compare.json");
        std::fs::write(&path, serde_json::to_vec_pretty(&comparison)?)
            .context("write compare.json")?;
        output.compare_json = Some(path);
        return Ok(output);
    }

    let facts = load_facts_from_sources(&input.sources)?;
    let base_dir = base_dir_for_sources(&input.sources);

    let validated = validate_facts(&facts)?;
    let normalized = normalize_facts(validated);
    let aggregated = aggregate_facts(normalized);
    let aggregated_rows: Vec<FactsRowV1> = aggregated
        .rows
        .iter()
        .map(FactRow::to_facts_row_v1)
        .collect();

    if matches!(
        input.options.mode,
        AnalyzeMode::Summary | AnalyzeMode::Report
    ) {
        let summary_path = base_dir.join("run_summary.json");
        write_run_summary_json(&summary_path, &aggregated_rows)?;
        output.summary_json = Some(summary_path);
    }

    if matches!(input.options.mode, AnalyzeMode::Report) {
        let report_path = write_run_report_from_facts(&base_dir, &aggregated_rows)?;
        output.report_json = Some(report_path);
    }

    if matches!(input.options.mode, AnalyzeMode::Rank { .. }) {
        let rankings = build_rankings(&rank_inputs_from_facts(&aggregated.rows))?;
        let rank_path = base_dir.join("ranking.json");
        std::fs::write(&rank_path, serde_json::to_vec_pretty(&rankings)?)
            .context("write ranking.json")?;
        output.ranking_json = Some(rank_path);
    }

    Ok(output)
}

fn load_facts_from_sources(sources: &AnalyzeSources) -> Result<Vec<FactsRowV1>> {
    match sources {
        AnalyzeSources::FactsJsonl(path) => load_facts(path).map_err(map_load_error),
        AnalyzeSources::FactsParquet(path) => load_facts_parquet(path).map_err(map_load_error),
        AnalyzeSources::RunSummaryJson(path) => {
            let _summary = load_run_summary(path).map_err(map_load_error)?;
            Err(anyhow!(
                "run summary input does not include facts: {}",
                path.display()
            ))
        }
        AnalyzeSources::RunIndexSqlite(path) => Err(anyhow!(
            "run index sqlite not yet wired for analyze_run: {}",
            path.display()
        )),
    }
}

fn base_dir_for_sources(sources: &AnalyzeSources) -> PathBuf {
    let path = match sources {
        AnalyzeSources::FactsJsonl(path)
        | AnalyzeSources::FactsParquet(path)
        | AnalyzeSources::RunSummaryJson(path)
        | AnalyzeSources::RunIndexSqlite(path) => path,
    };
    path.parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn validate_facts(facts: &[FactsRowV1]) -> Result<FactTable> {
    FactTable::from_facts(facts).map_err(|err| anyhow!(err))
}

fn normalize_facts(facts: FactTable) -> FactTable {
    facts
}

fn aggregate_facts(facts: FactTable) -> FactTable {
    facts
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

fn map_load_error(err: AnalyzeError) -> anyhow::Error {
    anyhow!(err)
}
