use std::fs;

use anyhow::Result;

#[test]
fn plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::trim::plan(
        "fastp",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
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
    let plan = bijux_stages_fastq::fastq::filter::plan_filter(
        "fastp",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_filter_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn merge_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::merge::plan_merge(
        "pear",
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_merge_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn validate_pre_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::validate_pre::plan(
        "fastqc",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    );
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_validate_pre_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn screen_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::screen::plan_screen(
        "kraken2",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_screen_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn umi_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::umi::plan_umi(
        "umi_tools",
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_umi_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn correct_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::correct::plan_correct(
        "rcorrector",
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_correct_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn preprocess_plan_json_is_emitted_and_stable() -> Result<()> {
    let args = bijux_stages_fastq::args::BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        r1: std::path::PathBuf::from("reads.fastq.gz"),
        r2: None,
        out: std::path::PathBuf::from("artifacts"),
        strict: false,
        auto: false,
        objective: bijux_analyze::selection::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        adapter_preset: "default_adna".to_string(),
        adapter_bank: None,
        enable_adapters: Vec::new(),
        disable_adapters: Vec::new(),
    };
    let plan = bijux_stages_fastq::fastq::preprocess::plan_preprocess(&args);
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_preprocess_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn qc_post_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::qc_post::plan_qc_post(
        "fastqc",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_qc_post_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
