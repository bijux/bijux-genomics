use super::graph_policy::stage_status;
use super::selection_planning::{
    bench_query_context_for_preprocess_stage, bench_query_context_for_stage,
};
use super::*;
use crate::{resolve_preprocess_pipeline, PreprocessDecisions};
use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_core::id_catalog;
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolExecutionSpecV1};

#[test]
fn select_trim_tools_dedup_and_sort() {
    let tools = vec!["fastp".to_string(), "FASTP".to_string(), "bbduk".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(normalized) => {
            assert_eq!(normalized, vec!["bbduk".to_string(), "fastp".to_string()]);
        }
        Err(err) => panic!("normalize failed: {err}"),
    }
}

#[test]
fn select_trim_tools_rejects_unknown_tools() {
    let tools = vec!["not_a_tool".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_trim_tools_keeps_contract_even_when_opt_in_flag_is_set() {
    let tools = vec!["not_a_tool".to_string()];
    match select_trim_tools(&tools, true) {
        Ok(_) => panic!("expected failure"),
        Err(err) => assert!(err.to_string().contains("unsupported tool")),
    }
}

#[test]
fn select_trim_tools_accepts_newly_governed_adapter_backends() {
    let tools = vec!["skewer".to_string(), "leehom".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(normalized) => {
            assert_eq!(normalized, vec!["leehom".to_string(), "skewer".to_string()]);
        }
        Err(err) => panic!("expected governed trim tools to normalize: {err}"),
    }
}

#[test]
fn select_trim_tools_accepts_newly_governed_streaming_backends() {
    let tools = vec!["seqkit".to_string(), "prinseq".to_string()];
    match select_trim_tools(&tools, false) {
        Ok(normalized) => assert_eq!(normalized, vec!["prinseq".to_string(), "seqkit".to_string()]),
        Err(err) => panic!("expected governed trim tools to normalize: {err}"),
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
    assert_eq!(stage_status("fastq.validate_reads").as_deref(), Some("supported"));
    assert_eq!(stage_status("fastq.infer_asvs").as_deref(), Some("supported"));
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
    let context = bench_query_context_for_stage(&StageId::new(format!(
        "{}unknown",
        id_catalog::FASTQ_PREFIX
    )))
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

    assert_eq!(context.lineage_hash.as_deref(), Some("fastq.validate_reads=fastqvalidator"));
}

#[test]
fn validate_manifest_lineage_becomes_artifact_bound_execution_edge() {
    let bindings = vec![
        FastqStageBinding {
            stage_id: "fastq.validate_reads".to_string(),
            stage_instance_id: Some("fastq.validate_reads.fastqvalidator".to_string()),
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::from_static(id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL),
                tool_version: "fixture".to_string(),
                image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
                command: CommandSpecV1 {
                    template: vec![
                        "fastqvalidator".to_string(),
                        "--file".to_string(),
                        "{{reads}}".to_string(),
                    ],
                },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::from_static(id_catalog::TOOL_FASTP),
                tool_version: "fixture".to_string(),
                image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
                command: CommandSpecV1 { template: vec!["fastp".to_string()] },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            reason: None,
            params: None,
        },
    ];
    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.validate_reads".to_string(),
                stage_instance_id: Some("fastq.validate_reads.fastqvalidator".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
            },
        ],
        vec![PipelineEdgeSpec {
            from: "fastq.validate_reads.fastqvalidator".to_string(),
            to: "fastq.trim_reads.fastp".to_string(),
            from_output_id: None,
            to_input_id: None,
        }],
    );

    let plans = crate::compose::compose_fastq_stage_bindings_with_dependencies(
        &bindings,
        &std::collections::BTreeMap::new(),
        None,
        None,
        None,
        false,
        std::path::Path::new("reads.fastq.gz"),
        None,
        None,
        Some(&stage_artifact_input_policy(&pipeline)),
        None,
        Some(&stage_dependency_policy(&pipeline)),
        |binding, _r1, _r2| Ok(std::path::PathBuf::from("out").join(&binding.stage_id)),
    )
    .expect("compose plans");
    let edges =
        execution_edges_for_stage_plans(&pipeline, &plans, &std::collections::BTreeMap::new())
            .expect("derive execution edges");

    assert!(edges.iter().any(|edge| {
        edge.from().as_str() == "fastq.validate_reads.fastqvalidator"
            && edge.to().as_str() == "fastq.trim_reads.fastp"
            && edge
                .from_output_id()
                .is_some_and(|artifact| artifact.as_str() == "validated_reads_manifest")
            && edge
                .to_input_id()
                .is_some_and(|artifact| artifact.as_str() == "validated_reads_manifest")
    }));
    assert!(!edges.iter().any(|edge| {
        edge.from().as_str() == "fastq.validate_reads.fastqvalidator"
            && edge.to().as_str() == "fastq.trim_reads.fastp"
            && edge.from_output_id().is_none()
            && edge.to_input_id().is_none()
    }));
}

#[test]
fn report_qc_preserves_multiple_explicit_qc_artifact_bindings() {
    let bindings = vec![
        FastqStageBinding {
            stage_id: "fastq.detect_adapters".to_string(),
            stage_instance_id: Some("fastq.detect_adapters.fastqc".to_string()),
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::from_static(id_catalog::TOOL_FASTQC),
                tool_version: "fixture".to_string(),
                image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
                command: CommandSpecV1 { template: vec!["fastqc".to_string()] },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.profile_reads".to_string(),
            stage_instance_id: Some("fastq.profile_reads.seqkit_stats".to_string()),
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
                tool_version: "fixture".to_string(),
                image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
                command: CommandSpecV1 {
                    template: vec!["seqkit".to_string(), "stats".to_string()],
                },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            reason: None,
            params: None,
        },
        FastqStageBinding {
            stage_id: "fastq.report_qc".to_string(),
            stage_instance_id: Some("fastq.report_qc.multiqc".to_string()),
            tool: ToolExecutionSpecV1 {
                tool_id: ToolId::from_static(id_catalog::TOOL_MULTIQC),
                tool_version: "fixture".to_string(),
                image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
                command: CommandSpecV1 { template: vec!["multiqc".to_string()] },
                resources: bijux_dna_core::contract::ToolConstraints::default(),
            },
            reason: None,
            params: None,
        },
    ];
    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.detect_adapters".to_string(),
                stage_instance_id: Some("fastq.detect_adapters.fastqc".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.profile_reads".to_string(),
                stage_instance_id: Some("fastq.profile_reads.seqkit_stats".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.report_qc".to_string(),
                stage_instance_id: Some("fastq.report_qc.multiqc".to_string()),
            },
        ],
        vec![
            PipelineEdgeSpec {
                from: "fastq.detect_adapters.fastqc".to_string(),
                to: "fastq.report_qc.multiqc".to_string(),
                from_output_id: Some("adapter_evidence_dir".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
            PipelineEdgeSpec {
                from: "fastq.profile_reads.seqkit_stats".to_string(),
                to: "fastq.report_qc.multiqc".to_string(),
                from_output_id: Some("qc_json".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
        ],
    );

    let plans = crate::compose::compose_fastq_stage_bindings_with_dependencies(
        &bindings,
        &std::collections::BTreeMap::new(),
        None,
        None,
        None,
        false,
        std::path::Path::new("reads.fastq.gz"),
        None,
        None,
        Some(&stage_artifact_input_policy(&pipeline)),
        None,
        Some(&stage_dependency_policy(&pipeline)),
        |binding, _r1, _r2| Ok(std::path::PathBuf::from("out").join(&binding.stage_id)),
    )
    .expect("compose plans");

    let report_plan = plans
        .iter()
        .find(|plan| plan.stage_id.as_str() == "fastq.report_qc")
        .expect("report_qc stage plan");

    assert!(report_plan.io.inputs.iter().any(|artifact| {
        artifact.name.as_str() == "fastq.detect_adapters.fastqc.adapter_evidence_dir"
    }));
    assert!(report_plan
        .io
        .inputs
        .iter()
        .any(|artifact| { artifact.name.as_str() == "fastq.profile_reads.seqkit_stats.qc_json" }));
    assert_eq!(report_plan.params["qc_input_count"], serde_json::json!(2));
}

#[test]
fn stage_tool_capability_surfaces_current_infer_asvs_runtime_contract() {
    let capability = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.infer_asvs"),
        &ToolId::from_static("dada2"),
    )
    .expect("FASTQ ASV binding must surface the governed runtime capability row");

    assert!(capability.execution.declared);
    assert!(capability.execution.plannable);
    assert!(capability.execution.runnable);
    assert!(capability.normalization.parse_normalized);
    assert!(!capability.normalization.benchmark_normalized);
    assert!(!capability.normalization.comparable);
}

#[test]
fn stage_tool_capability_uses_manifest_normalization_modes() {
    let detect_adapters = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.detect_adapters"),
        &ToolId::from_static("fastqc"),
    )
    .expect("fastqc detect-adapters capability must exist");
    assert!(detect_adapters.normalization.parse_normalized);
    assert!(detect_adapters.normalization.benchmark_normalized);
    assert!(detect_adapters.normalization.comparable);

    let trim_reads = crate::stage_api::stage_tool_capability(
        &StageId::from_static("fastq.trim_reads"),
        &ToolId::from_static("fastp"),
    )
    .expect("fastp trim capability must exist");
    assert!(trim_reads.normalization.parse_normalized);
    assert!(trim_reads.normalization.benchmark_normalized);
    assert!(!trim_reads.normalization.comparable);
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

    assert!(fastqc.normalization.parse_normalized);
    assert!(fastqc.normalization.benchmark_normalized);
    assert!(fastqc.normalization.comparable);

    assert!(seqkit.execution.runnable);
    assert!(seqkit.normalization.parse_normalized);
    assert!(seqkit.normalization.benchmark_normalized);
    assert!(seqkit.normalization.comparable);
}

#[test]
fn normalize_abundance_rejects_planned_seqfu_backend() {
    let error = crate::compose::ensure_normalize_abundance_tool("seqfu")
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
    assert_eq!(profile.benchmark_scenarios, vec!["detect_adapters_fairness"]);
}

#[test]
fn host_depletion_surfaces_benchmark_cohort_from_manifest_support() {
    let cohorts =
        crate::stage_api::benchmark_cohorts_for_stage(&StageId::from_static("fastq.deplete_host"));

    assert_eq!(cohorts.len(), 1);
    assert_eq!(cohorts[0].scenario_id, "host_depletion_fairness");
    assert_eq!(cohorts[0].tool_ids, vec![ToolId::from_static("bowtie2")]);
}

#[test]
fn governed_stage_benchmark_cohorts_surface_declared_toolsets() {
    let cases = [
        ("fastq.cluster_otus", "vsearch"),
        ("fastq.deplete_reference_contaminants", "bowtie2"),
        ("fastq.deplete_rrna", "sortmerna"),
        ("fastq.extract_umis", "umi_tools"),
    ];

    for (stage_id, tool_id) in cases {
        let cohorts = crate::stage_api::benchmark_cohorts_for_stage(&StageId::new(stage_id));
        assert_eq!(
            cohorts.len(),
            1,
            "{stage_id} should publish exactly one governed benchmark cohort",
        );
        assert_eq!(
            cohorts[0].tool_ids,
            vec![ToolId::new(tool_id)],
            "{stage_id} should surface the governed benchmark toolset from stage governance",
        );
    }
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

    assert_eq!(expanded_pipeline.nodes.len(), 3);
    assert_eq!(expanded_pipeline.edges.len(), 2);
    assert_eq!(expanded_stage_tools.len(), 3);
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

    assert_eq!(expanded_pipeline.nodes.len(), 4);
    assert_eq!(expanded_stage_tools.len(), 4);
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
    let pipeline = PipelineSpec::graph(
        vec![
            "fastq.validate_reads",
            "fastq.trim_reads",
            "fastq.filter_reads",
            "fastq.remove_duplicates",
            "fastq.profile_reads",
        ]
        .into_iter()
        .map(|stage_id| PipelineNodeSpec {
            stage_id: stage_id.to_string(),
            stage_instance_id: None,
        })
        .collect(),
        vec![
            ("fastq.validate_reads", "fastq.trim_reads"),
            ("fastq.trim_reads", "fastq.filter_reads"),
            ("fastq.filter_reads", "fastq.remove_duplicates"),
            ("fastq.remove_duplicates", "fastq.profile_reads"),
        ]
        .into_iter()
        .map(|(from, to)| PipelineEdgeSpec {
            from: from.to_string(),
            to: to.to_string(),
            from_output_id: None,
            to_input_id: None,
        })
        .collect(),
    );
    let toolsets = pipeline
        .ordered_nodes()
        .into_iter()
        .map(|node| ToolsetSelection {
            stage_id: node.stage_id,
            stage_instance_id: node.stage_instance_id,
            tool_ids: (0..6).map(|idx| format!("tool_{idx}")).collect(),
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        })
        .collect::<Vec<_>>();

    let error = expand_pipeline_stage_tool_routes(&pipeline, &toolsets)
        .expect_err("route expansion should reject unbounded graph fanout");
    assert!(error.to_string().contains("preprocess tool route expansion would create"));
}

#[test]
fn expand_pipeline_stage_tool_routes_collapses_selected_stage_context() {
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
    );
    let toolsets = vec![
        ToolsetSelection {
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            tool_ids: vec!["fastp".to_string(), "cutadapt".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
        ToolsetSelection {
            stage_id: "fastq.filter_reads".to_string(),
            stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
            tool_ids: vec!["seqkit".to_string()],
            reason: PlanDecisionReason::new(PlanReasonKind::Default, "test"),
        },
    ];

    let (expanded_pipeline, expanded_selections) =
        expand_pipeline_stage_tool_routes(&pipeline, &toolsets)
            .expect("planner-owned select nodes should collapse route context");

    let select_nodes = expanded_pipeline
        .nodes
        .iter()
        .filter(|node| node.stage_id == "benchmark.select_stage_tool")
        .collect::<Vec<_>>();
    assert_eq!(select_nodes.len(), 1);
    assert_eq!(
        select_nodes[0].stage_instance_id.as_deref(),
        Some("benchmark.select_stage_tool.trim_reads")
    );

    let downstream_nodes = expanded_pipeline
        .nodes
        .iter()
        .filter(|node| node.stage_id == "fastq.filter_reads")
        .collect::<Vec<_>>();
    assert_eq!(downstream_nodes.len(), 1);
    assert!(downstream_nodes[0]
        .stage_instance_id
        .as_deref()
        .is_some_and(|node_id| node_id.ends_with(".tool.seqkit")));
    assert!(downstream_nodes[0]
        .stage_instance_id
        .as_deref()
        .is_some_and(|node_id| !node_id.contains("fastq.trim_reads.cleanup=")));
    assert_eq!(
        expanded_selections
            .iter()
            .filter(|selection| selection.stage_id == "fastq.filter_reads")
            .count(),
        1
    );
}

#[test]
fn reference_aware_depletion_rejects_incompatible_index_bindings_early() {
    let error = FastqPlanner::plan(&FastqPlanConfig {
        pipeline_id: "fastq-to-fastq__host_depletion__v1".to_string(),
        policy: bijux_dna_core::contract::PlanPolicy::default(),
        selection_objective: bijux_dna_core::contract::Objective::Balanced,
        pipeline_spec: Some(PipelineSpec::graph(
            vec![
                PipelineNodeSpec {
                    stage_id: "fastq.index_reference".to_string(),
                    stage_instance_id: None,
                },
                PipelineNodeSpec {
                    stage_id: "fastq.deplete_host".to_string(),
                    stage_instance_id: None,
                },
            ],
            vec![PipelineEdgeSpec {
                from: "fastq.index_reference".to_string(),
                to: "fastq.deplete_host".to_string(),
                from_output_id: None,
                to_input_id: None,
            }],
        )),
        stage_bindings: vec![
            FastqStageBinding {
                stage_id: "fastq.index_reference".to_string(),
                stage_instance_id: None,
                tool: ToolExecutionSpecV1 {
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
                reason: None,
                params: None,
            },
            FastqStageBinding {
                stage_id: "fastq.deplete_host".to_string(),
                stage_instance_id: None,
                tool: ToolExecutionSpecV1 {
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
                reason: None,
                params: None,
            },
        ],
        stage_toolsets: Vec::new(),
        aux_images: std::collections::BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: Some(std::path::PathBuf::from("reference.fa")),
        out_dir: std::path::PathBuf::from("out"),
        allow_planned: false,
    })
    .expect_err("incompatible reference backends should fail before stage composition");

    assert!(error.to_string().contains("requires one of [bowtie2_build]"));
}

#[test]
fn resolve_preprocess_pipeline_materializes_profile_catalog_as_graph() {
    let args = crate::selection::args::BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: Some("fastq-to-fastq__default__v1".to_string()),
        r1: "reads_R1.fastq.gz".into(),
        r2: None,
        reference_fasta: None,
        out: "out".into(),
        strict: false,
        auto: false,
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

    let pipeline = resolve_preprocess_pipeline(
        &args,
        &PreprocessDecisions {
            enable_merge: false,
            enable_correct: false,
            merge_decision: None,
            correct_decision: None,
        },
    );

    assert!(
        pipeline
            .edges
            .iter()
            .any(|edge| edge.from == "fastq.filter_reads" && edge.to == "fastq.profile_reads"),
        "profile catalogs must resolve through the governed preprocess DAG"
    );
    assert!(
        pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.profile_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("qc_json")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }),
        "profile catalogs must keep the report_qc artifact aggregation join"
    );
    assert!(
        !pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.profile_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.is_none()
                && edge.to_input_id.is_none()
        }),
        "report_qc joins should be explicit artifact bindings in the default DAG"
    );
}
