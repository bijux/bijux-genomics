use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use bijux_analyze::{load::load_facts, report::write_run_report_from_facts};

fn fixture_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    Ok(repo_root
        .join("target")
        .join("test-fixtures")
        .join("report"))
}

#[test]
fn report_build_is_within_budget() -> Result<()> {
    if std::env::var("BIJUX_PERF_SKIP").is_ok() {
        return Ok(());
    }
    let root = fixture_root()?;
    let facts_path = root.join("happy").join("facts.jsonl");
    let facts = load_facts(&facts_path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    let start = Instant::now();
    let _ = write_run_report_from_facts(&root.join("perf_budget"), &facts)?;
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "report build exceeded budget: {:.2}s",
        elapsed.as_secs_f64()
    );
    // Ensure output exists to avoid optimizer skipping.
    let report_path = root.join("perf_budget").join("report.json");
    assert!(fs::metadata(report_path).is_ok());
    Ok(())
}
