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
    let root = format!("artifacts/runs/bench/results/{tool}");
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
        script_path: format!("artifacts/runs/bench/slurm/{tool}.sbatch"),
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
        script_path: "artifacts/runs/bench/hpc-dry-run/slurm/stage-benchmark-array.sbatch"
            .to_string(),
        manifest_path: "artifacts/runs/bench/hpc-dry-run/slurm/stage-benchmark-array-manifest.json"
            .to_string(),
        wrapper_log_root: "artifacts/runs/bench/hpc-dry-run/slurm/logs/stage-benchmark-array"
            .to_string(),
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

#[test]
fn local_hpc_stage_benchmark_array_rejects_overflowing_resource_durations() {
    let mut row = report();
    row.rows[0].resources.time_limit = "18446744073709551615:00:00".to_string();
    assert!(validate_benchmark_local_hpc_stage_benchmark_array(&row).is_err());
    let mut envelope = report();
    envelope.resource_envelope.time_limit = "18446744073709551615:00:00".to_string();
    assert!(validate_benchmark_local_hpc_stage_benchmark_array(&envelope).is_err());
}

#[test]
fn stage_array_join_rejects_missing_ambiguous_and_mismatched_observations() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        build_hpc_stage_array_report, BenchmarkLocalAllDomainSlurmSubmitJob,
        BenchmarkLocalHpcExecutionResolverRow, BenchmarkLocalHpcScratchLayoutJob,
        HpcStageArrayInputs,
    };
    use serde_json::json;
    let job: BenchmarkLocalAllDomainSlurmSubmitJob = serde_json::from_value(json!({
        "job_id_local":"benchmark:vcf:run:vcf.qc:profile:bcftools","result_id":"vcf:run:vcf.qc:profile:bcftools",
        "domain":"vcf","stage_id":"vcf.qc","tool_id":"bcftools","corpus_id":"run","asset_profile_id":"profile",
        "pipeline_id":null,"node_id":null,"script_path":"artifacts/runs/bench/job.sbatch","stdout":"out","stderr":"err","outputs":["artifacts/runs/bench/result.json"],"dependencies":[],
        "resources":{"cpus_per_task":1,"memory_mb":1024,"time_limit":"00:10:00"}
    }))?;
    let scratch: BenchmarkLocalHpcScratchLayoutJob = serde_json::from_value(json!({
        "job_id_local":job.job_id_local,"result_id":job.result_id,"job_kind":"benchmark_result",
        "domain":"vcf","stage_id":"vcf.qc","tool_id":"bcftools","corpus_id":"run","asset_profile_id":"profile",
        "pipeline_id":null,"node_id":null,"scratch_root":"scratch","input_links_root":"inputs","output_root":"outputs","log_root":"logs","stdout_path":"out","stderr_path":"err",
        "resources":{"cpus_per_task":1,"memory_mb":1024,"time_limit":"00:10:00","scratch_gb":1},
        "cleanup_policy":{"policy_id":"preserve","remove_scratch_after_successful_sync":false,"retain_scratch_on_failure":true,"preserve_output_root":true,"preserve_log_root":true,"preserve_staged_inputs":true},"input_links":[]
    }))?;
    let execution: BenchmarkLocalHpcExecutionResolverRow = serde_json::from_value(json!({
        "tool_id":"bcftools","lookup_tool_id":"bcftools","execution_mode":"containerized","execution_mode_summary":"governed execution","domains":["vcf"],"selected_stage_ids":["vcf.qc"],
        "selected_job_count":1,"selected_job_ids":[job.job_id_local],"selected_result_ids":[job.result_id],"selected_pipeline_ids":[],"selected_node_ids":[],
        "install_kind":"container","resolution_kind":"apptainer_image","resolution_target":"containers/bcftools.sif","command_entrypoint":null,"runtime_probe_paths":[],"registry_paths":[],"unavailable_reason":null
    }))?;
    let build = |jobs: &[BenchmarkLocalAllDomainSlurmSubmitJob],
                 scratch: &[BenchmarkLocalHpcScratchLayoutJob],
                 executions: &[BenchmarkLocalHpcExecutionResolverRow]| {
        build_hpc_stage_array_report(HpcStageArrayInputs {
            jobs,
            scratch,
            executions,
            script_path: "artifacts/runs/bench/array.sbatch".into(),
            manifest_path: "artifacts/runs/bench/array-manifest.json".into(),
            wrapper_log_root: "artifacts/runs/bench/logs/array".into(),
        })
    };
    let jobs = [job];
    let scratch = [scratch];
    let executions = [execution];
    let accepted = build(&jobs, &scratch, &executions)?;
    assert_eq!(accepted.array_spec, "0-0");
    assert_eq!(accepted.rows[0].benchmark_result_argv.last(), jobs[0].result_id.as_ref());
    assert!(build(&[], &scratch, &executions).is_err());
    assert!(build(&jobs, &[], &executions).is_err());
    assert!(build(&jobs, &scratch, &[]).is_err());
    assert!(build(&jobs, &[scratch[0].clone(), scratch[0].clone()], &executions).is_err());
    assert!(build(&jobs, &scratch, &[executions[0].clone(), executions[0].clone()]).is_err());
    let mut changed = scratch.clone();
    changed[0].tool_id = "other".into();
    assert!(build(&jobs, &changed, &executions).is_err());
    let mut changed = executions.clone();
    changed[0].selected_job_ids.clear();
    assert!(build(&jobs, &scratch, &changed).is_err());
    let mut changed = executions.clone();
    changed[0].selected_result_ids.clear();
    assert!(build(&jobs, &scratch, &changed).is_err());
    assert!(build(&[jobs[0].clone(), jobs[0].clone()], &scratch, &executions).is_err());
    Ok(())
}
