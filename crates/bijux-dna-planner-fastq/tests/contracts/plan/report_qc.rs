use std::collections::BTreeMap;
use std::path::Path;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, ToolConstraints,
    ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::{
    qc_post::{QcAggregationEngine, QcAggregationScope},
    PairedMode,
};
use bijux_dna_planner_fastq::{compose_fastq_stage_bindings, FastqStageBinding};
use bijux_dna_stage_contract::PlanDecisionReason;

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["echo".to_string(), tool_id.to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn binding(stage_id: &str, tool_id: &str) -> FastqStageBinding {
    FastqStageBinding {
        stage_id: stage_id.to_string(),
        stage_instance_id: None,
        tool: tool(tool_id),
        reason: Some(PlanDecisionReason::default()),
        params: None,
    }
}

#[test]
fn report_qc_can_plan_from_governed_qc_artifacts() -> anyhow::Result<()> {
    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
            &tool("multiqc"),
            &[
                ArtifactRef::required(
                    ArtifactId::from_static("qc_json"),
                    Path::new("profile_reads/qc.json").to_path_buf(),
                    ArtifactRole::MetricsJson,
                ),
                ArtifactRef::required(
                    ArtifactId::from_static("adapter_report"),
                    Path::new("detect_adapters/adapter_report.json").to_path_buf(),
                    ArtifactRole::ReportJson,
                ),
            ],
            Path::new("out"),
            BTreeMap::new(),
            PairedMode::SingleEnd,
            QcAggregationEngine::Multiqc,
            QcAggregationScope::GovernedQcArtifacts,
            None,
            None,
        )?;

    assert_eq!(plan.io.inputs.len(), 2);
    assert!(plan.io.outputs.iter().any(|artifact| {
        artifact.name.as_str() == "governed_qc_inputs_manifest"
            && artifact.path == Path::new("out/governed_qc_inputs_manifest.json")
    }));
    assert_eq!(plan.params["qc_input_count"], serde_json::json!(2));
    assert_eq!(
        plan.params["qc_input_paths"],
        serde_json::json!(["profile_reads/qc.json", "detect_adapters/adapter_report.json"])
    );
    assert_eq!(
        plan.effective_params["aggregation_scope"],
        serde_json::json!("governed_qc_artifacts")
    );
    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    assert!(plan.command.template[2].contains("profile_reads/qc.json"));
    assert!(plan.command.template[2].contains("detect_adapters/adapter_report.json"));
    Ok(())
}

#[test]
fn compose_routes_governed_qc_artifacts_into_report_qc() -> anyhow::Result<()> {
    let plans = compose_fastq_stage_bindings(
        &[
            binding("fastq.detect_adapters", "fastqc"),
            binding("fastq.profile_reads", "seqkit_stats"),
            binding("fastq.report_qc", "multiqc"),
        ],
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(Path::new("out").join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    let report_plan = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.report_qc")
        .expect("report_qc stage");
    assert!(report_plan
        .io
        .outputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest"));
    assert!(report_plan
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "fastq.profile_reads.tool.seqkit_stats.qc_json"));
    assert!(report_plan.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.detect_adapters.tool.fastqc.adapter_evidence_dir"
    }));
    assert_eq!(
        report_plan.effective_params["aggregation_scope"],
        serde_json::json!("governed_qc_artifacts")
    );
    Ok(())
}

#[test]
fn compose_rejects_report_qc_without_governed_upstream_artifacts() {
    let error = compose_fastq_stage_bindings(
        &[binding("fastq.report_qc", "multiqc")],
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(Path::new("out").join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("report_qc should require governed QC artifacts");

    assert!(error.to_string().contains("requires governed upstream QC artifacts"));
}

#[test]
fn compose_routes_reference_screen_reports_into_report_qc() -> anyhow::Result<()> {
    let plans = compose_fastq_stage_bindings(
        &[
            binding("fastq.index_reference", "bowtie2_build"),
            binding("fastq.deplete_host", "bowtie2"),
            binding("fastq.report_qc", "multiqc"),
        ],
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        Some(Path::new("reference.fa")),
        None,
        |binding, _r1, _r2| {
            Ok(Path::new("out").join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    let report_plan = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.report_qc")
        .expect("report_qc stage");
    assert!(report_plan.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.deplete_host.tool.bowtie2.host_depletion_report_json"
    }));
    Ok(())
}

#[test]
fn compose_routes_cleanup_and_length_reports_into_report_qc() -> anyhow::Result<()> {
    let plans = compose_fastq_stage_bindings(
        &[
            binding("fastq.profile_read_lengths", "seqkit_stats"),
            binding("fastq.filter_low_complexity", "prinseq"),
            binding("fastq.report_qc", "multiqc"),
        ],
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(Path::new("out").join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    let report_plan = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.report_qc")
        .expect("report_qc stage");
    assert!(report_plan.io.inputs.iter().any(|artifact| {
        artifact.name.as_str()
            == "fastq.profile_read_lengths.tool.seqkit_stats.length_distribution_json"
    }));
    assert!(report_plan.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.filter_low_complexity.tool.prinseq.filter_report_json"
    }));
    Ok(())
}

#[test]
fn essential_qc_workflow_routes_validation_trim_and_profile_artifacts_into_report_qc(
) -> anyhow::Result<()> {
    let plans = compose_fastq_stage_bindings(
        &[
            binding("fastq.validate_reads", "fastqvalidator"),
            binding("fastq.detect_adapters", "fastqc"),
            binding("fastq.trim_reads", "fastp"),
            binding("fastq.profile_reads", "seqkit_stats"),
            binding("fastq.report_qc", "multiqc"),
        ],
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        Path::new("reads_R1.fastq.gz"),
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(Path::new("out").join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    assert_eq!(
        plans.iter().map(|plan| plan.stage_id.as_str()).collect::<Vec<_>>(),
        vec![
            "fastq.validate_reads",
            "fastq.detect_adapters",
            "fastq.trim_reads",
            "fastq.profile_reads",
            "fastq.report_qc",
        ]
    );

    let report_plan = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.report_qc")
        .expect("report_qc stage");
    let input_names =
        report_plan.io.inputs.iter().map(|artifact| artifact.name.as_str()).collect::<Vec<_>>();
    assert!(input_names.iter().any(|name| {
        name.starts_with("fastq.validate_reads") && name.ends_with(".validation_report")
    }));
    assert!(input_names
        .iter()
        .any(|name| { name == &"fastq.detect_adapters.tool.fastqc.adapter_evidence_dir" }));
    assert!(input_names
        .iter()
        .any(|name| name.starts_with("fastq.trim_reads") && name.ends_with(".report_json")));
    assert!(input_names
        .iter()
        .any(|name| { name == &"fastq.profile_reads.tool.seqkit_stats.qc_json" }));
    assert!(report_plan
        .io
        .outputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest"));
    assert_eq!(report_plan.effective_params["aggregation_engine"], serde_json::json!("multiqc"));
    Ok(())
}

#[test]
fn graph_report_qc_inherits_branch_qc_lineage_from_upstream_nodes() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-report-qc-branch-lineage")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nACGT\n+\n####\n")?;

    let plan =
        bijux_dna_planner_fastq::FastqPlanner::plan(&bijux_dna_planner_fastq::FastqPlanConfig {
            pipeline_id: "fastq-to-fastq__branch_lineage__v1".to_string(),
            policy: PlanPolicy::PreferAccuracy,
            selection_objective: bijux_dna_core::contract::Objective::Balanced,
            pipeline_spec: Some(PipelineSpec::graph(
                vec![
                    PipelineNodeSpec {
                        stage_id: "fastq.validate_reads".to_string(),
                        stage_instance_id: Some("fastq.validate_reads.validation".to_string()),
                    },
                    PipelineNodeSpec {
                        stage_id: "fastq.trim_reads".to_string(),
                        stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                    },
                    PipelineNodeSpec {
                        stage_id: "fastq.trim_reads".to_string(),
                        stage_instance_id: Some("fastq.trim_reads.bbduk_branch".to_string()),
                    },
                    PipelineNodeSpec {
                        stage_id: "fastq.report_qc".to_string(),
                        stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    },
                ],
                vec![
                    PipelineEdgeSpec {
                        from: "fastq.validate_reads.validation".to_string(),
                        to: "fastq.trim_reads.fastp_branch".to_string(),
                        from_output_id: None,
                        to_input_id: None,
                    },
                    PipelineEdgeSpec {
                        from: "fastq.validate_reads.validation".to_string(),
                        to: "fastq.trim_reads.bbduk_branch".to_string(),
                        from_output_id: None,
                        to_input_id: None,
                    },
                    PipelineEdgeSpec {
                        from: "fastq.trim_reads.fastp_branch".to_string(),
                        to: "fastq.report_qc.aggregate".to_string(),
                        from_output_id: None,
                        to_input_id: None,
                    },
                    PipelineEdgeSpec {
                        from: "fastq.trim_reads.bbduk_branch".to_string(),
                        to: "fastq.report_qc.aggregate".to_string(),
                        from_output_id: None,
                        to_input_id: None,
                    },
                ],
            )),
            stage_bindings: vec![
                bijux_dna_planner_fastq::FastqStageBinding {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validation".to_string()),
                    tool: tool("fastqvalidator"),
                    reason: None,
                    params: None,
                },
                bijux_dna_planner_fastq::FastqStageBinding {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                    tool: tool("fastp"),
                    reason: None,
                    params: None,
                },
                bijux_dna_planner_fastq::FastqStageBinding {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.bbduk_branch".to_string()),
                    tool: tool("bbduk"),
                    reason: None,
                    params: None,
                },
                bijux_dna_planner_fastq::FastqStageBinding {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tool: tool("multiqc"),
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
            r1,
            r2: None,
            reference_fasta: None,
            out_dir: temp.path().join("out"),
            allow_planned: false,
        })?;

    let report_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.report_qc.aggregate")
        .expect("report_qc aggregate step");
    assert!(report_step.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.validate_reads.validation.validation_report"
    }));
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| { artifact.name.as_str() == "fastq.trim_reads.fastp_branch.report_json" }));
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| { artifact.name.as_str() == "fastq.trim_reads.bbduk_branch.report_json" }));
    Ok(())
}
