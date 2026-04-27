use bijux_dna_core::contract::{Objective, PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};
use bijux_dna_planner_fastq::stage_api::args::{BenchFastqPreprocessArgs, FastqPlannerMode};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[allow(clippy::duplicate_mod)]
#[path = "../support/tool_registry.rs"]
mod tool_registry_support;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn tool_registry() -> &'static bijux_dna_core::contract::ToolRegistry {
    static REGISTRY: OnceLock<bijux_dna_core::contract::ToolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        tool_registry_support::load_domain_tool_registry(&workspace_root())
            .unwrap_or_else(|error| panic!("load domain tool registry: {error}"))
    })
}

fn single_stage_graph(stage_id: &str) -> PipelineSpec {
    PipelineSpec::graph(
        vec![PipelineNodeSpec {
            stage_id: stage_id.to_string(),
            stage_instance_id: Some(stage_id.to_string()),
        }],
        Vec::new(),
    )
}

#[test]
fn toolset_selection_uses_execution_modes_for_governed_and_benchmark_paths() -> anyhow::Result<()> {
    let pipeline = single_stage_graph("fastq.trim_reads");

    let default = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::DefaultChoice,
        false,
    )?;
    assert_eq!(default.len(), 1);
    assert_eq!(default[0].stage_id, "fastq.trim_reads");
    assert_eq!(default[0].tool_ids, vec!["fastp".to_string()]);

    let governed = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
        false,
    )?;
    assert!(governed[0].tool_ids.iter().any(|tool_id| tool_id == "fastp"));
    assert!(!governed[0].tool_ids.iter().any(|tool_id| tool_id == "seqpurge"));

    let benchmark = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
        false,
    )?;
    assert!(!benchmark[0].tool_ids.is_empty());
    assert!(benchmark[0].tool_ids.iter().all(|tool_id| governed[0].tool_ids.contains(tool_id)));

    Ok(())
}

#[test]
fn toolset_selection_keeps_declared_bindings_and_governed_infer_asvs_explicit() -> anyhow::Result<()>
{
    let trim_pipeline = single_stage_graph("fastq.trim_reads");
    let all_bindings = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &trim_pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
        false,
    )?;
    assert!(all_bindings[0].tool_ids.iter().any(|tool_id| tool_id == "seqpurge"));

    let infer_pipeline = single_stage_graph("fastq.infer_asvs");
    let governed = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &infer_pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
        false,
    )
    .expect("governed infer_asvs must resolve through the standard toolset selector");
    assert_eq!(governed.len(), 1);
    assert_eq!(governed[0].stage_id, "fastq.infer_asvs");
    assert_eq!(governed[0].tool_ids, vec!["dada2".to_string()]);

    Ok(())
}

#[test]
fn toolset_selection_skips_planner_owned_select_nodes() -> anyhow::Result<()> {
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

    let toolsets = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
        false,
    )?;
    assert_eq!(toolsets.len(), 2);
    assert_eq!(toolsets[0].stage_id, "fastq.trim_reads");
    assert_eq!(toolsets[1].stage_id, "fastq.filter_reads");

    Ok(())
}

#[test]
fn stage_tool_selection_skips_planner_owned_select_nodes() -> anyhow::Result<()> {
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
    let args = BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: None,
        r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: None,
        out: std::path::PathBuf::from("out"),
        strict: false,
        auto: false,
        objective: Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: true,
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
        mode: FastqPlannerMode::Shotgun,
    };

    let selections = bijux_dna_planner_fastq::select_preprocess_stage_tools(
        tool_registry(),
        &pipeline,
        &args,
        None,
    )?;
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0].stage_id, "fastq.trim_reads");
    assert_eq!(selections[1].stage_id, "fastq.filter_reads");

    Ok(())
}

#[test]
fn stage_tool_selection_filters_paired_only_dedup_backends_for_single_end_inputs(
) -> anyhow::Result<()> {
    let pipeline = single_stage_graph("fastq.remove_duplicates");
    let args = BenchFastqPreprocessArgs {
        sample_id: "sample".to_string(),
        profile: None,
        r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
        r2: None,
        reference_fasta: None,
        out: std::path::PathBuf::from("out"),
        strict: false,
        auto: false,
        objective: Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        dry_run: true,
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
        mode: FastqPlannerMode::Shotgun,
    };

    let selections = bijux_dna_planner_fastq::select_preprocess_stage_tools(
        tool_registry(),
        &pipeline,
        &args,
        None,
    )?;
    assert_eq!(selections.len(), 1);
    assert_eq!(selections[0].stage_id, "fastq.remove_duplicates");
    assert_eq!(selections[0].tool_id, "clumpify");
    Ok(())
}

#[test]
fn layout_filter_keeps_fastuniq_only_for_paired_remove_duplicates() {
    let stage_id = bijux_dna_core::ids::StageId::new("fastq.remove_duplicates".to_string());
    let tools = vec![
        bijux_dna_core::ids::ToolId::new("fastuniq".to_string()),
        bijux_dna_core::ids::ToolId::new("clumpify".to_string()),
    ];

    let single_end = bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(
        &stage_id,
        tools.clone(),
        false,
    );
    assert_eq!(
        single_end.into_iter().map(|tool| tool.to_string()).collect::<Vec<_>>(),
        vec!["clumpify".to_string()]
    );

    let paired_end =
        bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(&stage_id, tools, true);
    assert_eq!(
        paired_end.into_iter().map(|tool| tool.to_string()).collect::<Vec<_>>(),
        vec!["fastuniq".to_string(), "clumpify".to_string()]
    );
}

#[test]
fn layout_filter_excludes_paired_only_correction_family_for_single_end_inputs() {
    let stage_id = bijux_dna_core::ids::StageId::new("fastq.correct_errors".to_string());
    let tools = vec![
        bijux_dna_core::ids::ToolId::new("rcorrector".to_string()),
        bijux_dna_core::ids::ToolId::new("musket".to_string()),
    ];

    let single_end =
        bijux_dna_planner_fastq::stage_api::filter_tools_for_input_layout(&stage_id, tools, false);

    assert_eq!(
        single_end,
        vec![
            bijux_dna_core::ids::ToolId::new("rcorrector".to_string()),
            bijux_dna_core::ids::ToolId::new("musket".to_string()),
        ]
    );
}
