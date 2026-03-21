use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec, PlanPolicy};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::{
    DepleteHostStageParams, DepleteReferenceContaminantsStageParams, DepleteRrnaStageParams,
    FastqPlanConfig, FastqPlanner, FastqStageBinding, FastqStageParameters,
    FastqStageToolsetBinding,
    TrimTerminalDamageStageParams,
};

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/dummy:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), tool_id.to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn planner_accepts_explicit_stage_bindings_with_repeated_stage_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-bindings")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__stage_bindings__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                tool: tool("cutadapt"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: r1.clone(),
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    assert_eq!(plan.steps().len(), 2);
    assert!(plan
        .steps()
        .iter()
        .any(|step| step.step_id.as_str() == "fastq.trim_reads.fastp_branch"));
    assert!(plan
        .steps()
        .iter()
        .any(|step| step.step_id.as_str() == "fastq.trim_reads.cutadapt_branch"));
    let fastp_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.fastp_branch")
        .expect("fastp branch");
    let cutadapt_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.cutadapt_branch")
        .expect("cutadapt branch");
    assert_ne!(fastp_step.out_dir, cutadapt_step.out_dir);
    assert!(
        fastp_step
            .out_dir
            .to_string_lossy()
            .contains("trim_reads.fastp_branch"),
        "fastp branch out_dir must include stage instance identity"
    );
    assert!(
        cutadapt_step
            .out_dir
            .to_string_lossy()
            .contains("trim_reads.cutadapt_branch"),
        "cutadapt branch out_dir must include stage instance identity"
    );
    Ok(())
}

#[test]
fn planner_expands_stage_toolsets_into_route_specific_graph_nodes() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-toolsets")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            },
        ],
        vec![PipelineEdgeSpec {
            from: "fastq.validate_reads.entry".to_string(),
            to: "fastq.trim_reads.cleanup".to_string(),
            from_output_id: None,
            to_input_id: None,
        }],
    );

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__stage_toolsets__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(pipeline),
        stage_bindings: Vec::new(),
        stage_toolsets: vec![
            FastqStageToolsetBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
                tools: vec![tool("fastqvalidator")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
                tools: vec![tool("fastp"), tool("bbduk")],
                reason: None,
                params: None,
            },
        ],
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    assert_eq!(plan.steps().len(), 5);
    assert_eq!(plan.edges().len(), 4);
    assert!(plan.steps().iter().any(|step| {
        step.step_id
            .as_str()
            .contains("fastq.validate_reads.entry=fastqvalidator")
    }));
    assert!(plan.steps().iter().any(|step| {
        step.step_id
            .as_str()
            .contains("fastq.trim_reads.cleanup=fastp")
            && step.step_id.as_str().contains(".tool.fastp")
    }));
    Ok(())
}

#[test]
fn planner_injects_compare_step_for_multi_tool_stage_routes() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-toolset-compare")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            },
        ],
        vec![PipelineEdgeSpec {
            from: "fastq.validate_reads.entry".to_string(),
            to: "fastq.trim_reads.cleanup".to_string(),
            from_output_id: None,
            to_input_id: None,
        }],
    );

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__stage_toolset_compare__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(pipeline),
        stage_bindings: Vec::new(),
        stage_toolsets: vec![
            FastqStageToolsetBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
                tools: vec![tool("fastqvalidator")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
                tools: vec![tool("fastp"), tool("cutadapt")],
                reason: None,
                params: None,
            },
        ],
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let compare_step = plan
        .steps()
        .iter()
        .find(|step| {
            step.step_id
                .as_str()
                .starts_with("fastq.trim_reads.cleanup.compare.route.")
        })
        .expect("trim toolset compare step");
    assert_eq!(
        compare_step.stage_id.as_str(),
        "benchmark.compare_stage_tools"
    );
    assert_eq!(compare_step.io.inputs.len(), 4);
    assert!(compare_step
        .io
        .inputs
        .iter()
        .all(|artifact| {
            let name = artifact.name.as_str();
            name.ends_with("__trimmed_reads_r1") || name.ends_with("__report_json")
        }));
    assert!(plan.edges().iter().any(|edge| {
        edge.to().as_str().starts_with("fastq.trim_reads.cleanup.compare.route.")
            && edge.from().as_str().contains(".tool.fastp")
    }));
    assert!(plan.edges().iter().any(|edge| {
        edge.to().as_str().starts_with("fastq.trim_reads.cleanup.compare.route.")
            && edge.from().as_str().contains(".tool.cutadapt")
    }));
    Ok(())
}

#[test]
fn planner_scopes_compare_steps_by_remaining_route_context() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-toolset-compare-context")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r2\nT\n+\n#\n")?;

    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.remove_duplicates".to_string(),
                stage_instance_id: Some("fastq.remove_duplicates.selected".to_string()),
            },
        ],
        vec![
            PipelineEdgeSpec {
                from: "fastq.validate_reads.entry".to_string(),
                to: "fastq.trim_reads.cleanup".to_string(),
                from_output_id: None,
                to_input_id: None,
            },
            PipelineEdgeSpec {
                from: "fastq.trim_reads.cleanup".to_string(),
                to: "fastq.remove_duplicates.selected".to_string(),
                from_output_id: None,
                to_input_id: None,
            },
        ],
    );

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__stage_toolset_compare_context__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(pipeline),
        stage_bindings: Vec::new(),
        stage_toolsets: vec![
            FastqStageToolsetBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
                tools: vec![tool("fastqvalidator")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
                tools: vec![tool("fastp"), tool("cutadapt")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.remove_duplicates".to_string(),
                stage_instance_id: Some("fastq.remove_duplicates.selected".to_string()),
                tools: vec![tool("clumpify"), tool("fastuniq")],
                reason: None,
                params: None,
            },
        ],
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let trim_compare_steps = plan
        .steps()
        .iter()
        .filter(|step| {
            step.step_id
                .as_str()
                .starts_with("fastq.trim_reads.cleanup.compare.route.")
        })
        .collect::<Vec<_>>();
    assert_eq!(trim_compare_steps.len(), 2);
    assert!(trim_compare_steps.iter().all(|step| step.io.inputs.len() == 6));
    assert!(trim_compare_steps.iter().any(|step| {
        step.step_id
            .as_str()
            .contains("fastq.remove_duplicates.selected=clumpify")
    }));
    assert!(trim_compare_steps.iter().any(|step| {
        step.step_id
            .as_str()
            .contains("fastq.remove_duplicates.selected=fastuniq")
    }));
    Ok(())
}

#[test]
fn graph_root_branches_do_not_inherit_previous_branch_reads() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-graph-root-branches")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__root_branches__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_root".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.bbduk_root".to_string()),
                },
            ],
            Vec::new(),
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_root".to_string()),
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.bbduk_root".to_string()),
                tool: tool("bbduk"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: r1.clone(),
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let fastp_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.fastp_root")
        .expect("fastp root branch");
    let bbduk_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.bbduk_root")
        .expect("bbduk root branch");

    assert_eq!(fastp_step.io.inputs[0].path, r1);
    assert_eq!(bbduk_step.io.inputs[0].path, r1);
    assert_ne!(fastp_step.io.outputs[0].path, bbduk_step.io.outputs[0].path);
    Ok(())
}

#[test]
fn planner_rejects_duplicate_stage_nodes_without_distinct_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-binding-collision")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__stage_bindings__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: None,
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: None,
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: PathBuf::from("out"),
        tool_reasons: None,
        allow_planned: false,
    })
    .expect_err("duplicate stage bindings without instance ids must fail");

    assert!(error
        .to_string()
        .contains("must set distinct stage_instance_id values"));
    Ok(())
}

#[test]
fn planner_uses_typed_trim_terminal_damage_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-trim-terminal-damage-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__trim_terminal_damage__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.trim_terminal_damage".to_string(),
            stage_instance_id: Some("fastq.trim_terminal_damage.custom".to_string()),
            tool: tool("cutadapt"),
            reason: None,
            params: Some(FastqStageParameters::TrimTerminalDamage(
                TrimTerminalDamageStageParams {
                    damage_mode: "udg_trimmed".to_string(),
                    trim_5p_bases: 5,
                    trim_3p_bases: 3,
                },
            )),
        }],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.trim_terminal_damage.custom");
    assert!(step.command.template.iter().any(|part| part == "5"));
    assert!(step.command.template.iter().any(|part| part == "-3"));
    Ok(())
}

#[test]
fn planner_uses_typed_rrna_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-deplete-rrna-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__deplete_rrna__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.deplete_rrna".to_string(),
            stage_instance_id: Some("fastq.deplete_rrna.sortmerna.custom".to_string()),
            tool: tool("sortmerna"),
            reason: None,
            params: Some(FastqStageParameters::DepleteRrna(DepleteRrnaStageParams {
                rrna_db: "silva_nr99".to_string(),
                min_identity: 0.99,
            })),
        }],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.deplete_rrna.sortmerna.custom");
    assert!(step.command.template.iter().any(|part| part == "--report"));
    Ok(())
}

#[test]
fn planner_rejects_unsupported_host_retention_policy_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-deplete-host-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__deplete_host__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.host".to_string()),
                tool: tool("bowtie2_build"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_host".to_string(),
                stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                tool: tool("bowtie2"),
                reason: None,
                params: Some(FastqStageParameters::DepleteHost(DepleteHostStageParams {
                    host_identity_threshold: 0.95,
                    retain_unmapped_only: false,
                })),
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })
    .expect_err("unsupported host retention policy must fail loudly");

    assert!(error.to_string().contains("retain_unmapped_only=true"));
    Ok(())
}

#[test]
fn planner_uses_typed_reference_contaminant_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-reference-contaminant-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__reference_contaminants__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: None,
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.decoy".to_string()),
                tool: tool("bowtie2_build"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_reference_contaminants".to_string(),
                stage_instance_id: Some("fastq.deplete_reference_contaminants.decoy".to_string()),
                tool: tool("bowtie2"),
                reason: None,
                params: Some(FastqStageParameters::DepleteReferenceContaminants(
                    DepleteReferenceContaminantsStageParams {
                        decoy_mode: "phix_only".to_string(),
                    },
                )),
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.deplete_reference_contaminants.decoy")
        .expect("reference contaminant step");
    assert!(step.command.template.iter().any(|part| part == "--un-gz"));
    Ok(())
}

#[test]
fn planner_preserves_explicit_pipeline_graph_edges() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-explicit-pipeline-graph")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__explicit_graph__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.detect_adapters".to_string(),
                    stage_instance_id: Some("fastq.detect_adapters.fastqc".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.validate_reads.validator".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: Some("validation_report".to_string()),
                    to_input_id: Some("validation_report".to_string()),
                },
                PipelineEdgeSpec {
                    from: "fastq.detect_adapters.fastqc".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: Some("adapter_report".to_string()),
                    to_input_id: Some("adapter_report".to_string()),
                },
            ],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                tool: tool("fastqvalidator"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.detect_adapters".to_string(),
                stage_instance_id: Some("fastq.detect_adapters.fastqc".to_string()),
                tool: tool("fastqc"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.report_qc".to_string(),
                stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                tool: tool("multiqc"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    assert_eq!(plan.edges().len(), 2);
    assert!(plan
        .edges()
        .iter()
        .all(|edge| edge.to().as_str() == "fastq.report_qc.aggregate"));
    assert!(plan
        .edges()
        .iter()
        .all(|edge| edge.from_output_id().is_some() && edge.to_input_id().is_some()));
    let report_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.report_qc.aggregate")
        .expect("report_qc step");
    assert_eq!(report_step.io.inputs.len(), 2);
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "validation_report"));
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "adapter_report"));
    Ok(())
}

#[test]
fn planner_routes_explicit_reads_bindings_into_rejoin_stage() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-explicit-read-rejoin")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__explicit_rejoin__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.filter_reads".to_string(),
                    stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.trim_reads.fastp_branch".to_string(),
                to: "fastq.filter_reads.selected".to_string(),
                from_output_id: Some("trimmed_reads_r1".to_string()),
                to_input_id: Some("reads_r1".to_string()),
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                tool: tool("cutadapt"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.filter_reads".to_string(),
                stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                tool: tool("seqkit"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let fastp_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.fastp_branch")
        .expect("fastp trim branch");
    let filter_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.filter_reads.selected")
        .expect("selected filter stage");
    assert_eq!(filter_step.io.inputs[0].path, fastp_step.io.outputs[0].path);
    Ok(())
}

#[test]
fn planner_injects_select_step_and_rejoins_downstream_reads() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-select-rejoin")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__select_rejoin__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "benchmark.select_stage_tool".to_string(),
                    stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.filter_reads".to_string(),
                    stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.fastp_branch".to_string(),
                    to: "benchmark.select_stage_tool.trim_reads".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("fastp_trimmed_reads_r1".to_string()),
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.cutadapt_branch".to_string(),
                    to: "benchmark.select_stage_tool.trim_reads".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("cutadapt_trimmed_reads_r1".to_string()),
                },
                PipelineEdgeSpec {
                    from: "benchmark.select_stage_tool.trim_reads".to_string(),
                    to: "fastq.filter_reads.selected".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("reads_r1".to_string()),
                },
            ],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                tool: tool("cutadapt"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.filter_reads".to_string(),
                stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                tool: tool("seqkit"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let select_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "benchmark.select_stage_tool.trim_reads")
        .expect("select step");
    let filter_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.filter_reads.selected")
        .expect("rejoin stage");
    assert_eq!(select_step.stage_id.as_str(), "benchmark.select_stage_tool");
    assert_eq!(select_step.io.inputs.len(), 2);
    assert_eq!(select_step.io.outputs.len(), 1);
    assert_eq!(filter_step.io.inputs[0].path, select_step.io.outputs[0].path);
    assert!(plan.edges().iter().any(|edge| {
        edge.from().as_str() == "benchmark.select_stage_tool.trim_reads"
            && edge.to().as_str() == "fastq.filter_reads.selected"
            && edge.from_output_id().is_some()
            && edge.to_input_id().is_some()
    }));
    Ok(())
}

#[test]
fn planner_rejects_selection_rejoin_without_artifact_bindings() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-select-rejoin-invalid")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__select_rejoin_invalid__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "benchmark.select_stage_tool".to_string(),
                    stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.filter_reads".to_string(),
                    stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.fastp_branch".to_string(),
                    to: "benchmark.select_stage_tool.trim_reads".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("fastp_trimmed_reads_r1".to_string()),
                },
                PipelineEdgeSpec {
                    from: "fastq.trim_reads.cutadapt_branch".to_string(),
                    to: "benchmark.select_stage_tool.trim_reads".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("cutadapt_trimmed_reads_r1".to_string()),
                },
                PipelineEdgeSpec {
                    from: "benchmark.select_stage_tool.trim_reads".to_string(),
                    to: "fastq.filter_reads.selected".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp_branch".to_string()),
                tool: tool("fastp"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cutadapt_branch".to_string()),
                tool: tool("cutadapt"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.filter_reads".to_string(),
                stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                tool: tool("seqkit"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })
    .expect_err("selection rejoin should require artifact-bound edges");

    assert!(error
        .to_string()
        .contains("requires artifact-bound outgoing rejoin edges"));
    Ok(())
}

#[test]
fn planner_uses_explicit_reference_index_bindings_for_reference_aware_stages() -> anyhow::Result<()>
{
    let temp = bijux_dna_infra::temp_dir("fastq-explicit-reference-index")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference_fasta = temp.path().join("reference.fasta");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference_fasta, b">chr1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__explicit_reference_index__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.host".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.alt".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.deplete_host".to_string(),
                    stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.index_reference.host".to_string(),
                to: "fastq.deplete_host.host".to_string(),
                from_output_id: Some("reference_index".to_string()),
                to_input_id: Some("reference_index".to_string()),
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.host".to_string()),
                tool: tool("bowtie2_build"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.alt".to_string()),
                tool: tool("star"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_host".to_string(),
                stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                tool: tool("bowtie2"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let host_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.deplete_host.host")
        .expect("host depletion stage");
    assert!(host_step
        .command
        .template
        .iter()
        .any(|part| part.contains("index_reference.host")));
    Ok(())
}

#[test]
fn planner_resolves_unique_reference_index_dependency_without_artifact_binding(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-implicit-reference-index")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference_fasta = temp.path().join("reference.fasta");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference_fasta, b">chr1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__implicit_reference_index__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.host".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.deplete_host".to_string(),
                    stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.index_reference.host".to_string(),
                to: "fastq.deplete_host.host".to_string(),
                from_output_id: None,
                to_input_id: None,
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.host".to_string()),
                tool: tool("bowtie2_build"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_host".to_string(),
                stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                tool: tool("bowtie2"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let host_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.deplete_host.host")
        .expect("host depletion stage");
    assert!(host_step
        .command
        .template
        .iter()
        .any(|part| part.contains("index_reference.host")));
    Ok(())
}

#[test]
fn planner_rejects_ambiguous_reference_index_dependencies_without_explicit_binding(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-ambiguous-reference-index")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference_fasta = temp.path().join("reference.fasta");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference_fasta, b">chr1\nACGT\n")?;

    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__ambiguous_reference_index__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.host".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: Some("fastq.index_reference.alt".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.deplete_host".to_string(),
                    stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                },
            ],
            vec![
                PipelineEdgeSpec {
                    from: "fastq.index_reference.host".to_string(),
                    to: "fastq.deplete_host.host".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
                PipelineEdgeSpec {
                    from: "fastq.index_reference.alt".to_string(),
                    to: "fastq.deplete_host.host".to_string(),
                    from_output_id: None,
                    to_input_id: None,
                },
            ],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.host".to_string()),
                tool: tool("bowtie2_build"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: Some("fastq.index_reference.alt".to_string()),
                tool: tool("star"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_host".to_string(),
                stage_instance_id: Some("fastq.deplete_host.host".to_string()),
                tool: tool("bowtie2"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })
    .expect_err("ambiguous dependency should fail");

    assert!(error
        .to_string()
        .contains("multiple fastq.index_reference nodes"));
    Ok(())
}

#[test]
fn planner_uses_explicit_abundance_table_bindings() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-explicit-abundance-table")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__explicit_abundance__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.cluster_otus".to_string(),
                    stage_instance_id: Some("fastq.cluster_otus.primary".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.cluster_otus".to_string(),
                    stage_instance_id: Some("fastq.cluster_otus.secondary".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.normalize_abundance".to_string(),
                    stage_instance_id: Some("fastq.normalize_abundance.selected".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.cluster_otus.secondary".to_string(),
                to: "fastq.normalize_abundance.selected".to_string(),
                from_output_id: Some("otu_table".to_string()),
                to_input_id: Some("abundance_table".to_string()),
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.cluster_otus".to_string(),
                stage_instance_id: Some("fastq.cluster_otus.primary".to_string()),
                tool: tool("vsearch"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.cluster_otus".to_string(),
                stage_instance_id: Some("fastq.cluster_otus.secondary".to_string()),
                tool: tool("vsearch"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.normalize_abundance".to_string(),
                stage_instance_id: Some("fastq.normalize_abundance.selected".to_string()),
                tool: tool("seqkit"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    let cluster_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.cluster_otus.secondary")
        .expect("secondary cluster stage");
    let abundance_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.normalize_abundance.selected")
        .expect("abundance stage");
    assert_eq!(
        abundance_step.io.inputs[0].path,
        cluster_step.io.outputs[0].path
    );
    Ok(())
}

#[test]
fn planner_resolves_graph_stage_aliases_for_unique_stages() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-graph-stage-aliases")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__graph_aliases__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.validate_reads".to_string(),
                    stage_instance_id: None,
                },
                PipelineNodeSpec {
                    stage_id: "fastq.detect_adapters".to_string(),
                    stage_instance_id: None,
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.validate_reads".to_string(),
                to: "fastq.detect_adapters".to_string(),
                from_output_id: None,
                to_input_id: None,
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: None,
                tool: tool("fastqvalidator"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.detect_adapters".to_string(),
                stage_instance_id: None,
                tool: tool("fastqc"),
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        stages: Vec::new(),
        tools: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
        allow_planned: false,
    })?;

    assert_eq!(plan.edges().len(), 1);
    assert_eq!(
        plan.edges()[0].from().as_str(),
        "fastq.validate_reads.tool.fastqvalidator"
    );
    assert_eq!(
        plan.edges()[0].to().as_str(),
        "fastq.detect_adapters.tool.fastqc"
    );
    Ok(())
}
