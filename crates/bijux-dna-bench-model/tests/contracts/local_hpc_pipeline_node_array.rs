use bijux_dna_bench_model::{
    benchmark_local_hpc_pipeline_node_array_schema,
    validate_benchmark_local_hpc_pipeline_node_array, BenchmarkLocalHpcArrayResources,
    BenchmarkLocalHpcPipelineNodeArrayDependency, BenchmarkLocalHpcPipelineNodeArrayExpectedOutput,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcPipelineNodeArrayRow,
    BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1,
};

fn resources(factor: u32, minutes: usize) -> BenchmarkLocalHpcArrayResources {
    BenchmarkLocalHpcArrayResources {
        cpus_per_task: factor,
        memory_mb: factor * 1024,
        time_limit: format!("00:{minutes:02}:00"),
        scratch_gb: factor,
    }
}

fn output(output_id: &str, output_path: &str) -> BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
    BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
        output_id: output_id.to_string(),
        output_path: output_path.to_string(),
    }
}

fn row(index: usize, node_id: &str) -> BenchmarkLocalHpcPipelineNodeArrayRow {
    let job_id = format!("pipeline:vcf-essential:{node_id}");
    let root = format!("runs/bench/results/{node_id}");
    let dependency = (index == 1).then(|| BenchmarkLocalHpcPipelineNodeArrayDependency {
        node_id: "map".to_string(),
        job_id_local: "pipeline:vcf-essential:map".to_string(),
        array_index: 0,
        upstream_result_ids: vec!["aligned-bam".to_string()],
        expected_outputs: vec![output("aligned-bam", "runs/bench/results/map/aligned.bam")],
    });
    BenchmarkLocalHpcPipelineNodeArrayRow {
        array_index: index,
        job_id_local: job_id,
        pipeline_id: "vcf-essential".to_string(),
        node_id: node_id.to_string(),
        domain: "vcf".to_string(),
        stage_id: if index == 0 { "bam.mapping" } else { "vcf.calling" }.to_string(),
        tool_id: if index == 0 { "bwa-mem2" } else { "bcftools" }.to_string(),
        corpus_id: "high-coverage".to_string(),
        asset_profile_id: "grch38".to_string(),
        dependency_job_ids: dependency.iter().map(|entry| entry.job_id_local.clone()).collect(),
        upstream_result_ids: dependency
            .iter()
            .flat_map(|entry| entry.upstream_result_ids.clone())
            .collect(),
        dependencies: dependency.into_iter().collect(),
        expected_outputs: if index == 0 {
            vec![output("aligned-bam", "runs/bench/results/map/aligned.bam")]
        } else {
            vec![output("variants-vcf", "runs/bench/results/call/variants.vcf.gz")]
        },
        script_path: format!("runs/bench/slurm/{node_id}.sbatch"),
        pipeline_node_argv: vec!["bijux-dna".to_string(), "bench".to_string()],
        pipeline_node_command: "bijux-dna bench".to_string(),
        scratch_root: format!("{root}/scratch"),
        input_links_root: format!("{root}/inputs"),
        output_root: format!("{root}/outputs"),
        log_root: format!("{root}/logs"),
        stdout_path: format!("{root}/stdout.log"),
        stderr_path: format!("{root}/stderr.log"),
        resources: resources(if index == 0 { 1 } else { 2 }, index + 10),
    }
}

fn report() -> BenchmarkLocalHpcPipelineNodeArrayReport {
    BenchmarkLocalHpcPipelineNodeArrayReport {
        schema_version: BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1.to_string(),
        script_path: "runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch".to_string(),
        manifest_path: "runs/bench/hpc-dry-run/slurm/pipeline-node-array-manifest.json".to_string(),
        wrapper_log_root: "runs/bench/hpc-dry-run/slurm/logs/pipeline-node-array".to_string(),
        array_spec: "0-1".to_string(),
        pipeline_count: 1,
        pipeline_job_count: 2,
        dependency_count: 1,
        resource_envelope: resources(2, 11),
        rows: vec![row(0, "map"), row(1, "call")],
    }
}

#[test]
fn local_hpc_pipeline_node_array_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_pipeline_node_array(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcPipelineNodeArrayReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_pipeline_node_array_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-pipeline-node-array.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC pipeline-node array schema drifted");
    Ok(())
}

#[test]
fn local_hpc_pipeline_node_array_rejects_lineage_count_and_envelope_drift() {
    let mut lineage = report();
    lineage.rows[1].dependencies[0].array_index = 1;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&lineage).is_err());

    let mut count = report();
    count.dependency_count = 0;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&count).is_err());

    let mut envelope = report();
    envelope.resource_envelope.memory_mb = 1024;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&envelope).is_err());

    let mut projection = report();
    projection.rows[1].dependencies[0].upstream_result_ids = vec!["unknown-result".to_string()];
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&projection).is_err());

    let mut upstream_closure = report();
    upstream_closure.rows[1].upstream_result_ids.clear();
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&upstream_closure).is_err());
}
