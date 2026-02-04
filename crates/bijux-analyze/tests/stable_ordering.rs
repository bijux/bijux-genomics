use std::fs;

use anyhow::Result;
use bijux_analyze::decision::score::{build_rankings, RankInput};
use bijux_analyze::report::write_run_report_from_facts;
use bijux_core::FactsRowV1;

fn row(tool: &str, runtime: f64, reads_in: u64, reads_out: u64) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: tool.to_string(),
        tool_version: "0.1".to_string(),
        image_digest: None,
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        params_hash: "p".to_string(),
        input_hash: "i".to_string(),
        output_hashes: vec![],
        runtime_s: runtime,
        memory_mb: 10.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }
}

#[allow(clippy::cast_precision_loss)]
fn rank_inputs(rows: &[FactsRowV1]) -> Vec<RankInput> {
    let mut by_tool = std::collections::BTreeMap::new();
    for row in rows {
        by_tool
            .entry(row.tool_id.clone())
            .or_insert_with(Vec::new)
            .push(row);
    }
    by_tool
        .into_iter()
        .map(|(tool, rows)| {
            let n = rows.len() as f64;
            let runtime_s = rows.iter().map(|row| row.runtime_s).sum::<f64>() / n.max(1.0);
            let memory_mb = rows.iter().map(|row| row.memory_mb).sum::<f64>() / n.max(1.0);
            let read_retention = rows
                .iter()
                .find_map(|row| match (row.reads_in, row.reads_out) {
                    (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
                    _ => None,
                });
            RankInput {
                tool,
                runtime_s,
                memory_mb,
                read_retention,
                base_retention: None,
                error_reduction_proxy: None,
            }
        })
        .collect()
}

#[test]
fn reordered_facts_produce_identical_report_and_rankings() -> Result<()> {
    let rows = vec![row("alpha", 1.0, 100, 90), row("beta", 2.0, 100, 80)];
    let mut rows_reordered = rows.clone();
    rows_reordered.reverse();

    let dir_a = bijux_infra::temp_dir("bijux")?;
    let dir_b = bijux_infra::temp_dir("bijux")?;
    let defaults = serde_json::json!({
        "pipeline_id": "fastq-to-fastq__default__v1",
        "tools": {},
        "params": {},
        "thresholds": {},
        "tool_provenance": {},
        "param_provenance": {},
        "assumptions": [],
        "citations": {},
    });
    bijux_infra::write_bytes(
        dir_a.path().join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;
    bijux_infra::write_bytes(
        dir_b.path().join("defaults_ledger.json"),
        serde_json::to_vec_pretty(&defaults)?,
    )?;

    let report_a = write_run_report_from_facts(dir_a.path(), &rows)?;
    let report_b = write_run_report_from_facts(dir_b.path(), &rows_reordered)?;

    let json_a = fs::read_to_string(report_a)?;
    let json_b = fs::read_to_string(report_b)?;
    assert_eq!(json_a, json_b);

    let rankings_a = build_rankings(&rank_inputs(&rows))?;
    let rankings_b = build_rankings(&rank_inputs(&rows_reordered))?;
    let rendered_a = serde_json::to_string_pretty(&rankings_a)?;
    let rendered_b = serde_json::to_string_pretty(&rankings_b)?;
    assert_eq!(rendered_a, rendered_b);

    Ok(())
}
