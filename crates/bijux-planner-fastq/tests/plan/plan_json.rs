use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_stage_contract::StagePlanJsonV1 as StagePlanJson;

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
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

fn snapshot_path(name: &str) -> Result<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    Ok(Path::new(&manifest_dir)
        .join("tests")
        .join("snapshots")
        .join(name))
}

fn assert_snapshot(name: &str, plan: &bijux_stage_contract::StagePlanV1) -> Result<()> {
    let plan_json = StagePlanJson::from_plan(plan);
    let rendered = serde_json::to_string_pretty(&plan_json)?;
    let path = snapshot_path(name)?;
    if std::env::var("UPDATE_SNAPSHOTS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        bijux_infra::write_bytes(path, rendered)?;
        return Ok(());
    }
    let snapshot = fs::read_to_string(path)?;
    assert_eq!(rendered.trim(), snapshot.trim());
    Ok(())
}

#[test]
fn plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::trim::plan(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.trim.json", &plan)
}

#[test]
fn filter_plan_json_is_emitted_and_stable() -> Result<()> {
    let options = bijux_planner_fastq::tool_adapters::fastq::filter::FilterPlanOptions::default();
    let plan = bijux_planner_fastq::tool_adapters::fastq::filter::plan_filter(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        &options,
    )?;
    assert_snapshot("stage__fastq__fastq.filter.json", &plan)
}

#[test]
fn merge_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::merge::plan_merge(
        &dummy_tool("pear"),
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_snapshot("stage__fastq__fastq.merge.json", &plan)
}

#[test]
fn validate_pre_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::validate_pre::plan(
        &dummy_tool("fastqc"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    );
    assert_snapshot("stage__fastq__fastq.validate_pre.json", &plan)
}

#[test]
fn screen_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::screen::plan_screen(
        &dummy_tool("kraken2"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_snapshot("stage__fastq__fastq.screen.json", &plan)
}

#[test]
fn umi_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::umi::plan_umi(
        &dummy_tool("umi_tools"),
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_snapshot("stage__fastq__fastq.umi.json", &plan)
}

#[test]
fn correct_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::correct::plan_correct(
        &dummy_tool("rcorrector"),
        std::path::Path::new("reads_r1.fastq.gz"),
        std::path::Path::new("reads_r2.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_snapshot("stage__fastq__fastq.correct.json", &plan)
}

#[test]
fn preprocess_plan_json_is_emitted_and_stable() -> Result<()> {
    let args = bijux_planner_fastq::args::BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: None,
        r1: std::path::PathBuf::from("reads.fastq.gz"),
        r2: None,
        out: std::path::PathBuf::from("artifacts"),
        strict: false,
        auto: false,
        objective: bijux_core::contract::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: false,
        replicates: 1,
        jobs: 1,
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
        enable_correct: false,
    };
    let _ = bijux_planner_fastq::preprocess_decisions(&args);
    let pipeline = bijux_core::contract::PipelineSpec {
        stages: vec![
            "fastq.validate_pre".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim".to_string(),
            "fastq.filter".to_string(),
            "fastq.stats_neutral".to_string(),
            "fastq.qc_post".to_string(),
        ],
    };
    let plan = bijux_planner_fastq::plan_preprocess(&args, pipeline);
    let plan = bijux_planner_fastq::tool_adapters::fastq::preprocess::plan_preprocess_stage(
        &plan,
        &dummy_tool("pipeline"),
    );
    assert_snapshot("stage__fastq__fastq.preprocess.json", &plan)
}

#[test]
fn qc_post_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::qc_post::plan_qc_post(
        &dummy_tool("fastqc"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        std::collections::BTreeMap::new(),
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.qc_post.json", &plan)
}

#[test]
fn stats_neutral_plan_json_is_emitted_and_stable() -> Result<()> {
    let plan = bijux_planner_fastq::tool_adapters::fastq::stats_neutral::plan_stats_neutral(
        &dummy_tool("seqkit"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_snapshot("stage__fastq__fastq.stats_neutral.json", &plan)
}
