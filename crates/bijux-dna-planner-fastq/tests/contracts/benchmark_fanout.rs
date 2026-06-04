use std::collections::BTreeMap;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use bijux_dna_planner_fastq::{CorrectErrorsStageParams, FastqStageParameters};

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

#[test]
fn benchmark_fanout_plans_parallel_tool_steps_for_one_stage() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-benchmark-fanout")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r2\nT\n+\n#\n")?;

    let graph = bijux_dna_planner_fastq::FastqPlanner::plan_stage_benchmark_cohort(
        &bijux_dna_planner_fastq::FastqStageBenchmarkConfig {
            pipeline_id: "fastq-to-fastq__trim_reads_benchmark__v1".to_string(),
            policy: PlanPolicy::PreferAccuracy,
            stage_id: "fastq.trim_reads".to_string(),
            tools: vec![tool("fastp"), tool("cutadapt")],
            params: None,
            aux_images: BTreeMap::new(),
            adapter_bank: None,
            polyx_bank: None,
            contaminant_bank: None,
            enable_contaminant_removal: false,
            r1: r1.clone(),
            r2: Some(r2.clone()),
            reference_fasta: None,
            out_dir: temp.path().join("out"),
            allow_planned: false,
        },
    )?;

    assert_eq!(graph.steps().len(), 3);
    assert_eq!(graph.edges().len(), 2);
    assert!(graph
        .steps()
        .iter()
        .any(|step| step.step_id.as_str() == "fastq.trim_reads.tool.fastp"));
    assert!(graph
        .steps()
        .iter()
        .any(|step| step.step_id.as_str() == "fastq.trim_reads.tool.cutadapt"));
    let compare_step = graph
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.trim_reads.compare")
        .expect("benchmark fanout graph must include a comparison fan-in step");
    assert_eq!(compare_step.stage_id.as_str(), "benchmark.compare_stage_tools");
    assert!(compare_step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--scenario", "trim_fairness"]));
    let trim_contract_hash = bijux_dna_domain_fastq::stage_contract_hash("fastq.trim_reads")
        .expect("trim contract hash should be available")
        .expect("trim contract hash should compute");
    assert!(compare_step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--stage-contract-hash", trim_contract_hash.as_str()]));
    assert!(compare_step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--comparison-input", "report_json"]));
    assert!(compare_step
        .expected_artifact_ids
        .iter()
        .any(|artifact_id| artifact_id.as_str() == "trim_tool_comparison_json"));
    assert_eq!(compare_step.io.inputs.len(), 6);
    for suffix in ["__report_json", "__trimmed_reads_r1", "__trimmed_reads_r2"] {
        assert_eq!(
            compare_step
                .io
                .inputs
                .iter()
                .filter(|artifact| artifact.name.as_str().ends_with(suffix))
                .count(),
            2,
            "trim fairness fan-in must receive {suffix} from both benchmarked tools",
        );
    }
    assert!(compare_step.io.outputs.iter().any(|artifact| {
        artifact.name.as_str() == "trim_tool_comparison_json"
            && artifact.path.ends_with(std::path::Path::new("compare/trim_tool_comparison.json"))
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.from().as_str() == "fastq.trim_reads.tool.fastp"
            && edge.to().as_str() == "fastq.trim_reads.compare"
    }));
    assert!(graph.edges().iter().any(|edge| {
        edge.from().as_str() == "fastq.trim_reads.tool.cutadapt"
            && edge.to().as_str() == "fastq.trim_reads.compare"
    }));
    Ok(())
}

#[test]
fn benchmark_fanout_rejects_unadmitted_taxonomy_tools() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-benchmark-fanout-planned")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r2\nT\n+\n#\n")?;

    let error = bijux_dna_planner_fastq::FastqPlanner::plan_stage_benchmark_cohort(
        &bijux_dna_planner_fastq::FastqStageBenchmarkConfig {
            pipeline_id: "fastq-to-fastq__screen_taxonomy_benchmark__v1".to_string(),
            policy: PlanPolicy::PreferAccuracy,
            stage_id: "fastq.screen_taxonomy".to_string(),
            tools: vec![tool("diamond")],
            params: None,
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
        },
    )
    .expect_err("unadmitted taxonomy binding must be rejected");

    let message = error.to_string();
    assert!(message.contains("diamond"));
    assert!(message.contains("fastq.screen_taxonomy"));
    Ok(())
}

#[test]
fn benchmark_fanout_scopes_detect_adapter_compare_inputs_to_governed_artifacts(
) -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-detect-adapters-fanout")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let graph = bijux_dna_planner_fastq::FastqPlanner::plan_stage_benchmark_cohort(
        &bijux_dna_planner_fastq::FastqStageBenchmarkConfig {
            pipeline_id: "fastq-to-fastq__detect_adapters_benchmark__v1".to_string(),
            policy: PlanPolicy::PreferAccuracy,
            stage_id: "fastq.detect_adapters".to_string(),
            tools: vec![tool("fastqc")],
            params: None,
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
        },
    )?;

    let compare_step = graph
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.detect_adapters.compare")
        .expect("detect_adapters benchmark graph must include a comparison fan-in step");
    assert!(compare_step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--comparison-input", "adapter_report"]));
    assert!(compare_step
        .command
        .template
        .windows(2)
        .any(|window| window == ["--comparison-input", "adapter_evidence_dir"]));
    assert!(compare_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str().ends_with("__adapter_report")));
    assert!(compare_step
        .io
        .inputs
        .iter()
        .any(|artifact| artifact.name.as_str().ends_with("__adapter_evidence_dir")));
    Ok(())
}

#[test]
fn benchmark_fanout_propagates_typed_correct_error_params() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-correct-benchmark-fanout")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let trusted = temp.path().join("trusted.kmers");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&trusted, b"ACGT\n")?;

    let graph = bijux_dna_planner_fastq::FastqPlanner::plan_stage_benchmark_cohort(
        &bijux_dna_planner_fastq::FastqStageBenchmarkConfig {
            pipeline_id: "fastq-to-fastq__correct_errors_benchmark__v1".to_string(),
            policy: PlanPolicy::PreferAccuracy,
            stage_id: "fastq.correct_errors".to_string(),
            tools: vec![tool("lighter"), tool("musket")],
            params: Some(FastqStageParameters::CorrectErrors(CorrectErrorsStageParams {
                threads: Some(6),
                quality_encoding: QualityEncoding::Phred33,
                kmer_size: Some(31),
                musket_kmer_budget: Some(536_870_912),
                genome_size: Some(2_500_000),
                max_memory_gb: None,
                trusted_kmer_artifact: Some(trusted.clone()),
                conservative_mode: false,
            })),
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
        },
    )?;

    let lighter_step = graph
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.correct_errors.tool.lighter")
        .expect("lighter step");
    assert!(lighter_step.command.template[2].contains("\"kmer_size\":31"));
    assert!(lighter_step.command.template[2].contains("\"trusted_kmer_artifact\":"));
    let musket_step = graph
        .steps()
        .iter()
        .find(|step| step.step_id.as_str() == "fastq.correct_errors.tool.musket")
        .expect("musket step");
    assert!(musket_step.command.template[2].contains("\"kmer_size\":31"));
    assert!(musket_step.command.template[2].contains("\"musket_kmer_budget\":536870912"));
    assert!(!musket_step.command.template[2].contains("\"genome_size\":2500000"));
    assert!(!musket_step.command.template[2].contains("trusted.kmers"));
    Ok(())
}
