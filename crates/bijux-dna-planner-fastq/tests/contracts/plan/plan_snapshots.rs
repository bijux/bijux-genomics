#![allow(clippy::expect_used)]

/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec, PlanPolicy};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::stage_api::default_tool_for_stage;
use bijux_dna_planner_fastq::{
    apply_tool_overrides, plan_fastq_to_fastq__default__v1, DefaultPipelineOptions,
    FastqPipelineInputs, FastqPlanConfig, FastqPlanner, FastqStageBinding,
    FastqStageToolsetBinding,
};

fn snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings
}

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-planner-fastq__{group}__{name}")
}

fn default_stage_tool(stage: &str) -> ToolExecutionSpecV1 {
    let stage_id = StageId::new(stage);
    let tool_id = default_tool_for_stage(&stage_id)
        .map_or_else(|| "echo".to_string(), |tool| tool.to_string());
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test".to_string(),
            digest: Some("sha256:plan".to_string()),
        },
        command: CommandSpecV1 { template: vec!["echo".to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn fastq_plan_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let tool_trim = ToolExecutionSpecV1 {
        tool_id: ToolId::from_static("fastp"),
        tool_version: "0.23.4".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/fastp".to_string(),
            digest: Some("sha256:fastp".to_string()),
        },
        command: CommandSpecV1 { template: vec!["fastp".to_string()] },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 2,
        },
    };
    let tool_validate = ToolExecutionSpecV1 {
        tool_id: ToolId::from_static("fastqvalidator"),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/fastqvalidator".to_string(),
            digest: Some("sha256:fastqvalidator".to_string()),
        },
        command: CommandSpecV1 { template: vec!["fastqvalidator".to_string()] },
        resources: ToolConstraints {
            runtime: "short".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: None,
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: None,
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.validate_reads".to_string(),
                to: "fastq.trim_reads".to_string(),
                from_output_id: None,
                to_input_id: None,
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: None,
                tool: tool_validate,
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: None,
                tool: tool_trim,
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: None,
        out_dir: PathBuf::from("out"),
        allow_planned: false,
    };
    let plan = FastqPlanner::plan(&config).expect("plan");
    let name = snapshot_name("contracts", "fastq_plan_snapshot");
    let json = serde_json::to_value(&plan).expect("serialize plan");
    insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn tool_override_precedence_is_stable() {
    let mut base = BTreeMap::new();
    base.insert("fastq.trim_reads".to_string(), "fastp".to_string());
    let mut profile = BTreeMap::new();
    profile.insert("fastq.trim_reads".to_string(), "cutadapt".to_string());
    let mut cli = BTreeMap::new();
    cli.insert("fastq.trim_reads".to_string(), "bbduk".to_string());
    let mut forced = BTreeMap::new();
    forced.insert("fastq.trim_reads".to_string(), "trimmomatic".to_string());
    let merged = apply_tool_overrides(base, profile, cli, forced);
    assert_eq!(merged.get("fastq.trim_reads").map(String::as_str), Some("trimmomatic"));
}

#[test]
fn default_pipeline_plan_snapshot_is_stable() {
    let _guard = snapshot_settings().bind_to_scope();
    let pipeline =
        bijux_dna_planner_fastq::default_pipeline_spec(DefaultPipelineOptions::default());
    let stage_toolsets: Vec<FastqStageToolsetBinding> = pipeline
        .ordered_nodes()
        .iter()
        .map(|node| FastqStageToolsetBinding {
            stage_id: node.stage_id.clone(),
            stage_instance_id: node.stage_instance_id.clone(),
            tools: vec![default_stage_tool(&node.stage_id)],
            reason: None,
            params: None,
        })
        .collect();
    let inputs = FastqPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        stage_toolsets,
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: None,
        out_dir: PathBuf::from("out"),
    };
    let plan =
        plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default()).expect("plan");
    let name = snapshot_name("contracts", "fastq_default_pipeline_plan");
    let json = serde_json::to_value(&plan).expect("serialize plan");
    insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}
