use std::fs;

use anyhow::Result;

#[test]
fn plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages::fastq::trim::plan(
        "fastp",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_trim_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn filter_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages::fastq::filter::plan_filter(
        "fastp",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_filter_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
