use bijux_dna_bench_model::{
    benchmark_local_hpc_stage_benchmark_array_schema,
    validate_benchmark_local_hpc_stage_benchmark_array, BenchmarkLocalHpcArrayResources,
    BenchmarkLocalHpcStageBenchmarkArrayReport, BenchmarkLocalHpcStageBenchmarkArrayRow,
    BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1,
};

fn row(
    index: usize,
    resource_factor: u32,
    tool: &str,
    unavailable: bool,
) -> BenchmarkLocalHpcStageBenchmarkArrayRow {
    let root = format!("runs/bench/results/{tool}");
    BenchmarkLocalHpcStageBenchmarkArrayRow {
        array_index: index,
        job_id_local: format!("benchmark:vcf:run:vcf.qc:profile:{tool}"),
        result_id: format!("vcf:run:vcf.qc:profile:{tool}"),
        domain: "vcf".to_string(),
        stage_id: "vcf.qc".to_string(),
        tool_id: tool.to_string(),
        corpus_id: "run".to_string(),
        asset_profile_id: "profile".to_string(),
        execution_mode: if unavailable { "external" } else { "containerized" }.to_string(),
        execution_mode_summary: "governed execution".to_string(),
        resolution_kind: if unavailable { "unavailable_with_reason" } else { "apptainer_image" }
            .to_string(),
        resolution_target: if unavailable {
            String::new()
        } else {
            format!("containers/{tool}.sif")
        },
        unavailable_reason: unavailable.then(|| "external artifact admission required".to_string()),
        script_path: format!("runs/bench/slurm/{tool}.sbatch"),
        benchmark_result_argv: vec!["bijux-dna".to_string(), "bench".to_string()],
        benchmark_result_command: "bijux-dna bench".to_string(),
        scratch_root: format!("{root}/scratch"),
        input_links_root: format!("{root}/inputs"),
        output_root: format!("{root}/outputs"),
        log_root: format!("{root}/logs"),
        stdout_path: format!("{root}/stdout.log"),
        stderr_path: format!("{root}/stderr.log"),
        output_paths: vec![format!("{root}/stage-result.json")],
        resources: BenchmarkLocalHpcArrayResources {
            cpus_per_task: resource_factor,
            memory_mb: 1024 * resource_factor,
            time_limit: format!("00:{:02}:00", 10 + index),
            scratch_gb: resource_factor,
        },
    }
}

fn report() -> BenchmarkLocalHpcStageBenchmarkArrayReport {
    BenchmarkLocalHpcStageBenchmarkArrayReport {
        schema_version: BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1.to_string(),
        script_path: "runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch".to_string(),
        manifest_path: "runs/bench/hpc-dry-run/slurm/stage-benchmark-array-manifest.json"
            .to_string(),
        wrapper_log_root: "runs/bench/hpc-dry-run/slurm/logs/stage-benchmark-array".to_string(),
        array_spec: "0-1".to_string(),
        benchmark_job_count: 2,
        resource_envelope: BenchmarkLocalHpcArrayResources {
            cpus_per_task: 2,
            memory_mb: 2048,
            time_limit: "00:11:00".to_string(),
            scratch_gb: 2,
        },
        rows: vec![row(0, 1, "bcftools", false), row(1, 2, "impute5", true)],
    }
}

#[test]
fn local_hpc_stage_benchmark_array_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_stage_benchmark_array(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcStageBenchmarkArrayReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_stage_benchmark_array_schema())?;
    let path =
        root.join("benchmarks/contracts/schemas/local-hpc-stage-benchmark-array.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC stage-benchmark array schema drifted");
    Ok(())
}

#[test]
fn local_hpc_stage_benchmark_array_rejects_order_resolution_and_envelope_drift() {
    let mut order = report();
    order.rows[1].array_index = 0;
    assert!(validate_benchmark_local_hpc_stage_benchmark_array(&order).is_err());

    let mut resolution = report();
    resolution.rows[1].resolution_target = "restricted.sif".to_string();
    assert!(validate_benchmark_local_hpc_stage_benchmark_array(&resolution).is_err());

    let mut envelope = report();
    envelope.resource_envelope.memory_mb = 1024;
    assert!(validate_benchmark_local_hpc_stage_benchmark_array(&envelope).is_err());
}
