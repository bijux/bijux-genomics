#![allow(clippy::expect_used)]

use std::collections::{BTreeMap, HashSet};

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::stage_api::default_tool_for_stage;
use bijux_dna_planner_fastq::{
    compose_fastq_stage_bindings, default_pipeline_spec, DefaultPipelineOptions, FastqStageBinding,
    FastqStageParameters, FilterReadsStageParams, StageArtifactInputBinding,
    StageArtifactInputPolicy,
};
use bijux_dna_stage_contract::{default_edges_for_stages, ExecutionPlan, PlanValidationContext};

fn tool_for_stage(stage: &str) -> ToolExecutionSpecV1 {
    let stage_id = StageId::new(stage);
    let tool_id = default_tool_for_stage(&stage_id)
        .map_or_else(|| "fastp".to_string(), |tool| tool.to_string());
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["echo".to_string(), stage.to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn binding_for_stage(stage: &str) -> FastqStageBinding {
    FastqStageBinding {
        stage_id: stage.to_string(),
        stage_instance_id: None,
        tool: tool_for_stage(stage),
        reason: None,
        params: None,
    }
}

#[test]
fn fastq_plan_validates_against_contracts() -> anyhow::Result<()> {
    let pipeline = default_pipeline_spec(DefaultPipelineOptions::default());
    let stages = pipeline.ordered_stage_ids();
    let bindings = stages.iter().map(|stage| binding_for_stage(stage)).collect::<Vec<_>>();
    let temp = bijux_dna_infra::temp_dir("fastq-plan-handshake")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plans = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;
    let edges = default_edges_for_stages(&plans);
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        bijux_dna_planner_fastq::PLANNER_VERSION,
        PlanPolicy::PreferAccuracy,
        plans,
        edges,
    )?;
    let allowed_id_catalog = stages.into_iter().collect::<HashSet<_>>();
    let allowed_tool_ids = bindings
        .into_iter()
        .map(|binding| binding.tool.tool_id.to_string())
        .collect::<HashSet<_>>();
    let context = PlanValidationContext {
        allowed_id_catalog: Some(&allowed_id_catalog),
        allowed_tool_ids: Some(&allowed_tool_ids),
    };
    plan.validate_strict(&context)?;
    Ok(())
}

#[test]
fn reference_guided_plan_validates_index_to_depletion_flow() -> anyhow::Result<()> {
    let stages = ["fastq.index_reference".to_string(), "fastq.deplete_host".to_string()];
    let bindings = stages.iter().map(|stage| binding_for_stage(stage)).collect::<Vec<_>>();
    let temp = bijux_dna_infra::temp_dir("fastq-plan-reference-handshake")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

    let plans = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        Some(&reference),
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )?;

    assert_eq!(plans[0].stage_id.as_str(), "fastq.index_reference");
    assert_eq!(plans[1].stage_id.as_str(), "fastq.deplete_host");
    assert_eq!(plans[1].io.inputs[1].path, plans[0].io.outputs[0].path);
    Ok(())
}

#[test]
fn reference_guided_plan_rejects_incompatible_index_backend() -> anyhow::Result<()> {
    let bindings = vec![
        FastqStageBinding {
            stage_id: "fastq.index_reference".to_string(),
            stage_instance_id: None,
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::new("star"),
                tool_version: "99.99.99+fixture".to_string(),
                image: ContainerImageRefV1 {
                    image: "bijux/dummy:latest".to_string(),
                    digest: None,
                },
                command: CommandSpecV1 {
                    template: vec!["echo".to_string(), "fastq.index_reference".to_string()],
                },
                resources: ToolConstraints {
                    runtime: "docker".to_string(),
                    mem_gb: 1,
                    tmp_gb: 1,
                    threads: 1,
                },
            },
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.deplete_host".to_string(),
            stage_instance_id: None,
            tool: tool_for_stage("fastq.deplete_host"),
            reason: None,
            params: None,
        },
    ];
    let temp = bijux_dna_infra::temp_dir("fastq-plan-reference-mismatch")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        Some(&reference),
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("STAR index must not satisfy bowtie2 depletion");

    let message = error.to_string();
    assert!(
        message.contains("requires one of [bowtie2_build]"),
        "planner must explain the governed compatible index backends: {message}"
    );
    Ok(())
}

#[test]
fn compose_rejects_tool_from_wrong_stage_roster() -> anyhow::Result<()> {
    let bindings = vec![FastqStageBinding {
        stage_id: "fastq.validate_reads".to_string(),
        stage_instance_id: None,
        tool: tool_for_stage("fastq.trim_reads"),
        reason: None,
        params: None,
    }];
    let temp = bijux_dna_infra::temp_dir("fastq-plan-foreign-tool")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("foreign stage tool must be rejected before adapter planning");

    assert!(error.to_string().contains("is not admitted for fastq.validate_reads"));
    Ok(())
}

#[test]
fn compose_rejects_params_from_wrong_stage_variant() -> anyhow::Result<()> {
    let bindings = vec![FastqStageBinding {
        stage_id: "fastq.validate_reads".to_string(),
        stage_instance_id: None,
        tool: tool_for_stage("fastq.validate_reads"),
        reason: None,
        params: Some(FastqStageParameters::FilterReads(FilterReadsStageParams {
            max_n: Some(0),
            ..FilterReadsStageParams::default()
        })),
    }];
    let temp = bijux_dna_infra::temp_dir("fastq-plan-foreign-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("stage params from a different stage must be rejected");

    assert!(error.to_string().contains("received FilterReads parameters"));
    Ok(())
}

#[test]
fn compose_rejects_report_artifact_bound_as_reads() -> anyhow::Result<()> {
    let bindings =
        vec![binding_for_stage("fastq.validate_reads"), binding_for_stage("fastq.trim_reads")];
    let mut explicit_inputs = StageArtifactInputPolicy::new();
    explicit_inputs.insert(
        "fastq.trim_reads".to_string(),
        vec![StageArtifactInputBinding {
            from_stage_node_id: "fastq.validate_reads".to_string(),
            from_output_id: "validation_report".to_string(),
            to_input_id: "reads_r1".to_string(),
        }],
    );
    let temp = bijux_dna_infra::temp_dir("fastq-plan-artifact-role")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        Some(&explicit_inputs),
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("report JSON must not satisfy reads_r1");

    assert!(error.to_string().contains("has role report_json"));
    Ok(())
}

#[test]
fn compose_rejects_reads_artifact_bound_as_qc_input() -> anyhow::Result<()> {
    let bindings =
        vec![binding_for_stage("fastq.trim_reads"), binding_for_stage("fastq.report_qc")];
    let mut explicit_inputs = StageArtifactInputPolicy::new();
    explicit_inputs.insert(
        "fastq.report_qc".to_string(),
        vec![StageArtifactInputBinding {
            from_stage_node_id: "fastq.trim_reads".to_string(),
            from_output_id: "trimmed_reads_r1".to_string(),
            to_input_id: "qc_artifacts".to_string(),
        }],
    );
    let temp = bijux_dna_infra::temp_dir("fastq-plan-qc-artifact-role")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        Some(&explicit_inputs),
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("read artifacts must not satisfy report_qc qc_artifacts");

    assert!(error.to_string().contains("explicit input qc_artifacts"));
    assert!(error.to_string().contains("has role trimmed_reads"));
    Ok(())
}

#[test]
fn compose_rejects_non_governed_index_bound_as_qc_input() -> anyhow::Result<()> {
    let bindings =
        vec![binding_for_stage("fastq.index_reference"), binding_for_stage("fastq.report_qc")];
    let mut explicit_inputs = StageArtifactInputPolicy::new();
    explicit_inputs.insert(
        "fastq.report_qc".to_string(),
        vec![StageArtifactInputBinding {
            from_stage_node_id: "fastq.index_reference".to_string(),
            from_output_id: "reference_index".to_string(),
            to_input_id: "qc_artifacts".to_string(),
        }],
    );
    let temp = bijux_dna_infra::temp_dir("fastq-plan-qc-governance")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        Some(&reference),
        Some(&explicit_inputs),
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("non-QC reference indexes must not satisfy report_qc qc_artifacts");

    assert!(error.to_string().contains("is not a governed QC output"));
    Ok(())
}

#[test]
fn compose_rejects_single_end_input_for_paired_only_tool() -> anyhow::Result<()> {
    let bindings = vec![FastqStageBinding {
        stage_id: "fastq.remove_duplicates".to_string(),
        stage_instance_id: None,
        tool: ToolExecutionSpecV1 {
            tool_id: ToolId::new("fastuniq"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["fastuniq".to_string()] },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        },
        reason: None,
        params: None,
    }];
    let temp = bijux_dna_infra::temp_dir("fastq-plan-layout")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = compose_fastq_stage_bindings(
        &bindings,
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        &r1,
        None,
        None,
        None,
        |binding, _r1, _r2| {
            Ok(temp.path().join(binding.stage_id.as_str()).join(binding.tool.tool_id.as_str()))
        },
    )
    .expect_err("paired-only tools must reject single-end inputs");

    assert!(error.to_string().contains("does not support single-end inputs"));
    Ok(())
}
