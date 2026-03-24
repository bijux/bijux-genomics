use std::path::Path;

use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy;
use bijux_dna_planner_fastq::tool_adapters::fastq::merge_pairs::{
    plan_merge_with_options, MergePlanOptions,
};

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: format!("bijuxdna/{tool_id}"),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), tool_id.to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 2,
        },
    }
}

#[test]
fn pear_merge_plan_maps_overlap_and_min_length() -> Result<()> {
    let plan = plan_merge_with_options(
        &tool("pear"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: Some(9),
            merge_overlap: Some(24),
            min_length: Some(120),
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        },
    )?;

    assert_eq!(plan.command.template[0], "bash");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("'pear' '-f' 'reads_R1.fastq.gz'"));
    assert!(script.contains("'pear' '-f' 'reads_R1.fastq.gz' '-r' 'reads_R2.fastq.gz' '-o' 'out/pear' '-j' '9'"));
    assert!(script.contains("'24'"));
    assert!(script.contains("'120'"));
    assert!(script.contains("\"merge_overlap\": 24"));
    assert!(script.contains("\"min_len\": 120"));
    assert!(script.contains("\"threads\": 9"));
    assert_eq!(plan.resources.threads, 9);
    assert_eq!(plan.params["merge_overlap"], serde_json::json!(24));
    assert_eq!(plan.params["min_length"], serde_json::json!(120));
    assert_eq!(plan.params["threads"], serde_json::json!(9));
    Ok(())
}

#[test]
fn flash2_merge_plan_rejects_min_length_policy() {
    let err = plan_merge_with_options(
        &tool("flash2"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: Some(6),
            merge_overlap: None,
            min_length: Some(80),
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        },
    )
    .expect_err("flash2 should reject unsupported min_length");

    assert!(err
        .to_string()
        .contains("merge planning does not yet map min_length for flash2"));
}

#[test]
fn bbmerge_merge_plan_maps_threads() -> Result<()> {
    let plan = plan_merge_with_options(
        &tool("bbmerge"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: Some(8),
            merge_overlap: Some(20),
            min_length: None,
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        },
    )?;

    let script = &plan.command.template[2];
    assert!(script.contains("'threads=8'"));
    assert!(script.contains("'minoverlap=20'"));
    assert!(script.contains("\"threads\": 8"));
    assert_eq!(plan.resources.threads, 8);
    Ok(())
}

#[test]
fn flash2_merge_plan_maps_threads() -> Result<()> {
    let plan = plan_merge_with_options(
        &tool("flash2"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: Some(6),
            merge_overlap: Some(15),
            min_length: None,
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        },
    )?;

    let script = &plan.command.template[2];
    assert!(script.contains("'flash2' '-o' 'flash2' '-d' 'out' '-t' '6'"));
    assert!(script.contains("'15'"));
    assert!(script.contains("\"threads\": 6"));
    assert_eq!(plan.resources.threads, 6);
    Ok(())
}

#[test]
fn leehom_merge_plan_rejects_unmerged_pair_outputs() {
    let err = plan_merge_with_options(
        &tool("leehom"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: None,
            merge_overlap: None,
            min_length: None,
            unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
        },
    )
    .expect_err("leehom should reject governed unmerged pair outputs");

    assert!(err
        .to_string()
        .contains("merge planning cannot emit governed unmerged pair artifacts for leehom"));
}

#[test]
fn vsearch_merge_plan_omits_unmerged_outputs_when_requested() -> Result<()> {
    let plan = plan_merge_with_options(
        &tool("vsearch"),
        Path::new("reads_R1.fastq.gz"),
        Path::new("reads_R2.fastq.gz"),
        Path::new("out"),
        &MergePlanOptions {
            threads: Some(11),
            merge_overlap: Some(18),
            min_length: Some(90),
            unmerged_read_policy: UnmergedReadPolicy::OmitUnmergedPairs,
        },
    )?;

    let output_names = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["merged_reads", "report_json"]);
    assert_eq!(plan.params["unmerged_reads_r1"], serde_json::Value::Null);
    assert_eq!(plan.params["unmerged_reads_r2"], serde_json::Value::Null);

    let script = &plan.command.template[2];
    assert!(script.contains("'--threads' '11'"));
    assert!(!script.contains("--fastqout_notmerged_fwd"));
    assert!(!script.contains("--fastqout_notmerged_rev"));
    assert!(script.contains("\"unmerged_read_policy\": \"omit_unmerged_pairs\""));
    assert!(script.contains("\"threads\": 11"));
    assert_eq!(plan.resources.threads, 11);
    Ok(())
}
