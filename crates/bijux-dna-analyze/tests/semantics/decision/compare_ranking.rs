use anyhow::Result;
use bijux_dna_analyze::{compare::compare_runs, ranking::build_rankings, ranking::RankInput};
use bijux_dna_core::contract::objective_spec;
use bijux_dna_core::contract::Objective;
use bijux_dna_testkit::snapshot_normalize_json;
use insta::Settings;
/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-analyze__{group}__{name}")
}

#[test]
fn compare_and_ranking_snapshot() -> Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("compare_ranking")
        .join("default");
    let run_a = root.join("run_a");
    let run_b = root.join("run_b");
    bijux_dna_infra::ensure_dir(run_a.join("summary"))?;
    bijux_dna_infra::ensure_dir(run_b.join("summary"))?;

    bijux_dna_infra::write_bytes(
        run_a.join("summary").join("metrics_deltas.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "runtime_s": 1.0,
            "read_retention": 0.95,
        }))?,
    )?;
    bijux_dna_infra::write_bytes(
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
    let rendered = serde_json::to_string_pretty(&snapshot_normalize_json(&payload))?;
    let name = snapshot_name("semantics", "compare_ranking");
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.bind(|| {
        insta::assert_snapshot!(name, bijux_dna_testkit::snapshot_normalize_text(&rendered));
    });
    Ok(())
}
