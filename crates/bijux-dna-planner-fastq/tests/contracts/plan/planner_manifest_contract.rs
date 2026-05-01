use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{
    build_plan_manifest, planner_refusal_from_message, ArtifactRole, ParameterResolutionTraceV1,
    PlanManifestBuildInputV1, PlanPolicy, PlannerParameterSourceV1, PlannerWarningCodeV1,
    PlannerWarningRecordV1, WorkflowInputArtifactV1, WorkflowManifestV1, WorkflowStageRequestV1,
};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::stage_api::default_tool_for_stage;
use bijux_dna_planner_fastq::{
    plan_fastq_stage_plans, plan_fastq_to_fastq__default__v1, DefaultPipelineOptions,
    FastqPipelineInputs, FastqPlanConfig, FastqPlanner, FastqStageToolsetBinding,
};
use insta::Settings;

fn snapshot_settings() -> Settings {
    let mut settings = Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings
}

fn snapshot_name(name: &str) -> String {
    format!("bijux-dna-planner-fastq__contracts__{name}")
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

fn default_inputs() -> FastqPipelineInputs {
    let pipeline =
        bijux_dna_planner_fastq::default_pipeline_spec(DefaultPipelineOptions::default());
    let stage_toolsets = pipeline
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
    FastqPipelineInputs {
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
    }
}

fn workflow_manifest(advisory_trim: bool) -> WorkflowManifestV1 {
    let mut manifest = WorkflowManifestV1::new("fastq", "fastq-to-fastq__default__v1");
    manifest.inputs = vec![WorkflowInputArtifactV1 {
        artifact_id: "reads".to_string(),
        role: ArtifactRole::Reads,
        path: PathBuf::from("reads_R1.fastq.gz"),
        layout: None,
        compression: None,
        format_id: Some("fastq.gz".to_string()),
    }];
    manifest.requested_stages = vec![
        WorkflowStageRequestV1 {
            stage_id: "fastq.validate_reads".to_string(),
            advisory_only: false,
        },
        WorkflowStageRequestV1 {
            stage_id: "fastq.trim_reads".to_string(),
            advisory_only: advisory_trim,
        },
    ];
    manifest
}

fn plan_manifest_payload(advisory_trim: bool) -> anyhow::Result<serde_json::Value> {
    let inputs = default_inputs();
    let graph = plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default())?;
    let pipeline =
        bijux_dna_planner_fastq::default_pipeline_spec(DefaultPipelineOptions::default());
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: inputs.policy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(pipeline.clone()),
        stage_bindings: Vec::new(),
        stage_toolsets: pipeline
            .ordered_nodes()
            .iter()
            .map(|node| FastqStageToolsetBinding {
                stage_id: node.stage_id.clone(),
                stage_instance_id: node.stage_instance_id.clone(),
                tools: vec![default_stage_tool(&node.stage_id)],
                reason: None,
                params: None,
            })
            .collect(),
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
    let stage_plans = plan_fastq_stage_plans(&config)?;
    let effective_parameters_by_step = stage_plans
        .iter()
        .map(|plan| {
            (
                plan.stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                plan.effective_params.clone(),
            )
        })
        .collect();
    let parameter_traces = stage_plans
        .iter()
        .filter_map(|plan| {
            let serde_json::Value::Object(map) = &plan.effective_params else {
                return None;
            };
            let (name, value) = map.iter().next()?;
            Some(ParameterResolutionTraceV1 {
                step_id: plan
                    .stage_instance_id
                    .as_ref()
                    .map_or_else(|| plan.stage_id.to_string(), std::string::ToString::to_string),
                stage_id: plan.stage_id.to_string(),
                parameter: name.clone(),
                source: PlannerParameterSourceV1::PlannerInferred,
                resolved_value: value.clone(),
                detail: "derived from stage effective params".to_string(),
            })
        })
        .collect();
    let warnings = if advisory_trim {
        vec![PlannerWarningRecordV1 {
            code: PlannerWarningCodeV1::AdvisoryStage,
            stage_id: Some("fastq.trim_reads".to_string()),
            message: "trim stage kept in advisory-only mode".to_string(),
        }]
    } else {
        Vec::new()
    };
    let manifest = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_manifest(advisory_trim),
        graph,
        stage_contract_refs: Vec::new(),
        effective_parameters_by_step,
        parameter_traces,
        refusal_records: Vec::new(),
        warning_records: warnings,
    })?;
    Ok(serde_json::to_value(manifest)?)
}

#[test]
fn fastq_happy_plan_manifest_snapshot_is_stable() -> anyhow::Result<()> {
    let _guard = snapshot_settings().bind_to_scope();
    insta::assert_json_snapshot!(
        snapshot_name("fastq_happy_plan_manifest"),
        bijux_dna_testkit::snapshot_normalize_json(&plan_manifest_payload(false)?)
    );
    Ok(())
}

#[test]
fn fastq_advisory_plan_manifest_snapshot_is_stable() -> anyhow::Result<()> {
    let _guard = snapshot_settings().bind_to_scope();
    insta::assert_json_snapshot!(
        snapshot_name("fastq_advisory_plan_manifest"),
        bijux_dna_testkit::snapshot_normalize_json(&plan_manifest_payload(true)?)
    );
    Ok(())
}

#[test]
fn fastq_refusal_snapshot_is_stable() {
    let _guard = snapshot_settings().bind_to_scope();
    let config = FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(bijux_dna_planner_fastq::default_pipeline_spec(
            DefaultPipelineOptions::default(),
        )),
        stage_bindings: Vec::new(),
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
    let error = FastqPlanner::plan_stage_plans(&config).expect_err("missing toolsets must fail");
    let refusal = planner_refusal_from_message(None, &error.to_string());
    let payload = serde_json::json!({
        "workflow_manifest": workflow_manifest(false),
        "refusal_records": [refusal],
    });
    insta::assert_json_snapshot!(
        snapshot_name("fastq_refusal_manifest"),
        bijux_dna_testkit::snapshot_normalize_json(&payload)
    );
}
