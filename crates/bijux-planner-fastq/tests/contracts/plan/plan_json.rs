use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_domain_fastq::{STAGE_TRIM, STAGE_VALIDATE_PRE};
use bijux_stage_contract::StagePlanV1;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-planner-fastq__{group}__{name}")
}

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

fn assert_snapshot(name: &str, plan: &StagePlanV1) -> Result<()> {
    let payload = serde_json::to_string_pretty(&plan)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(format!("{}.json", snapshot_name("contracts", name)));
    if std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        fs::write(&snapshot_path, payload)?;
        return Ok(());
    }
    let expected = fs::read_to_string(&snapshot_path)?;
    assert_eq!(payload, expected);
    Ok(())
}

#[test]
fn stage_plan_snapshots_are_stable() -> Result<()> {
    let r1 = Path::new("reads_R1.fastq.gz");
    let r2 = Path::new("reads_R2.fastq.gz");
    let out_dir = Path::new("out");

    let plan = bijux_planner_fastq::tool_adapters::fastq::trim::plan(
        &dummy_tool("fastp"),
        r1,
        out_dir,
        None,
        None,
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.trim", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::filter::plan_filter(
        &dummy_tool("seqkit"),
        r1,
        out_dir,
        &bijux_planner_fastq::tool_adapters::fastq::filter::FilterPlanOptions::default(),
    )?;
    assert_snapshot("stage__fastq__fastq.filter", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::merge::plan_merge(
        &dummy_tool("pear"),
        r1,
        r2,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.merge", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::validate_pre::plan(
        &dummy_tool("fastqvalidator"),
        r1,
        out_dir,
    );
    assert_snapshot("stage__fastq__fastq.validate_pre", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::screen::plan_screen(
        &dummy_tool("kraken2"),
        r1,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.screen", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::umi::plan_umi(
        &dummy_tool("umi_tools"),
        r1,
        r2,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.umi", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::correct::plan_correct(
        &dummy_tool("rcorrector"),
        r1,
        r2,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.correct", &plan)?;

    let preprocess_plan =
        bijux_planner_fastq::tool_adapters::stages::pre::plan_preprocess::PreprocessPlan {
            r1: r1.to_path_buf(),
            r2: Some(r2.to_path_buf()),
            stages: vec![
                STAGE_TRIM.as_str().to_string(),
                STAGE_VALIDATE_PRE.as_str().to_string(),
            ],
            enable_contaminant_removal: false,
        };
    let plan =
        bijux_planner_fastq::tool_adapters::stages::pre::plan_preprocess::plan_preprocess_stage(
            &preprocess_plan,
            &dummy_tool("planner"),
        );
    assert_snapshot("stage__fastq__fastq.preprocess", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::qc_post::plan_qc_post(
        &dummy_tool("multiqc"),
        r1,
        out_dir,
        std::collections::BTreeMap::new(),
        None,
    )?;
    assert_snapshot("stage__fastq__fastq.qc_post", &plan)?;

    let plan = bijux_planner_fastq::tool_adapters::fastq::stats_neutral::plan_stats_neutral(
        &dummy_tool("seqkit_stats"),
        r1,
        out_dir,
    )?;
    assert_snapshot("stage__fastq__fastq.stats_neutral", &plan)?;
    Ok(())
}
