use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, PlanPolicy, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{
    default_edges_for_stages, execution_step_from_stage_plan, ExecutionPlan, PlanDecisionReason,
    PlanEdge, StagePlanV1,
};

fn trim_plan(instance_id: &str, tool_id: &str, output_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_reads"),
        stage_instance_id: Some(StepId::new(instance_id)),
        stage_version: StageVersion(1),
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec![tool_id.to_string()],
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                PathBuf::from("reads_R1.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new(output_id.to_string()),
                PathBuf::from(format!("{output_id}.fastq.gz")),
                ArtifactRole::TrimmedReads,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn execution_plan_allows_duplicate_stage_ids_with_unique_instance_ids() {
    let left = trim_plan(
        "fastq.trim_reads.tool.fastp",
        "fastp",
        "trimmed_reads_fastp",
    );
    let right = trim_plan(
        "fastq.trim_reads.tool.cutadapt",
        "cutadapt",
        "trimmed_reads_cutadapt",
    );
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_benchmark__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![left.clone(), right.clone()],
        vec![PlanEdge::new(
            "fastq.trim_reads.tool.fastp",
            "fastq.trim_reads.tool.cutadapt",
        )],
    )
    .expect("execution plan should allow repeated stage ids when node ids are unique");
    assert_eq!(plan.stages().len(), 2);
    assert!(plan
        .stages()
        .iter()
        .all(|stage| stage.stage_id.as_str() == "fastq.trim_reads"));
}

#[test]
fn default_edges_use_stage_instance_ids_when_present() {
    let left = trim_plan(
        "fastq.trim_reads.tool.fastp",
        "fastp",
        "trimmed_reads_fastp",
    );
    let right = trim_plan(
        "fastq.trim_reads.tool.cutadapt",
        "cutadapt",
        "trimmed_reads_cutadapt",
    );
    let edges = default_edges_for_stages(&[left, right]);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from(), "fastq.trim_reads.tool.fastp");
    assert_eq!(edges[0].to(), "fastq.trim_reads.tool.cutadapt");
}

#[test]
fn execution_plan_accepts_artifact_bound_edges() {
    let left = trim_plan(
        "fastq.trim_reads.tool.fastp",
        "fastp",
        "trimmed_reads_fastp",
    );
    let right = StagePlanV1 {
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads_fastp".to_string()),
                PathBuf::from("trimmed_reads_fastp.fastq.gz"),
                ArtifactRole::TrimmedReads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("qc_report".to_string()),
                PathBuf::from("qc_report.json"),
                ArtifactRole::ReportJson,
            )],
        },
        stage_id: StageId::from_static("fastq.report_qc"),
        stage_instance_id: Some(StepId::new("fastq.report_qc.multiqc")),
        stage_version: StageVersion(1),
        tool_id: ToolId::new("multiqc".to_string()),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["multiqc".to_string()],
        },
        resources: ToolConstraints::default(),
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: PlanDecisionReason::default(),
    };
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![left, right],
        vec![PlanEdge::with_artifact_binding(
            "fastq.trim_reads.tool.fastp",
            "fastq.report_qc.multiqc",
            "trimmed_reads_fastp",
            "trimmed_reads_fastp",
        )],
    )
    .expect("artifact-bound plan edge should validate");
    assert_eq!(
        plan.edges()[0].from_output_id(),
        Some("trimmed_reads_fastp")
    );
    assert_eq!(plan.edges()[0].to_input_id(), Some("trimmed_reads_fastp"));
}

#[test]
fn execution_steps_inherit_stage_instance_identity() {
    let plan = trim_plan(
        "fastq.trim_reads.tool.fastp",
        "fastp",
        "trimmed_reads_fastp",
    );
    let step = execution_step_from_stage_plan(&plan);
    assert_eq!(step.step_id.as_str(), "fastq.trim_reads.tool.fastp");
}
