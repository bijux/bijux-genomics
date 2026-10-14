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
        stage_id: "fastq.merge_pairs".to_string(),
        stage_instance_id: None,
        tool: ToolExecutionSpecV1 {
            tool_id: ToolId::new("flash2"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
            command: CommandSpecV1 { template: vec!["flash2".to_string()] },
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

    assert!(error
        .to_string()
        .contains("does not admit SingleEnd inputs under the governed FASTQ layout contract"));
    Ok(())
}

#[test]
fn correction_read_lineage_preserves_declared_mates_and_excludes_reports() -> anyhow::Result<()> {
    for r2 in [None, Some(std::path::Path::new("raw_R2.fastq"))] {
        let bindings = [
            FastqStageBinding {
                stage_id: "fastq.correct_errors".into(),
                stage_instance_id: Some("fastq.correct_errors.corrected".into()),
                tool: tool_for_stage("fastq.correct_errors"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.validate_reads".into(),
                stage_instance_id: Some("fastq.validate_reads.corrected".into()),
                tool: tool_for_stage("fastq.validate_reads"),
                reason: None,
                params: None,
            },
        ];
        let plans = compose_fastq_stage_bindings(
            &bindings,
            &BTreeMap::new(),
            None,
            None,
            None,
            false,
            std::path::Path::new("raw_R1.fastq"),
            r2,
            None,
            None,
            |binding, _, _| Ok(std::path::PathBuf::from("out").join(&binding.stage_id)),
        )?;
        assert_eq!(plans.len(), 2);
        let correction = &plans[0];
        let validation = &plans[1];
        for (input_id, output_id) in
            [("reads_r1", "corrected_reads_r1"), ("reads_r2", "corrected_reads_r2")]
        {
            let output =
                correction.io.outputs.iter().find(|artifact| artifact.name.as_str() == output_id);
            let input =
                validation.io.inputs.iter().find(|artifact| artifact.name.as_str() == input_id);
            if input_id == "reads_r2" && r2.is_none() {
                assert!(output.is_none());
                assert!(input.is_none());
            } else {
                let output = output
                    .ok_or_else(|| anyhow::anyhow!("correction output {output_id} missing"))?;
                let input =
                    input.ok_or_else(|| anyhow::anyhow!("validation input {input_id} missing"))?;
                assert_eq!(
                    input.path, output.path,
                    "{input_id} must preserve its declared read artifact"
                );
            }
        }
        for report in
            correction.io.outputs.iter().filter(|artifact| {
                artifact.role == bijux_dna_core::prelude::ArtifactRole::ReportJson
            })
        {
            assert!(
                validation
                    .io
                    .inputs
                    .iter()
                    .filter(|artifact| matches!(artifact.name.as_str(), "reads_r1" | "reads_r2"))
                    .all(|artifact| artifact.path != report.path),
                "report artifact must never become a read mate"
            );
        }
    }
    Ok(())
}

fn explicitly_bound_validation(
    instance: Option<&str>,
    inputs: &[bijux_dna_planner_fastq::FastqStageExplicitInput],
) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> {
    bijux_dna_planner_fastq::plan_fastq_stage_binding_with_explicit_inputs(
        FastqStageBinding {
            stage_id: "fastq.validate_reads".into(),
            stage_instance_id: instance.map(str::to_owned),
            tool: tool_for_stage("fastq.validate_reads"),
            reason: None,
            params: None,
        },
        &BTreeMap::new(),
        None,
        None,
        None,
        false,
        std::path::Path::new("fallback.fastq"),
        None,
        None,
        inputs,
        std::path::Path::new("out"),
    )
}

fn explicit_reads(
    artifact: bijux_dna_core::prelude::ArtifactRef,
) -> bijux_dna_planner_fastq::FastqStageExplicitInput {
    bijux_dna_planner_fastq::FastqStageExplicitInput {
        input_id: "reads_r1".into(),
        artifact,
        source_tool_id: Some("rcorrector".into()),
    }
}

#[test]
fn explicit_input_binding_selects_reads_for_both_stage_identity_forms() -> anyhow::Result<()> {
    use bijux_dna_core::prelude::{ArtifactId, ArtifactRef, ArtifactRole};
    for instance in [None, Some("fastq.validate_reads.selected")] {
        let input = explicit_reads(ArtifactRef::required(
            ArtifactId::from_static("corrected_reads_r1"),
            "selected.fastq".into(),
            ArtifactRole::Reads,
        ));
        let plan = explicitly_bound_validation(instance, &[input])?;
        let observed = plan
            .io
            .inputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "reads_r1")
            .ok_or_else(|| anyhow::anyhow!("validation reads_r1 missing"))?;
        assert_eq!(observed.path, std::path::Path::new("selected.fastq"));
        assert_ne!(observed.path, std::path::Path::new("fallback.fastq"));
        if let Some(instance) = instance {
            assert_eq!(
                plan.stage_instance_id.as_ref().map(bijux_dna_core::contract::StepId::as_str),
                Some(instance)
            );
        }
    }
    Ok(())
}

#[test]
fn explicit_input_binding_refuses_reports_and_duplicate_reads_for_both_stage_identity_forms(
) -> anyhow::Result<()> {
    use bijux_dna_core::prelude::{ArtifactId, ArtifactRef, ArtifactRole};
    for instance in [None, Some("fastq.validate_reads.selected")] {
        let report = explicit_reads(ArtifactRef::required(
            ArtifactId::from_static("correction_report"),
            "correction.json".into(),
            ArtifactRole::ReportJson,
        ));
        let error = explicitly_bound_validation(instance, &[report])
            .err()
            .ok_or_else(|| anyhow::anyhow!("report artifact accepted as reads for {instance:?}"))?;
        assert!(error.to_string().contains("has role report_json"), "{error}");
        let left = explicit_reads(ArtifactRef::required(
            ArtifactId::from_static("corrected_reads_left"),
            "left.fastq".into(),
            ArtifactRole::Reads,
        ));
        let right = explicit_reads(ArtifactRef::required(
            ArtifactId::from_static("corrected_reads_right"),
            "right.fastq".into(),
            ArtifactRole::Reads,
        ));
        let error = explicitly_bound_validation(instance, &[left, right])
            .err()
            .ok_or_else(|| anyhow::anyhow!("duplicate singular reads accepted for {instance:?}"))?;
        assert!(error.to_string().contains("multiple explicit artifact bindings"), "{error}");
    }
    Ok(())
}
