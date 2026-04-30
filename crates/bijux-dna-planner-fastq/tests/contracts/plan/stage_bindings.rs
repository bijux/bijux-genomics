use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec, PlanPolicy};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use bijux_dna_domain_fastq::params::edna::ChimeraDetectionEffectiveParams;
use bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy;
use bijux_dna_domain_fastq::params::qc_post::{
    QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams,
};
use bijux_dna_domain_fastq::params::remove_duplicates::{
    DedupMode, RemoveDuplicatesEffectiveParams,
};
use bijux_dna_domain_fastq::params::screen::{
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
    TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::stats::{
    FastqStatsParams, OVERREPRESENTED_PROFILE_SCHEMA_VERSION, READ_LENGTH_PROFILE_SCHEMA_VERSION,
    STATS_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::trim::{
    TrimEffectiveParams, TrimPolygTailsParams, TRIM_POLYG_TAILS_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::validate::{
    PairSyncPolicy, ValidateEffectiveParams, ValidationMode, VALIDATE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::DamageMode;
use bijux_dna_domain_fastq::{FastqOverrepresentedProfileParams, FastqReadLengthProfileParams};
use bijux_dna_planner_fastq::{
    ClusterOtusStageParams, CorrectErrorsStageParams, DepleteHostStageParams,
    DepleteReferenceContaminantsStageParams, DepleteRrnaStageParams, DetectAdaptersStageParams,
    ExtractUmisStageParams, FastqPlanConfig, FastqPlanner, FastqStageBinding, FastqStageParameters,
    FastqStageToolsetBinding, FilterLowComplexityStageParams, FilterReadsStageParams,
    IndexReferenceStageParams, InferAsvsStageParams, MergePairsStageParams,
    NormalizeAbundanceStageParams, NormalizePrimersStageParams, TrimTerminalDamageStageParams,
};

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

fn tool_with_threads(tool_id: &str, threads: u32) -> ToolExecutionSpecV1 {
    let mut tool = tool(tool_id);
    tool.resources.threads = threads;
    tool
}

fn seqkit_stats_tool() -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new("seqkit_stats".to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 {
            template: vec![
                "seqkit".to_string(),
                "stats".to_string(),
                "-a".to_string(),
                "-T".to_string(),
                "-j".to_string(),
                "{{threads}}".to_string(),
                "{{reads_r1}}".to_string(),
                "{{reads_r2}}".to_string(),
            ],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn umi_tools_tool(threads: u32) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new("umi_tools".to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 {
            template: vec![
                "umi_tools".to_string(),
                "extract".to_string(),
                "--stdin".to_string(),
                "{{reads_r1}}".to_string(),
                "--stdout".to_string(),
                "{{umi_reads_r1}}".to_string(),
                "--read2-in".to_string(),
                "{{reads_r2}}".to_string(),
                "--read2-out".to_string(),
                "{{umi_reads_r2}}".to_string(),
                "--bc-pattern".to_string(),
                "{{umi_pattern}}".to_string(),
                "--log".to_string(),
                "{{raw_backend_report}}".to_string(),
            ],
        },
        resources: ToolConstraints { runtime: "docker".to_string(), mem_gb: 1, tmp_gb: 1, threads },
    }
}

fn clumpify_tool() -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new("clumpify".to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/dummy:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec![
                "bash".to_string(),
                "-lc".to_string(),
                "set -euo pipefail\nclumpify in='{{reads_r1}}' {{paired_io_args}} out='{{dedup_reads_r1}}' {{dedup_mode_args}} {{keep_order_args}} {{threads_args}} > '{{out_dir}}/clumpify.log' 2>&1\n".to_string(),
            ],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

fn vsearch_tool() -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new("vsearch".to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["vsearch".to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 2,
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
            ],
            Vec::new(),
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
        ],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: r1.clone(),
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
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
        fastp_step.out_dir.to_string_lossy().contains("trim_reads.fastp_branch"),
        "fastp branch out_dir must include stage instance identity"
    );
    assert!(
        cutadapt_step.out_dir.to_string_lossy().contains("trim_reads.cutadapt_branch"),
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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

    assert_eq!(plan.steps().len(), 4);
    assert_eq!(plan.edges().len(), 4);
    assert!(plan.steps().iter().any(|step| {
        step.step_id.as_str().contains("fastq.validate_reads.entry=fastqvalidator")
    }));
    assert!(plan.steps().iter().any(|step| {
        step.step_id.as_str().contains("fastq.trim_reads.cleanup=fastp")
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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

    let compare_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str().starts_with("fastq.trim_reads.cleanup.compare.route."))
        .expect("trim toolset compare step");
    assert_eq!(compare_step.stage_id.as_str(), "benchmark.compare_stage_tools");
    assert_eq!(compare_step.io.inputs.len(), 4);
    assert!(compare_step.io.inputs.iter().all(|artifact| {
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let trim_compare_steps = plan
        .steps()
        .iter()
        .filter(|step| step.step_id.as_str().starts_with("fastq.trim_reads.cleanup.compare.route."))
        .collect::<Vec<_>>();
    assert_eq!(trim_compare_steps.len(), 1);
    assert!(trim_compare_steps.iter().all(|step| step.io.inputs.len() == 6));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: r1.clone(),
        r2: None,
        reference_fasta: None,
        out_dir: temp.path().join("out"),
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: None,
        out_dir: PathBuf::from("out"),
        allow_planned: false,
    })
    .expect_err("duplicate stage bindings without instance ids must fail");

    assert!(error.to_string().contains("must set distinct stage_instance_id values"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.trim_terminal_damage".to_string(),
            stage_instance_id: Some("fastq.trim_terminal_damage.custom".to_string()),
            tool: tool("cutadapt"),
            reason: None,
            params: Some(FastqStageParameters::TrimTerminalDamage(TrimTerminalDamageStageParams {
                threads: None,
                damage_mode: DamageMode::UdgTrimmed,
                execution_policy: None,
                trim_5p_bases: 5,
                trim_3p_bases: 3,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.trim_terminal_damage.custom");
    let script = &step.command.template[2];
    assert!(script.contains(" -u 5"));
    assert!(script.contains(" -u -3"));
    Ok(())
}

#[test]
fn planner_uses_typed_validate_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-validate-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r1\nT\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__validate_reads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.validate_reads".to_string(),
            stage_instance_id: Some("fastq.validate_reads.custom".to_string()),
            tool: tool("fastqvalidator"),
            reason: None,
            params: Some(FastqStageParameters::Validate(ValidateEffectiveParams {
                schema_version: VALIDATE_SCHEMA_VERSION.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
                threads: 6,
                validation_mode: ValidationMode::ReportOnly,
                pair_sync_policy: PairSyncPolicy::SkipHeaderSync,
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.validate_reads.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("\"validation_mode\":\"report_only\""));
    assert!(step.command.template[2].contains("\"pair_sync_policy\":\"skip_header_sync\""));
    assert!(!step.command.template[2].contains("cmp -s"));
    Ok(())
}

#[test]
fn planner_uses_typed_merge_pairs_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-merge-pairs-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__merge_pairs__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.merge_pairs".to_string(),
            stage_instance_id: Some("fastq.merge_pairs.custom".to_string()),
            tool: vsearch_tool(),
            reason: None,
            params: Some(FastqStageParameters::MergePairs(MergePairsStageParams {
                threads: Some(7),
                merge_overlap: Some(22),
                min_len: Some(120),
                unmerged_read_policy: UnmergedReadPolicy::OmitUnmergedPairs,
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.merge_pairs.custom");
    assert_eq!(step.resources.threads, 7);
    assert!(
        step.command.template[2].contains("'--fastq_minovlen' '22'"),
        "merge overlap flag missing from {:?}",
        step.command.template
    );
    assert!(
        step.command.template[2].contains("'--fastq_minmergelen' '120'"),
        "min length flag missing from {:?}",
        step.command.template
    );
    assert!(!step.io.outputs.iter().any(|artifact| artifact.name.as_str() == "unmerged_reads_r1"));
    assert!(!step.io.outputs.iter().any(|artifact| artifact.name.as_str() == "unmerged_reads_r2"));
    Ok(())
}

#[test]
fn planner_uses_manifest_default_merge_pairs_threads_without_binding_override() -> anyhow::Result<()>
{
    let temp = bijux_dna_infra::temp_dir("fastq-merge-pairs-default-threads")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__merge_pairs_default_threads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.merge_pairs".to_string(),
            stage_instance_id: Some("fastq.merge_pairs.default".to_string()),
            tool: tool_with_threads("pear", 2),
            reason: None,
            params: None,
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("\"threads\": 6"));
    Ok(())
}

#[test]
fn planner_uses_typed_normalize_primers_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-normalize-primers-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let primer_fasta = temp.path().join("primers.fa");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&primer_fasta, b">p1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__normalize_primers__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.normalize_primers".to_string(),
            stage_instance_id: Some("fastq.normalize_primers.custom".to_string()),
            tool: tool("cutadapt"),
            reason: None,
            params: Some(FastqStageParameters::NormalizePrimers(NormalizePrimersStageParams {
                primer_set_id: "16s_v4".to_string(),
                marker_id: Some("16s".to_string()),
                primer_fasta: Some(primer_fasta.clone()),
                orientation_policy: "normalize_to_reverse_complement".to_string(),
                max_mismatch_rate: 0.05,
                min_overlap_bp: 14,
                strict_5p_anchor: false,
                allow_iupac_codes: false,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.normalize_primers.custom");
    assert!(step.command.template[2].contains("--overlap 14"));
    assert!(step.command.template[2].contains("--error-rate 0.05"));
    assert!(step.command.template[2].contains("\"primer_set_id\":\"16s_v4\""));
    assert!(step.command.template[2]
        .contains("\"orientation_policy\":\"normalize_to_reverse_complement\""));
    Ok(())
}

#[test]
fn planner_uses_typed_index_reference_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-index-reference-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__index_reference__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.index_reference".to_string(),
            stage_instance_id: Some("fastq.index_reference.custom".to_string()),
            tool: tool("bowtie2_build"),
            reason: None,
            params: Some(FastqStageParameters::IndexReference(IndexReferenceStageParams {
                threads: Some(7),
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.index_reference.custom");
    assert_eq!(step.resources.threads, 7);
    assert_eq!(step.command.template[0], "sh");
    assert_eq!(step.command.template[1], "-lc");
    assert!(step.command.template[2].contains("bowtie2-build --threads 7"));
    Ok(())
}

#[test]
fn planner_uses_manifest_default_index_reference_threads_without_binding_override(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-index-reference-default-threads")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&reference, b">chr1\nACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__index_reference_default_threads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.index_reference".to_string(),
            stage_instance_id: Some("fastq.index_reference.default".to_string()),
            tool: tool_with_threads("bowtie2_build", 9),
            reason: None,
            params: None,
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.resources.threads, 2);
    assert_eq!(step.command.template[0], "sh");
    assert_eq!(step.command.template[1], "-lc");
    assert!(step.command.template[2].contains("bowtie2-build --threads 2"));
    Ok(())
}

#[test]
fn planner_uses_typed_infer_asvs_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-infer-asvs-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__infer_asvs__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.infer_asvs".to_string(),
            stage_instance_id: Some("fastq.infer_asvs.custom".to_string()),
            tool: tool("dada2"),
            reason: None,
            params: Some(FastqStageParameters::InferAsvs(InferAsvsStageParams {
                denoising_method: "dada2".to_string(),
                pooling_mode: "pooled".to_string(),
                chimera_policy: "keep_candidates".to_string(),
                threads: Some(8),
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.infer_asvs.custom");
    assert_eq!(step.resources.threads, 8);
    assert!(step.command.template.windows(2).any(|pair| pair == ["--threads", "8"]));
    assert!(step.command.template.windows(2).any(|pair| pair == ["--pooling-mode", "pooled"]));
    assert!(step
        .command
        .template
        .windows(2)
        .any(|pair| pair == ["--chimera-policy", "keep_candidates"]));
    Ok(())
}

#[test]
fn planner_uses_manifest_default_infer_asvs_threads_without_binding_override() -> anyhow::Result<()>
{
    let temp = bijux_dna_infra::temp_dir("fastq-infer-asvs-default-threads")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__infer_asvs_default_threads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.infer_asvs".to_string(),
            stage_instance_id: Some("fastq.infer_asvs.default".to_string()),
            tool: tool_with_threads("dada2", 6),
            reason: None,
            params: None,
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.resources.threads, 1);
    assert!(step.command.template.windows(2).any(|pair| pair == ["--threads", "1"]));
    Ok(())
}

#[test]
fn planner_uses_typed_profile_read_lengths_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-profile-read-lengths-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__profile_read_lengths__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.profile_read_lengths".to_string(),
            stage_instance_id: Some("fastq.profile_read_lengths.custom".to_string()),
            tool: seqkit_stats_tool(),
            reason: None,
            params: Some(FastqStageParameters::ProfileReadLengths(FastqReadLengthProfileParams {
                schema_version: READ_LENGTH_PROFILE_SCHEMA_VERSION.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
                threads: 6,
                histogram_bins: 64,
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.profile_read_lengths.custom");
    assert_eq!(step.resources.threads, 6);
    assert_eq!(step.command.template[5], "6");
    assert!(step.io.outputs.iter().any(|artifact| artifact.name.as_str() == "report_json"));
    Ok(())
}

#[test]
fn planner_uses_typed_screen_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-screen-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__screen_taxonomy__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.screen_taxonomy".to_string(),
            stage_instance_id: Some("fastq.screen_taxonomy.custom".to_string()),
            tool: tool("kraken2"),
            reason: None,
            params: Some(FastqStageParameters::Screen(ScreenEffectiveParams {
                schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
                threads: 6,
                contaminant_db: Some("curated_taxonomy".to_string()),
                database_catalog_id: "metagenome_catalog".to_string(),
                database_artifact_id: "kraken2_pluspf".to_string(),
                database_build_id: Some("2026-03-01".to_string()),
                database_digest: Some("sha256:taxonomy".to_string()),
                database_namespace: Some("contaminant_screening".to_string()),
                database_scope: TaxonomyDatabaseScope::ReadScreening,
                classifier: TaxonomyClassifier::Kraken2,
                report_format: TaxonomyReportFormat::KrakenReport,
                assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
                minimum_confidence: Some(0.15),
                emit_unclassified: false,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.screen_taxonomy.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("\"database_catalog_id\":\"metagenome_catalog\""));
    assert!(step.command.template[2].contains("\"database_artifact_id\":\"kraken2_pluspf\""));
    assert!(step.command.template[2].contains("\"minimum_confidence\":0.15"));
    assert!(step.command.template[2].contains("\"emit_unclassified\":false"));
    Ok(())
}

#[test]
fn planner_uses_typed_profile_overrepresented_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-profile-overrepresented-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__profile_overrepresented_sequences__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.profile_overrepresented_sequences".to_string(),
            stage_instance_id: Some("fastq.profile_overrepresented_sequences.custom".to_string()),
            tool: tool("fastqc"),
            reason: None,
            params: Some(FastqStageParameters::ProfileOverrepresented(
                FastqOverrepresentedProfileParams {
                    schema_version: OVERREPRESENTED_PROFILE_SCHEMA_VERSION.to_string(),
                    paired_mode: bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
                    threads: 6,
                    top_k: 25,
                },
            )),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.profile_overrepresented_sequences.custom");
    assert_eq!(step.resources.threads, 6);
    assert_eq!(step.command.template[0], "sh");
    assert_eq!(step.command.template[1], "-lc");
    assert!(step.command.template[2].contains("'--threads' '6'"));
    Ok(())
}

#[test]
fn planner_uses_typed_profile_reads_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-profile-reads-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__profile_reads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.profile_reads".to_string(),
            stage_instance_id: Some("fastq.profile_reads.custom".to_string()),
            tool: seqkit_stats_tool(),
            reason: None,
            params: Some(FastqStageParameters::ProfileReads(FastqStatsParams {
                schema_version: STATS_SCHEMA_VERSION.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
                threads: 6,
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.profile_reads.custom");
    assert_eq!(step.resources.threads, 6);
    assert_eq!(step.command.template[5], "6");
    Ok(())
}

#[test]
fn planner_uses_typed_remove_duplicates_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-remove-duplicates-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;
    std::fs::write(&r2, b"@r1\nTTTT\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__remove_duplicates__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.remove_duplicates".to_string(),
            stage_instance_id: Some("fastq.remove_duplicates.custom".to_string()),
            tool: clumpify_tool(),
            reason: None,
            params: Some(FastqStageParameters::RemoveDuplicates(
                RemoveDuplicatesEffectiveParams {
                    schema_version:
                        bijux_dna_domain_fastq::params::remove_duplicates::REMOVE_DUPLICATES_SCHEMA_VERSION
                            .to_string(),
                    paired_mode: bijux_dna_domain_fastq::params::PairedMode::PairedEnd,
                    threads: 6,
                    dedup_mode: DedupMode::OpticalAware,
                    keep_order: false,
                },
            )),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.remove_duplicates.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("threads=6"));
    assert!(step.command.template[2].contains("optical dupedist=40"));
    assert!(step.command.template[2].contains("reorder=f"));
    Ok(())
}

#[test]
fn planner_uses_typed_remove_chimeras_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-remove-chimeras-stage-params")?;
    let r1 = temp.path().join("merged.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__remove_chimeras__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.remove_chimeras".to_string(),
            stage_instance_id: Some("fastq.remove_chimeras.custom".to_string()),
            tool: vsearch_tool(),
            reason: None,
            params: Some(FastqStageParameters::RemoveChimeras(
                ChimeraDetectionEffectiveParams {
                    method: "vsearch_uchime_denovo".to_string(),
                    detection_scope: "denovo".to_string(),
                    input_layout: "single_stream".to_string(),
                    threads: 6,
                    report_artifact: "report_json".to_string(),
                    metrics_artifact: "chimera_metrics_json".to_string(),
                    chimera_sequence_artifact: "chimeras_fasta".to_string(),
                    raw_backend_report_artifact: "uchime_report_tsv".to_string(),
                    raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
                    chimera_removed_definition:
                        "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                            .to_string(),
                    fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
                },
            )),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.remove_chimeras.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template.windows(2).any(|pair| pair == ["--threads", "6"]));
    Ok(())
}

#[test]
fn planner_uses_typed_report_qc_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-report-qc-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nAAAA\n+\n####\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__report_qc__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.profile_reads".to_string(),
                stage_instance_id: Some("fastq.profile_reads.metrics".to_string()),
                tool: seqkit_stats_tool(),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.report_qc".to_string(),
                stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                tool: tool("multiqc"),
                reason: None,
                params: Some(FastqStageParameters::ReportQc(QcPostEffectiveParams {
                    schema_version:
                        bijux_dna_domain_fastq::params::qc_post::REPORT_QC_SCHEMA_VERSION
                            .to_string(),
                    paired_mode: bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
                    aggregation_engine: QcAggregationEngine::Multiqc,
                    aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
                })),
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

    let step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.report_qc.aggregate")
        .expect("report_qc step");
    assert!(step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "fastq.profile_reads.metrics.qc_json"));
    assert!(step
        .io
        .outputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest"));
    Ok(())
}

#[test]
fn planner_uses_typed_trim_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-trim-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__trim_reads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.custom".to_string()),
            tool: tool("fastp"),
            reason: None,
            params: Some(FastqStageParameters::Trim(TrimEffectiveParams {
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
                threads: 6,
                min_len: 42,
                q_cutoff: Some(20),
                adapter_policy: "none".to_string(),
                damage_mode: None,
                polyx_policy: Some("trim".to_string()),
                n_policy: Some("drop".to_string()),
                contaminant_policy: Some("none".to_string()),
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.trim_reads.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("'--thread' '6'"));
    assert!(step.command.template[2].contains("\"min_length\":42"));
    assert!(step.command.template[2].contains("\"quality_cutoff\":20"));
    assert!(step.command.template[2].contains("\"polyx_policy\":\"trim\""));
    assert!(step.command.template[2].contains("\"n_policy\":\"drop\""));
    Ok(())
}

#[test]
fn planner_uses_typed_trim_polyg_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-trim-polyg-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__trim_polyg_tails__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.trim_polyg_tails".to_string(),
            stage_instance_id: Some("fastq.trim_polyg_tails.custom".to_string()),
            tool: tool("fastp"),
            reason: None,
            params: Some(FastqStageParameters::TrimPolygTails(TrimPolygTailsParams {
                schema_version: TRIM_POLYG_TAILS_SCHEMA_VERSION.to_string(),
                paired_mode: bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
                threads: 6,
                trim_polyg: false,
                min_polyg_run: 14,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.trim_polyg_tails.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(!step.command.template[2].contains("--trim_poly_g"));
    assert!(step.command.template[2].contains("\"trim_polyg\":false"));
    assert!(step.command.template[2].contains("\"min_polyg_run\":14"));
    Ok(())
}

#[test]
fn planner_uses_typed_correct_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-correct-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let trusted = temp.path().join("trusted.kmers");
    std::fs::write(&trusted, b"ACGT\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__correct_errors__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.correct_errors".to_string(),
            stage_instance_id: Some("fastq.correct_errors.custom".to_string()),
            tool: tool("lighter"),
            reason: None,
            params: Some(FastqStageParameters::CorrectErrors(CorrectErrorsStageParams {
                threads: Some(6),
                quality_encoding: QualityEncoding::Phred33,
                kmer_size: Some(31),
                musket_kmer_budget: None,
                genome_size: Some(2_500_000),
                max_memory_gb: None,
                trusted_kmer_artifact: Some(trusted.clone()),
                conservative_mode: false,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.correct_errors.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template[2].contains("\"threads\":6"));
    assert!(step.command.template[2].contains("\"kmer_size\":31"));
    assert!(step.command.template[2].contains("\"genome_size\":2500000"));
    assert!(step.command.template[2].contains("\"trusted_kmer_artifact\":"));
    assert_eq!(step.expected_artifact_ids[0].as_str(), "corrected_reads_r1");
    Ok(())
}

#[test]
fn planner_uses_manifest_default_correct_errors_threads_without_binding_override(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-correct-default-threads")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__correct_errors_default_threads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.correct_errors".to_string(),
            stage_instance_id: Some("fastq.correct_errors.default".to_string()),
            tool: tool_with_threads("rcorrector", 8),
            reason: None,
            params: None,
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.resources.threads, 1);
    assert!(step.command.template[2].contains("\"threads\":1"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.deplete_rrna".to_string(),
            stage_instance_id: Some("fastq.deplete_rrna.sortmerna.custom".to_string()),
            tool: tool("sortmerna"),
            reason: None,
            params: Some(FastqStageParameters::DepleteRrna(DepleteRrnaStageParams {
                rrna_db: "silva_nr99".to_string(),
                min_identity: 0.95,
                threads: Some(6),
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.deplete_rrna.sortmerna.custom");
    assert_eq!(step.command.template[0], "bash");
    assert!(step.command.template[2].contains("--ref"));
    assert!(step.command.template.get(2).is_some_and(|part| part.contains("silva_nr99")));
    assert!(step.command.template.get(2).is_some_and(|part| part.contains("--workdir")));
    assert!(step.command.template.get(2).is_some_and(|part| part.contains("'6'")));
    assert!(step.command.template.get(2).is_some_and(|part| !part.contains("--report")));
    Ok(())
}

#[test]
fn planner_rejects_unsupported_rrna_identity_override_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-deplete-rrna-unsupported-identity")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__deplete_rrna__identity_guard__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.deplete_rrna".to_string(),
            stage_instance_id: None,
            tool: tool("sortmerna"),
            reason: None,
            params: Some(FastqStageParameters::DepleteRrna(DepleteRrnaStageParams {
                rrna_db: "silva_nr99".to_string(),
                min_identity: 0.99,
                threads: None,
            })),
        }],
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
    })
    .expect_err("unsupported rrna min_identity override must fail");

    assert!(error.to_string().contains("does not support governed min_identity overrides"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
                    threads: None,
                })),
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
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
                        threads: None,
                    },
                )),
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
        reference_fasta: Some(reference),
        out_dir: temp.path().join("out"),
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
                    to_input_id: Some("qc_artifacts".to_string()),
                },
                PipelineEdgeSpec {
                    from: "fastq.detect_adapters.fastqc".to_string(),
                    to: "fastq.report_qc.aggregate".to_string(),
                    from_output_id: Some("adapter_report".to_string()),
                    to_input_id: Some("qc_artifacts".to_string()),
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

    assert_eq!(plan.edges().len(), 3);
    assert!(plan.edges().iter().all(|edge| edge.to().as_str() == "fastq.report_qc.aggregate"));
    let edge_bindings = plan
        .edges()
        .iter()
        .map(|edge| {
            (
                edge.from().as_str().to_string(),
                edge.from_output_id().map(|artifact| artifact.as_str().to_string()),
                edge.to_input_id().map(|artifact| artifact.as_str().to_string()),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        edge_bindings,
        std::collections::BTreeSet::from([
            (
                "fastq.detect_adapters.fastqc".to_string(),
                Some("adapter_report".to_string()),
                Some("qc_artifacts".to_string()),
            ),
            (
                "fastq.validate_reads.validator".to_string(),
                Some("validated_reads_manifest".to_string()),
                Some("validated_reads_manifest".to_string()),
            ),
            (
                "fastq.validate_reads.validator".to_string(),
                Some("validation_report".to_string()),
                Some("qc_artifacts".to_string()),
            ),
        ])
    );
    let report_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.report_qc.aggregate")
        .expect("report_qc step");
    assert_eq!(report_step.io.inputs.len(), 3);
    assert!(report_step.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.validate_reads.validator.validation_report"
    }));
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "validated_reads_manifest"));
    assert!(report_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str() == "fastq.detect_adapters.fastqc.adapter_report"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
    })
    .expect_err("selection rejoin should require artifact-bound edges");

    assert!(error.to_string().contains("requires artifact-bound outgoing rejoin edges"));
    Ok(())
}

#[test]
fn planner_uses_typed_filter_reads_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-filter-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__filter_reads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.filter_reads".to_string(),
            stage_instance_id: Some("fastq.filter_reads.custom".to_string()),
            tool: tool_with_threads("fastp", 4),
            reason: None,
            params: Some(FastqStageParameters::FilterReads(FilterReadsStageParams {
                threads: Some(7),
                max_n: None,
                max_n_fraction: None,
                max_n_count: Some(2),
                low_complexity_threshold: None,
                entropy_threshold: Some(0.25),
                kmer_ref: None,
                polyx_policy: None,
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.filter_reads.custom");
    assert_eq!(step.resources.threads, 7);
    assert!(step.command.template.windows(2).any(|window| window == ["--thread", "7"]));
    assert!(step.command.template.windows(2).any(|window| window == ["--n_base_limit", "2"]));
    assert!(step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--complexity_threshold", "0.25"]));
    Ok(())
}

#[test]
fn planner_uses_typed_filter_low_complexity_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-filter-low-complexity-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__filter_low_complexity__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.filter_low_complexity".to_string(),
            stage_instance_id: Some("fastq.filter_low_complexity.custom".to_string()),
            tool: tool("bbduk"),
            reason: None,
            params: Some(FastqStageParameters::FilterLowComplexity(
                FilterLowComplexityStageParams {
                    entropy_threshold: Some(0.82),
                    polyx_threshold: Some(24),
                },
            )),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.filter_low_complexity.custom");
    assert!(step.command.template.iter().any(|token| token == "entropy=0.82"));
    assert!(step.command.template.iter().any(|token| token == "maxpoly=24"));
    Ok(())
}

#[test]
fn planner_uses_typed_extract_umis_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-extract-umis-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r2\nT\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__extract_umis__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.extract_umis".to_string(),
            stage_instance_id: Some("fastq.extract_umis.custom".to_string()),
            tool: umi_tools_tool(2),
            reason: None,
            params: Some(FastqStageParameters::ExtractUmis(ExtractUmisStageParams {
                threads: Some(5),
                umi_pattern: Some("NNNNCCCC".to_string()),
                extraction_location: None,
                read_name_transform: None,
                failed_extraction_policy: None,
                grouping_policy: Some("exact_header_tag".to_string()),
                downstream_dedup_policy: Some("coordinate_aware_recommended".to_string()),
                downstream_propagation: None,
            })),
        }],
        stage_toolsets: Vec::new(),
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: Some(r2),
        reference_fasta: None,
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.extract_umis.custom");
    assert_eq!(step.resources.threads, 5);
    assert!(step.command.template.iter().any(|token| token == "NNNNCCCC"));
    Ok(())
}

#[test]
fn planner_uses_typed_detect_adapters_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-detect-adapters-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__detect_adapters__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.detect_adapters".to_string(),
            stage_instance_id: Some("fastq.detect_adapters.custom".to_string()),
            tool: tool_with_threads("fastqc", 1),
            reason: None,
            params: Some(FastqStageParameters::DetectAdapters(DetectAdaptersStageParams {
                threads: Some(3),
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.detect_adapters.custom");
    assert_eq!(step.resources.threads, 3);
    assert_eq!(step.command.template[0], "sh");
    assert_eq!(step.command.template[1], "-lc");
    assert!(step.command.template[2].contains("'--threads' '3'"));
    Ok(())
}

#[test]
fn planner_uses_typed_cluster_otus_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-cluster-otus-stage-params")?;
    let r1 = temp.path().join("reads.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__cluster_otus__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.cluster_otus".to_string(),
            stage_instance_id: Some("fastq.cluster_otus.custom".to_string()),
            tool: vsearch_tool(),
            reason: None,
            params: Some(FastqStageParameters::ClusterOtus(ClusterOtusStageParams {
                otu_identity: 0.99,
                threads: Some(6),
            })),
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.step_id.as_str(), "fastq.cluster_otus.custom");
    assert_eq!(step.resources.threads, 6);
    assert!(step.command.template.windows(2).any(|window| window == ["--threads", "6"]));
    Ok(())
}

#[test]
fn planner_uses_manifest_default_cluster_otus_threads_without_binding_override(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-cluster-otus-default-threads")?;
    let r1 = temp.path().join("reads.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__cluster_otus_default_threads__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: None,
        stage_bindings: vec![FastqStageBinding {
            stage_id: "fastq.cluster_otus".to_string(),
            stage_instance_id: Some("fastq.cluster_otus.default".to_string()),
            tool: vsearch_tool(),
            reason: None,
            params: None,
        }],
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

    let step = &plan.steps()[0];
    assert_eq!(step.resources.threads, 4);
    assert!(step.command.template.windows(2).any(|window| window == ["--threads", "4"]));
    Ok(())
}

#[test]
fn planner_supports_stage_toolsets_with_select_nodes() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-select-toolsets")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__select_toolsets__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Retention,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
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
                    from: "fastq.trim_reads.cleanup".to_string(),
                    to: "benchmark.select_stage_tool.trim_reads".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("candidate_trimmed_reads_r1".to_string()),
                },
                PipelineEdgeSpec {
                    from: "benchmark.select_stage_tool.trim_reads".to_string(),
                    to: "fastq.filter_reads.selected".to_string(),
                    from_output_id: Some("trimmed_reads_r1".to_string()),
                    to_input_id: Some("reads_r1".to_string()),
                },
            ],
        )),
        stage_bindings: Vec::new(),
        stage_toolsets: vec![
            FastqStageToolsetBinding {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
                tools: vec![tool("fastp"), tool("cutadapt")],
                reason: None,
                params: None,
            },
            FastqStageToolsetBinding {
                stage_id: "fastq.filter_reads".to_string(),
                stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
                tools: vec![tool("seqkit")],
                reason: None,
                params: None,
            },
        ],
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

    let select_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "benchmark.select_stage_tool.trim_reads")
        .expect("select step");
    let filter_step = plan
        .steps()
        .iter()
        .find(|step| {
            step.stage_id.as_str() == "fastq.filter_reads"
                && !step.step_id.as_str().contains("fastq.trim_reads.cleanup=")
        })
        .expect("collapsed downstream stage");
    assert_eq!(select_step.io.inputs.len(), 2);
    assert_eq!(select_step.io.outputs.len(), 1);
    assert!(
        select_step
            .command
            .template
            .windows(2)
            .any(|window| window == ["--objective", "retention"]),
        "select command must carry the configured ranking objective"
    );
    assert_eq!(filter_step.io.inputs[0].path, select_step.io.outputs[0].path);
    assert!(plan.edges().iter().any(|edge| {
        edge.from().as_str() == "benchmark.select_stage_tool.trim_reads"
            && edge.to().as_str() == filter_step.step_id.as_str()
            && edge.from_output_id().is_some()
            && edge.to_input_id().is_some()
    }));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let host_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.deplete_host.host")
        .expect("host depletion stage");
    assert!(host_step.command.template.iter().any(|part| part.contains("index_reference.host")));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })?;

    let host_step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.deplete_host.host")
        .expect("host depletion stage");
    assert!(host_step.command.template.iter().any(|part| part.contains("index_reference.host")));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        reference_fasta: Some(reference_fasta),
        out_dir: temp.path().join("out"),
        allow_planned: false,
    })
    .expect_err("ambiguous dependency should fail");

    assert!(error.to_string().contains("multiple fastq.index_reference nodes"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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
    assert_eq!(abundance_step.io.inputs[0].path, cluster_step.io.outputs[0].path);
    Ok(())
}

#[test]
fn planner_uses_typed_normalize_abundance_params_from_stage_binding() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-normalize-abundance-stage-params")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plan = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__normalize_abundance__v1".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.cluster_otus".to_string(),
                    stage_instance_id: Some("fastq.cluster_otus.source".to_string()),
                },
                PipelineNodeSpec {
                    stage_id: "fastq.normalize_abundance".to_string(),
                    stage_instance_id: Some("fastq.normalize_abundance.custom".to_string()),
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.cluster_otus.source".to_string(),
                to: "fastq.normalize_abundance.custom".to_string(),
                from_output_id: Some("otu_table".to_string()),
                to_input_id: Some("abundance_table".to_string()),
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.cluster_otus".to_string(),
                stage_instance_id: Some("fastq.cluster_otus.source".to_string()),
                tool: tool("vsearch"),
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.normalize_abundance".to_string(),
                stage_instance_id: Some("fastq.normalize_abundance.custom".to_string()),
                tool: tool("seqkit"),
                reason: None,
                params: Some(FastqStageParameters::NormalizeAbundance(
                    NormalizeAbundanceStageParams { method: "counts_per_million".to_string() },
                )),
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

    let step = plan
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.normalize_abundance.custom")
        .expect("normalize abundance step");
    assert!(step.command.template[2].contains("counts_per_million"));
    assert!(step.command.template[2].contains("per_sample_sum_to_one_million"));
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
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
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

    assert_eq!(plan.edges().len(), 1);
    assert_eq!(plan.edges()[0].from().as_str(), "fastq.validate_reads.tool.fastqvalidator");
    assert_eq!(plan.edges()[0].to().as_str(), "fastq.detect_adapters.tool.fastqc");
    Ok(())
}
