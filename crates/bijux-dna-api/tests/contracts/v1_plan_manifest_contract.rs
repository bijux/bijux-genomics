use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_api::v1::api::{dry_run, plan, DryRunRequest, PlanRequest};
use bijux_dna_core::contract::{
    ArtifactRef, ArtifactRole, StageIO, ToolConstraints, WorkflowInputArtifactV1,
    WorkflowManifestV1, WorkflowStageRequestV1,
};
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

fn minimal_graph() -> ExecutionGraph {
    let step = ExecutionStep {
        step_id: StepId::from_static("fastq.validate_reads"),
        stage_id: StageId::from_static("fastq.validate_reads"),
        image: ContainerImageRefV1 {
            image: "example/validator:1".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        command: CommandSpecV1 { template: vec!["echo".to_string(), "hello".to_string()] },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads"),
                PathBuf::from("reads.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("validated"),
                PathBuf::from("validated.fastq.gz"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner.test",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::<ExecutionEdge>::new(),
    )
    .unwrap_or_else(|err| panic!("graph build failed: {err}"))
}

fn workflow_manifest() -> WorkflowManifestV1 {
    let mut manifest = WorkflowManifestV1::new("fastq", "fastq-to-fastq__default__v1");
    manifest.inputs = vec![WorkflowInputArtifactV1 {
        artifact_id: "reads".to_string(),
        role: ArtifactRole::Reads,
        path: PathBuf::from("reads.fastq.gz"),
        layout: None,
        compression: None,
        format_id: Some("fastq.gz".to_string()),
    }];
    manifest.requested_stages = vec![WorkflowStageRequestV1 {
        stage_id: "fastq.validate_reads".to_string(),
        advisory_only: false,
    }];
    manifest
}

#[test]
fn plan_response_materializes_workflow_and_plan_manifests() -> Result<()> {
    let graph = minimal_graph();
    let request = PlanRequest {
        graph,
        profile_id: "fastq-to-fastq__default__v1".to_string(),
        workflow_manifest: Some(workflow_manifest()),
        stage_plans: Vec::new(),
        parameter_traces: Vec::new(),
        planner_refusals: Vec::new(),
        planner_warnings: Vec::new(),
        compare_against: None,
    };
    let response = plan(request)?;
    assert_eq!(response.workflow_manifest.domain, "fastq");
    assert_eq!(response.plan_manifest.domain, "fastq");
    assert_eq!(response.plan_manifest.ordered_steps.len(), 1);
    assert_eq!(
        response.plan_manifest.workflow_fingerprint,
        response.workflow_manifest.fingerprint()?
    );
    Ok(())
}

#[test]
fn dry_run_writes_plan_manifest_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let request = DryRunRequest {
        graph: minimal_graph(),
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;
    let plan_manifest_path = temp.path().join("plan_manifest.json");
    assert!(plan_manifest_path.exists());
    let manifest_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&response.manifest_path)?)?;
    assert!(manifest_json["output_artifacts"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .any(|artifact| artifact["kind"] == "plan_manifest"));
    Ok(())
}
