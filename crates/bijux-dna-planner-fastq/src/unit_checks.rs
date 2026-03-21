use super::*;
use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;

#[test]
fn select_trim_tools_dedup_and_sort() {
    let tools = vec![
        "fastp".to_string(),
        "FASTP".to_string(),
        "bbduk".to_string(),
    ];
    match select_trim_tools(&tools, false) {
        Ok(normalized) => {
            assert_eq!(normalized, vec!["bbduk".to_string(), "fastp".to_string()]);
        }
        Err(err) => panic!("normalize failed: {err}"),
    }
}

#[test]
fn select_trim_tools_rejects_tools_outside_execution_support() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_trim_tools_keeps_contract_even_when_opt_in_flag_is_set() {
    let tools = vec!["seqpurge".to_string()];
    match select_trim_tools(&tools, true) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_tools_rejects_empty() {
    match select_validate_tools(&[]) {
        Ok(_) => panic!("expected empty failure"),
        Err(err) => assert!(err.to_string().contains("no tools specified")),
    }
}

#[test]
fn stage_status_comes_from_domain_execution_support() {
    assert_eq!(
        stage_status("fastq.validate_reads").as_deref(),
        Some("supported")
    );
    assert_eq!(stage_status("fastq.infer_asvs").as_deref(), Some("planned"));
    assert!(stage_status("fastq.unknown_stage").is_none());
}

#[test]
fn benchmark_query_context_uses_stage_contract_hash_for_governed_stages() {
    let context = bench_query_context_for_stage(&StageId::from_static("fastq.trim_reads"))
        .expect("governed stage contract hash should be available");

    assert!(context.params_hash.is_none());
    assert!(context.image_digest.is_none());
    assert!(context.stage_contract_hash.is_some());
}

#[test]
fn benchmark_query_context_stays_empty_for_unknown_stages() {
    let context = bench_query_context_for_stage(&StageId::new("fastq.unknown".to_string()))
        .expect("unknown stages should not fail benchmark query context construction");

    assert!(context.is_empty());
}

#[test]
fn preprocess_benchmark_query_context_tracks_lineage_hash() {
    let args = crate::selection::args::BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: None,
        r1: "reads_R1.fastq.gz".into(),
        r2: None,
        reference_fasta: None,
        out: "out".into(),
        strict: false,
        auto: true,
        objective: bijux_dna_core::contract::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: false,
        replicates: 1,
        jobs: 1,
        ci_bootstrap: None,
        adapter_bank_preset: None,
        adapter_bank: None,
        adapter_bank_file: None,
        enable_adapters: Vec::new(),
        disable_adapters: Vec::new(),
        polyx_preset: None,
        contaminant_preset: None,
        enable_contaminant_removal: false,
        no_qc_post: false,
        force_merge: false,
        enable_correct: false,
        run_all_governed_tools: false,
        allow_planned: false,
        mode: crate::selection::args::FastqPlannerMode::Shotgun,
    };
    let prior_tools = vec![StageToolSelection {
        stage_id: "fastq.validate_reads".to_string(),
        stage_instance_id: None,
        tool_id: "fastqvalidator".to_string(),
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
    }];

    let context = bench_query_context_for_preprocess_stage(
        &StageId::from_static("fastq.trim_reads"),
        &args,
        &["fastq.validate_reads".to_string()],
        &prior_tools,
    )
    .expect("preprocess query context should serialize prior lineage");

    assert_eq!(
        context.lineage_hash.as_deref(),
        Some("fastq.validate_reads=fastqvalidator")
    );
}

#[test]
fn stage_tool_capability_no_longer_treats_planned_bindings_as_plannable() {
    let capability = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    )
    .expect("declared FASTQ ASV binding must still surface a capability row");

    assert!(capability.declared);
    assert!(!capability.plannable);
    assert!(!capability.runnable);
    assert!(!capability.parse_normalized);
    assert!(!capability.benchmark_normalized);
    assert!(!capability.comparable);
}

#[test]
fn stage_tool_capability_uses_manifest_normalization_modes() {
    let detect_adapters = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("fastqc detect-adapters capability must exist");
    assert!(detect_adapters.parse_normalized);
    assert!(detect_adapters.benchmark_normalized);
    assert!(detect_adapters.comparable);

    let trim_reads = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    )
    .expect("fastp trim capability must exist");
    assert!(trim_reads.parse_normalized);
    assert!(!trim_reads.benchmark_normalized);
    assert!(!trim_reads.comparable);
}

#[test]
fn mixed_normalization_stages_only_mark_observer_specialized_tools_comparable() {
    let fastqc = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.profile_overrepresented_sequences"),
        &ToolId::from_static("fastqc"),
    )
    .expect("fastqc overrepresented capability must exist");
    let seqkit = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.profile_overrepresented_sequences"),
        &ToolId::from_static("seqkit"),
    )
    .expect("seqkit overrepresented capability must exist");

    assert!(fastqc.parse_normalized);
    assert!(fastqc.benchmark_normalized);
    assert!(fastqc.comparable);

    assert!(seqkit.runnable);
    assert!(!seqkit.parse_normalized);
    assert!(!seqkit.benchmark_normalized);
    assert!(!seqkit.comparable);
}

#[test]
fn normalize_abundance_rejects_planned_seqfu_backend() {
    let error = crate::plan_compose::ensure_normalize_abundance_tool("seqfu")
        .expect_err("seqfu should stay outside the governed normalize_abundance runtime");

    assert!(error.to_string().contains("requires seqkit"));
}

#[test]
fn detect_adapters_surfaces_observer_specialized_benchmark_profile() {
    let profile = crate::stage_api::benchmark_profile_for_stage_tool(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("detect_adapters benchmark profile must exist");

    assert_eq!(
        profile.readiness,
        crate::stage_api::BenchmarkReadinessLevel::ObserverSpecializedBenchmark
    );
    assert_eq!(
        profile.benchmark_scenarios,
        vec!["detect_adapters_fairness"]
    );
}

#[test]
fn host_depletion_surfaces_benchmark_cohort_from_manifest_support() {
    let cohorts =
        crate::stage_api::benchmark_cohorts_for_stage(&StageId::from_static("fastq.deplete_host"));

    assert_eq!(cohorts.len(), 1);
    assert_eq!(cohorts[0].scenario_id, "host_depletion_fairness");
    assert!(cohorts[0].tool_ids.is_empty());
}

#[test]
fn expand_pipeline_stage_tool_routes_materializes_graph_bound_stage_bindings() {
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
    let toolsets = vec![
        ToolsetSelection {
            stage_id: "fastq.validate_reads".to_string(),
            stage_instance_id: Some("fastq.validate_reads.entry".to_string()),
            tool_ids: vec!["fastqvalidator".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
        ToolsetSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            tool_ids: vec!["bbduk".to_string(), "fastp".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
    ];

    let (expanded_pipeline, expanded_stage_tools) =
        expand_pipeline_stage_tool_routes(&pipeline, &toolsets).expect("expand routes");

    assert_eq!(expanded_pipeline.nodes.len(), 4);
    assert_eq!(expanded_pipeline.edges.len(), 2);
    assert_eq!(expanded_stage_tools.len(), 4);
    assert!(expanded_stage_tools.iter().any(|selection| {
        selection.stage_id == "fastq.trim_reads"
            && selection.tool_id == "bbduk"
            && selection
                .stage_instance_id
                .as_deref()
                .is_some_and(|id| id.contains("fastq.validate_reads.entry=fastqvalidator"))
    }));
    assert!(expanded_pipeline.edges.iter().all(|edge| {
        edge.from.contains("fastq.validate_reads.entry=fastqvalidator")
            && edge.to.contains("fastq.validate_reads.entry=fastqvalidator")
    }));
}

#[test]
fn expand_pipeline_stage_tool_routes_keys_repeated_stage_nodes_by_instance_id() {
    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.left".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.right".to_string()),
            },
        ],
        Vec::new(),
    );
    let toolsets = vec![
        ToolsetSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.left".to_string()),
            tool_ids: vec!["fastp".to_string(), "cutadapt".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
        ToolsetSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.right".to_string()),
            tool_ids: vec!["bbduk".to_string(), "seqkit".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
    ];

    let (expanded_pipeline, expanded_stage_tools) =
        expand_pipeline_stage_tool_routes(&pipeline, &toolsets).expect("expand routes");

    assert_eq!(expanded_pipeline.nodes.len(), 8);
    assert_eq!(expanded_stage_tools.len(), 8);
    assert!(expanded_stage_tools.iter().any(|selection| {
        selection
            .stage_instance_id
            .as_deref()
            .is_some_and(|id| id.contains("fastq.trim_reads.left=fastp"))
    }));
    assert!(expanded_stage_tools.iter().any(|selection| {
        selection
            .stage_instance_id
            .as_deref()
            .is_some_and(|id| id.contains("fastq.trim_reads.right=bbduk"))
    }));
}

#[test]
fn expand_pipeline_stage_tool_routes_rejects_excessive_route_counts() {
    let pipeline = PipelineSpec::linear(vec![
        "fastq.validate_reads".to_string(),
        "fastq.trim_reads".to_string(),
        "fastq.filter_reads".to_string(),
        "fastq.remove_duplicates".to_string(),
        "fastq.profile_reads".to_string(),
    ]);
    let toolsets = pipeline
        .ordered_nodes()
        .into_iter()
        .map(|node| ToolsetSelection {
            stage_id: node.stage_id,
            stage_instance_id: node.stage_instance_id,
            tool_ids: (0..4).map(|idx| format!("tool_{idx}")).collect(),
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        })
        .collect::<Vec<_>>();

    let error = expand_pipeline_stage_tool_routes(&pipeline, &toolsets)
        .expect_err("route expansion should reject unbounded graph fanout");
    assert!(error
        .to_string()
        .contains("preprocess tool route expansion would create"));
}

#[test]
fn expand_pipeline_stage_tool_routes_rejects_planner_owned_select_nodes() {
    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "benchmark.select_stage_tool".to_string(),
                stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
            },
        ],
        vec![PipelineEdgeSpec {
            from: "fastq.trim_reads.cleanup".to_string(),
            to: "benchmark.select_stage_tool.trim_reads".to_string(),
            from_output_id: Some("trimmed_reads_r1".to_string()),
            to_input_id: Some("fastp_trimmed_reads_r1".to_string()),
        }],
    );
    let toolsets = vec![ToolsetSelection {
        stage_id: "fastq.trim_reads".to_string(),
        stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
        tool_ids: vec!["fastp".to_string(), "cutadapt".to_string()],
        reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
    }];

    let error = expand_pipeline_stage_tool_routes(&pipeline, &toolsets)
        .expect_err("planner-owned select nodes must not enter route expansion");
    assert!(error
        .to_string()
        .contains("toolset route expansion does not support planner-owned node"));
}

#[test]
fn reference_aware_depletion_rejects_incompatible_index_bindings_early() {
    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__host_depletion__v1".to_string(),
        policy: bijux_dna_core::contract::PlanPolicy::default(),
        pipeline_spec: None,
        stage_bindings: Vec::new(),
        stage_toolsets: Vec::new(),
        stages: vec![
            "fastq.index_reference".to_string(),
            "fastq.deplete_host".to_string(),
        ],
        tools: vec![
            ToolExecutionSpecV1 {
                tool_id: ToolId::from_static("star"),
                tool_version: "test".to_string(),
                image: bijux_dna_core::prelude::ContainerImageRefV1 {
                    image: "bijux/test".to_string(),
                    digest: None,
                },
                command: bijux_dna_core::prelude::CommandSpecV1 {
                    template: vec!["star".to_string()],
                },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            ToolExecutionSpecV1 {
                tool_id: ToolId::from_static("bowtie2"),
                tool_version: "test".to_string(),
                image: bijux_dna_core::prelude::ContainerImageRefV1 {
                    image: "bijux/test".to_string(),
                    digest: None,
                },
                command: bijux_dna_core::prelude::CommandSpecV1 {
                    template: vec!["bowtie2".to_string()],
                },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
        ],
        aux_images: std::collections::BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: Some(std::path::PathBuf::from("reference.fa")),
        out_dir: std::path::PathBuf::from("out"),
        tool_reasons: None,
        allow_planned: false,
    })
    .expect_err("incompatible reference backends should fail before stage composition");

    assert!(error
        .to_string()
        .contains("requires one of [bowtie2_build]"));
}
