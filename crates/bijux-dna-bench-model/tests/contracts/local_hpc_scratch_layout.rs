use bijux_dna_bench_model::{
    benchmark_local_hpc_scratch_layout_schema, validate_benchmark_local_hpc_scratch_layout,
    BenchmarkLocalHpcScratchCleanupPolicy, BenchmarkLocalHpcScratchInputLink,
    BenchmarkLocalHpcScratchJobResources, BenchmarkLocalHpcScratchLayout,
    BenchmarkLocalHpcScratchLayoutJob, BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
};

fn input_link(job_root: &str, source: &str) -> BenchmarkLocalHpcScratchInputLink {
    BenchmarkLocalHpcScratchInputLink {
        artifact_id: Some("reference_fasta".to_string()),
        artifact_role: Some("reference".to_string()),
        source_kind: "directory".to_string(),
        source_path: source.to_string(),
        staged_path: format!("runs/bench/hpc-dry-run/staged/{source}"),
        link_path: format!("{job_root}/inputs/{source}"),
        checksum_sha256: "a".repeat(64),
        size_bytes: 1024,
        member_count: 1,
    }
}

fn job(job_id: &str, job_kind: &str, lineage: (&str, &str)) -> BenchmarkLocalHpcScratchLayoutJob {
    let job_root = format!("runs/bench/hpc-dry-run/scratch/{job_id}");
    let benchmark_result = job_kind == "benchmark_result";
    BenchmarkLocalHpcScratchLayoutJob {
        job_id_local: job_id.to_string(),
        job_kind: job_kind.to_string(),
        result_id: benchmark_result.then(|| lineage.0.to_string()),
        pipeline_id: (!benchmark_result).then(|| lineage.0.to_string()),
        node_id: (!benchmark_result).then(|| lineage.1.to_string()),
        domain: "bam".to_string(),
        stage_id: "bam.align".to_string(),
        tool_id: "bwa-mem2".to_string(),
        corpus_id: "corpus-01-mini".to_string(),
        asset_profile_id: "sample-set".to_string(),
        scratch_root: job_root.clone(),
        input_links_root: format!("{job_root}/inputs"),
        output_root: format!("runs/bench/hpc-dry-run/results/{job_id}"),
        log_root: format!("runs/bench/hpc-dry-run/logs/{job_id}"),
        stdout_path: format!("runs/bench/hpc-dry-run/logs/{job_id}/stdout.log"),
        stderr_path: format!("runs/bench/hpc-dry-run/logs/{job_id}/stderr.log"),
        resources: BenchmarkLocalHpcScratchJobResources {
            cpus_per_task: 4,
            memory_mb: 8192,
            time_limit: "00:30:00".to_string(),
            scratch_gb: 10,
        },
        cleanup_policy: BenchmarkLocalHpcScratchCleanupPolicy {
            policy_id: "copy_back_outputs_remove_successful_scratch".to_string(),
            remove_scratch_after_successful_sync: true,
            retain_scratch_on_failure: true,
            preserve_output_root: true,
            preserve_log_root: true,
            preserve_staged_inputs: true,
        },
        input_links: vec![input_link(
            &job_root,
            "assets/reference/host/references/toy_host_reference",
        )],
    }
}

fn layout() -> BenchmarkLocalHpcScratchLayout {
    BenchmarkLocalHpcScratchLayout {
        schema_version: BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1.to_string(),
        output_path: "runs/bench/hpc-dry-run/scratch-layout.json".to_string(),
        staging_root: "runs/bench/hpc-dry-run/staged".to_string(),
        scratch_root: "runs/bench/hpc-dry-run/scratch".to_string(),
        selected_job_count: 2,
        benchmark_job_count: 1,
        essential_pipeline_job_count: 1,
        input_link_count: 2,
        jobs: vec![
            job("job-benchmark", "benchmark_result", ("result-01", "")),
            job("job-pipeline", "essential_pipeline_node", ("pipeline-01", "align")),
        ],
    }
}

#[test]
fn local_hpc_scratch_layout_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let layout = layout();
    validate_benchmark_local_hpc_scratch_layout(&layout)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcScratchLayout>(&serde_json::to_vec(&layout)?)?,
        layout
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_scratch_layout_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-hpc-scratch-layout.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC scratch-layout schema drifted");
    Ok(())
}

#[test]
fn local_hpc_scratch_layout_rejects_counter_and_lineage_drift() {
    let mut counter_drift = layout();
    counter_drift.input_link_count = 1;
    assert!(validate_benchmark_local_hpc_scratch_layout(&counter_drift).is_err());

    let mut result_lineage_drift = layout();
    result_lineage_drift.jobs[0].pipeline_id = Some("pipeline-01".to_string());
    assert!(validate_benchmark_local_hpc_scratch_layout(&result_lineage_drift).is_err());

    let mut pipeline_lineage_drift = layout();
    pipeline_lineage_drift.jobs[1].node_id = None;
    assert!(validate_benchmark_local_hpc_scratch_layout(&pipeline_lineage_drift).is_err());
}

#[test]
fn local_hpc_scratch_layout_rejects_unsafe_scratch_and_cleanup_contracts() {
    let mut nested_shared_roots = layout();
    nested_shared_roots.staging_root = "runs/bench/hpc-dry-run/scratch/staged".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&nested_shared_roots).is_err());

    let mut escaped_root = layout();
    escaped_root.jobs[0].scratch_root = "runs/bench/hpc-dry-run/scratch-escaped".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&escaped_root).is_err());

    let mut unbounded_resources = layout();
    unbounded_resources.jobs[0].resources.memory_mb = 0;
    assert!(validate_benchmark_local_hpc_scratch_layout(&unbounded_resources).is_err());

    let mut unsafe_cleanup = layout();
    unsafe_cleanup.jobs[0].cleanup_policy.retain_scratch_on_failure = false;
    assert!(validate_benchmark_local_hpc_scratch_layout(&unsafe_cleanup).is_err());

    let mut scratch_owned_output = layout();
    scratch_owned_output.jobs[0].output_root =
        "runs/bench/hpc-dry-run/scratch/job-benchmark/output".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&scratch_owned_output).is_err());
}

#[test]
fn local_hpc_scratch_layout_rejects_invalid_input_admission() {
    let mut escaped_link = layout();
    escaped_link.jobs[0].input_links[0].link_path =
        "runs/bench/hpc-dry-run/scratch/job-benchmark/inputs-escaped/reference".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&escaped_link).is_err());

    let mut invalid_checksum = layout();
    invalid_checksum.jobs[0].input_links[0].checksum_sha256 = "not-a-checksum".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&invalid_checksum).is_err());

    let mut empty_asset = layout();
    empty_asset.jobs[0].input_links[0].member_count = 0;
    assert!(validate_benchmark_local_hpc_scratch_layout(&empty_asset).is_err());

    let mut unknown_source_kind = layout();
    unknown_source_kind.jobs[0].input_links[0].source_kind = "remote".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&unknown_source_kind).is_err());

    let mut incomplete_artifact_identity = layout();
    incomplete_artifact_identity.jobs[0].input_links[0].artifact_role = None;
    assert!(validate_benchmark_local_hpc_scratch_layout(&incomplete_artifact_identity).is_err());

    let mut traversing_staged_path = layout();
    traversing_staged_path.jobs[0].input_links[0].staged_path =
        "runs/bench/hpc-dry-run/staged/../escaped".to_string();
    assert!(validate_benchmark_local_hpc_scratch_layout(&traversing_staged_path).is_err());
}
