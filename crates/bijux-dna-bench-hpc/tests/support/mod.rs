use bijux_dna_bench_model::{
    BenchmarkLocalPipelineDagValidationNode, BenchmarkLocalPipelineDagValidationReport,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
};
pub fn dag() -> BenchmarkLocalPipelineDagValidationReport {
    let nodes = nodes();
    BenchmarkLocalPipelineDagValidationReport {
        schema_version: BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1.into(),
        config_path: "pipeline.toml".into(),
        output_path: "dag.json".into(),
        pipeline_id: "fastq-core-preprocess".into(),
        domain: "fastq".into(),
        default_corpus_id: "corpus-01-mini".into(),
        summary: "governed core preprocessing dependencies".into(),
        reference_context: None,
        execution_context: None,
        project_source_count: 0,
        project_sources: vec![],
        ancient_dna_policy: None,
        reference_assets: None,
        variant_assets: None,
        node_count: nodes.len(),
        edge_count: nodes.iter().map(|n| n.depends_on.len()).sum(),
        acyclic: true,
        valid: true,
        topological_order: nodes.iter().map(|n| n.node_id.clone()).collect(),
        validation_profiles: vec![],
        nodes,
    }
}
fn node(
    stage: &str,
    dependencies: &[&str],
    external: &[&str],
    upstream: &[&str],
    outputs: &[&str],
) -> BenchmarkLocalPipelineDagValidationNode {
    let owned = |items: &[&str]| items.iter().map(|s| (*s).to_owned()).collect();
    BenchmarkLocalPipelineDagValidationNode {
        node_id: stage.into(),
        stage_id: stage.into(),
        readiness_kind: "smoke".into(),
        dependency_count: dependencies.len(),
        external_input_count: external.len(),
        upstream_input_count: upstream.len(),
        output_count: outputs.len(),
        depends_on: owned(dependencies),
        external_inputs: owned(external),
        upstream_inputs: owned(upstream),
        outputs: owned(outputs),
    }
}
fn nodes() -> Vec<BenchmarkLocalPipelineDagValidationNode> {
    vec![
        node(
            bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS,
            &[],
            &["corpus.raw_fastq_reads"],
            &[],
            &["validated_reads", "validation_report"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS,
            &[bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS],
            &[],
            &["validated_reads"],
            &["read_length_metrics"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS,
            &[bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS],
            &[],
            &["validated_reads"],
            &["adapter_report"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_TRIM,
            &[
                bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS,
                bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS,
            ],
            &[],
            &["validated_reads", "adapter_report"],
            &["trimmed_reads", "trim_metrics"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_FILTER,
            &[bijux_dna_core::id_catalog::FASTQ_TRIM],
            &[],
            &["trimmed_reads"],
            &["filtered_reads", "filter_metrics"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_STATS_NEUTRAL,
            &[bijux_dna_core::id_catalog::FASTQ_FILTER],
            &[],
            &["filtered_reads"],
            &["filtered_profile"],
        ),
        node(
            bijux_dna_core::id_catalog::FASTQ_QC_POST,
            &[
                bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS,
                bijux_dna_core::id_catalog::FASTQ_PROFILE_READ_LENGTHS,
                bijux_dna_core::id_catalog::FASTQ_DETECT_ADAPTERS,
                bijux_dna_core::id_catalog::FASTQ_TRIM,
                bijux_dna_core::id_catalog::FASTQ_FILTER,
                bijux_dna_core::id_catalog::FASTQ_STATS_NEUTRAL,
            ],
            &[],
            &[
                "validation_report",
                "read_length_metrics",
                "adapter_report",
                "trim_metrics",
                "filter_metrics",
                "filtered_profile",
            ],
            &["qc_bundle"],
        ),
    ]
}
