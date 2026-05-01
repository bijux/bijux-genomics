use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, PlanPolicy, StageIO, ToolConstraints};
use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};
use bijux_dna_stage_contract::{
    default_edges_for_stages, execution_step_from_stage_plan, ExecutionPlan, PlanDecisionReason,
    PlanEdge, PlannerContractV1, StagePlanV1,
};

fn trim_plan(instance_id: &str, tool_id: &str, output_id: &str) -> StagePlanV1 {
    StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_reads"),
        stage_instance_id: Some(StepId::new(instance_id)),
        stage_version: StageVersion(1),
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec![tool_id.to_string()] },
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
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn execution_plan_allows_duplicate_stage_ids_with_unique_instance_ids() {
    let left = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    let right = trim_plan("fastq.trim_reads.tool.cutadapt", "cutadapt", "trimmed_reads_cutadapt");
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_benchmark__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![left.clone(), right.clone()],
        vec![PlanEdge::new("fastq.trim_reads.tool.fastp", "fastq.trim_reads.tool.cutadapt")],
    )
    .expect("execution plan should allow repeated stage ids when node ids are unique");
    assert_eq!(plan.stages().len(), 2);
    assert!(plan.stages().iter().all(|stage| stage.stage_id.as_str() == "fastq.trim_reads"));
}

#[test]
fn default_edges_use_stage_instance_ids_when_present() {
    let left = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    let right = trim_plan("fastq.trim_reads.tool.cutadapt", "cutadapt", "trimmed_reads_cutadapt");
    let edges = default_edges_for_stages(&[left, right]);
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from(), "fastq.trim_reads.tool.fastp");
    assert_eq!(edges[0].to(), "fastq.trim_reads.tool.cutadapt");
}

#[test]
fn default_edges_prefer_artifact_bound_handoffs_when_stage_contracts_match() {
    let validate = StagePlanV1 {
        stage_id: StageId::from_static("fastq.validate_reads"),
        stage_instance_id: Some(StepId::new("fastq.validate_reads.fastqvalidator")),
        stage_version: StageVersion(1),
        tool_id: ToolId::new("fastqvalidator".to_string()),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["fastqvalidator".to_string()] },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::from_static("reads_r1"),
                PathBuf::from("reads_R1.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::from_static("validated_reads_manifest"),
                PathBuf::from("validated_reads_manifest.json"),
                ArtifactRole::StageReport,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };
    let trim = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_r1");
    let report = StagePlanV1 {
        stage_id: StageId::from_static("fastq.report_qc"),
        stage_instance_id: Some(StepId::new("fastq.report_qc.multiqc")),
        stage_version: StageVersion(1),
        tool_id: ToolId::new("multiqc".to_string()),
        tool_version: "test".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::from_static("validated_reads_manifest"),
                    PathBuf::from("validated_reads_manifest.json"),
                    ArtifactRole::StageReport,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("trimmed_reads_r1"),
                    PathBuf::from("trimmed_reads_r1.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                ),
            ],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("qc_report".to_string()),
                PathBuf::from("qc_report.json"),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };

    let edges = default_edges_for_stages(&[validate, trim, report]);

    assert_eq!(edges.len(), 3);
    assert_eq!(edges[0].from(), "fastq.trim_reads.tool.fastp");
    assert_eq!(edges[0].to(), "fastq.report_qc.multiqc");
    assert_eq!(edges[0].from_output_id(), Some("trimmed_reads_r1"));
    assert_eq!(edges[0].to_input_id(), Some("trimmed_reads_r1"));
    assert_eq!(edges[1].from(), "fastq.validate_reads.fastqvalidator");
    assert_eq!(edges[1].to(), "fastq.report_qc.multiqc");
    assert_eq!(edges[1].from_output_id(), Some("validated_reads_manifest"));
    assert_eq!(edges[1].to_input_id(), Some("validated_reads_manifest"));
    assert_eq!(edges[2].from(), "fastq.validate_reads.fastqvalidator");
    assert_eq!(edges[2].to(), "fastq.trim_reads.tool.fastp");
    assert_eq!(edges[2].from_output_id(), None);
}

#[test]
fn execution_plan_accepts_artifact_bound_edges() {
    let left = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
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
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
        resources: ToolConstraints::default(),
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
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
    assert_eq!(plan.edges()[0].from_output_id(), Some("trimmed_reads_fastp"));
    assert_eq!(plan.edges()[0].to_input_id(), Some("trimmed_reads_fastp"));
}

#[test]
fn execution_plan_sorts_artifact_bound_edges_by_binding_ids() {
    let mut left = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_z");
    left.io.outputs.push(ArtifactRef::required(
        ArtifactId::new("trimmed_reads_a".to_string()),
        PathBuf::from("trimmed_reads_a.fastq.gz"),
        ArtifactRole::TrimmedReads,
    ));
    let right = StagePlanV1 {
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("trimmed_reads_z".to_string()),
                    PathBuf::from("trimmed_reads_z.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("trimmed_reads_a".to_string()),
                    PathBuf::from("trimmed_reads_a.fastq.gz"),
                    ArtifactRole::TrimmedReads,
                ),
            ],
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
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
        resources: ToolConstraints::default(),
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };

    let plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![right, left],
        vec![
            PlanEdge::with_artifact_binding(
                "fastq.trim_reads.tool.fastp",
                "fastq.report_qc.multiqc",
                "trimmed_reads_z",
                "trimmed_reads_z",
            ),
            PlanEdge::with_artifact_binding(
                "fastq.trim_reads.tool.fastp",
                "fastq.report_qc.multiqc",
                "trimmed_reads_a",
                "trimmed_reads_a",
            ),
        ],
    )
    .expect("artifact-bound edges should validate and sort deterministically");

    assert_eq!(plan.edges()[0].from_output_id(), Some("trimmed_reads_a"));
    assert_eq!(plan.edges()[1].from_output_id(), Some("trimmed_reads_z"));
}

#[test]
fn execution_steps_inherit_stage_instance_identity() {
    let plan = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    let step = execution_step_from_stage_plan(&plan);
    assert_eq!(step.step_id.as_str(), "fastq.trim_reads.tool.fastp");
}

#[test]
fn execution_plan_validation_reports_unresolved_edge_nodes_without_panicking() {
    let valid_plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp")],
        Vec::new(),
    )
    .expect("fixture plan should be valid");

    let mut encoded = serde_json::to_value(&valid_plan).expect("serialize execution plan");
    encoded["edges"] = serde_json::json!([{
        "from": "fastq.trim_reads.tool.fastp",
        "to": "fastq.report_qc.multiqc"
    }]);
    let malformed: ExecutionPlan =
        serde_json::from_value(encoded).expect("deserialize malformed execution plan");
    let error = malformed
        .validate_strict(&bijux_dna_stage_contract::PlanValidationContext {
            allowed_id_catalog: None,
            allowed_tool_ids: None,
        })
        .expect_err("unknown edge targets must fail validation");

    assert!(error.to_string().contains("plan edge references unknown stage"));
}

#[test]
fn execution_plan_rejects_edges_with_empty_endpoints() {
    let error = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp")],
        vec![PlanEdge::new("", "fastq.trim_reads.tool.fastp")],
    )
    .expect_err("empty edge endpoints must fail validation");

    assert!(error.to_string().contains("plan edge has empty endpoint"));
}

#[test]
fn execution_plan_rejects_empty_artifact_bindings() {
    let left = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
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
        image: ContainerImageRefV1 { image: "bijux/test".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
        resources: ToolConstraints::default(),
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };

    let error = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![left, right],
        vec![PlanEdge::with_artifact_binding(
            "fastq.trim_reads.tool.fastp",
            "fastq.report_qc.multiqc",
            "",
            "trimmed_reads_fastp",
        )],
    )
    .expect_err("empty artifact bindings must fail validation");

    assert!(error.to_string().contains("has empty artifact binding"));
}

#[test]
fn execution_plan_rejects_duplicate_stage_output_artifacts() {
    let mut stage = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    stage.io.outputs.push(ArtifactRef::required(
        ArtifactId::new("trimmed_reads_fastp".to_string()),
        PathBuf::from("trimmed_reads_fastp.copy.fastq.gz"),
        ArtifactRole::TrimmedReads,
    ));

    let error = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![stage],
        Vec::new(),
    )
    .expect_err("duplicate stage outputs must fail validation");

    assert!(error.to_string().contains("duplicate output artifact trimmed_reads_fastp"));
}

#[test]
fn strict_validation_rejects_missing_command_template() {
    let mut stage = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    stage.command.template.clear();
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__trim_reads_qc__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![stage],
        Vec::new(),
    )
    .expect("non-strict plan construction allows incomplete command metadata");

    let error = plan
        .validate_strict(&bijux_dna_stage_contract::PlanValidationContext {
            allowed_id_catalog: None,
            allowed_tool_ids: None,
        })
        .expect_err("strict validation must require a command template");

    assert!(error.to_string().contains("missing command template"));
}

#[test]
fn planner_contract_trims_tool_version_projection() {
    let mut stage = trim_plan("fastq.trim_reads.tool.fastp", "fastp", "trimmed_reads_fastp");
    stage.tool_version = " 0.24.0 ".to_string();

    let contract = PlannerContractV1::from(&stage);

    assert_eq!(contract.tool_version.as_deref(), Some("0.24.0"));
}
