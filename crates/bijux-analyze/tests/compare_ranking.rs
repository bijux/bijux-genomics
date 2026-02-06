use std::path::PathBuf;

use anyhow::Result;
use bijux_analyze::{compare::compare_runs, ranking::build_rankings, ranking::RankInput};
use bijux_core::contract::Objective;
use bijux_core::selection::objective_spec;

#[test]
fn compare_and_ranking_snapshot() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("compare_ranking");
    let run_a = root.join("run_a");
    let run_b = root.join("run_b");
    bijux_infra::ensure_dir(run_a.join("summary"))?;
    bijux_infra::ensure_dir(run_b.join("summary"))?;

    bijux_infra::write_bytes(
        run_a.join("summary").join("metrics_deltas.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime_s": 1.0,
            "read_retention": 0.95,
        }))?,
    )?;
    bijux_infra::write_bytes(
        run_b.join("summary").join("metrics_deltas.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime_s": 2.0,
            "read_retention": 0.90,
        }))?,
    )?;

    let objective = objective_spec(Objective::Balanced);
    let comparison = compare_runs(&run_a, &run_b, &objective)?;

    let rankings = build_rankings(&[
        RankInput {
            tool: "alpha".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
            read_retention: Some(0.95),
            base_retention: Some(0.95),
            error_reduction_proxy: Some(0.1),
        },
        RankInput {
            tool: "beta".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
            read_retention: Some(0.95),
            base_retention: Some(0.95),
            error_reduction_proxy: Some(0.1),
        },
    ])?;

    let payload = serde_json::json!({
        "comparison": comparison,
        "rankings": rankings,
    });
    let rendered = serde_json::to_string_pretty(&payload)?;
    insta::assert_snapshot!("compare_ranking", rendered);
    Ok(())
}
