use std::fs;

use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId(tool.to_string()),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::trim::plan(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        None,
        None,
        None,
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
    let options = bijux_stages_fastq::fastq::filter::FilterPlanOptions::default();
    let plan = bijux_stages_fastq::fastq::filter::plan_filter(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        &options,
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
        &dummy_tool("pear"),
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
        &dummy_tool("fastqc"),
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
        &dummy_tool("kraken2"),
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
        &dummy_tool("umi_tools"),
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
        &dummy_tool("rcorrector"),
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
        objective: bijux_core::selection::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        replicates: 1,
        ci_bootstrap: None,
        adapter_bank_preset: None,
        adapter_bank: Some("preset:best_practice_adna".to_string()),
        adapter_bank_file: None,
        enable_adapters: Vec::new(),
        disable_adapters: Vec::new(),
        polyx_preset: None,
        contaminant_preset: None,
        enable_contaminant_removal: false,
        no_qc_post: false,
        force_merge: false,
    };
    let plan = bijux_stages_fastq::fastq::preprocess::plan_preprocess(&args);
    let plan = bijux_stages_fastq::fastq::preprocess::plan_preprocess_stage(
        &plan,
        &dummy_tool("pipeline"),
    );
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
        &dummy_tool("fastqc"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        std::collections::BTreeMap::new(),
        None,
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

#[test]
fn stats_neutral_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::stats_neutral::plan_stats_neutral(
        &dummy_tool("seqkit"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    let plan_json = bijux_stages_fastq::StagePlanJson::from_plan(&plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let snapshot_path =
        std::path::Path::new(&manifest_dir).join("tests/snapshots/fastq_stats_neutral_plan.json");
    let snapshot = fs::read_to_string(&snapshot_path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}
