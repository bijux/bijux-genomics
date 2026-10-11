use bijux_dna_bench_model::{
    benchmark_local_hpc_resume_simulation_schema, validate_benchmark_local_hpc_resume_simulation,
    BenchmarkLocalHpcResumeSimulationBehavior, BenchmarkLocalHpcResumeSimulationJob,
    BenchmarkLocalHpcResumeSimulationReport, BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
};

#[allow(clippy::too_many_arguments)]
fn pipeline_job(
    id: &str,
    node: &str,
    dependencies: Vec<String>,
    completion_state: &str,
    runtime_status: Option<&str>,
    resume_action: &str,
    reason: &str,
    rerun_dependencies: Vec<String>,
) -> BenchmarkLocalHpcResumeSimulationJob {
    job(
        id,
        "essential_pipeline_node",
        None,
        Some("relatedness-segments-vcf"),
        Some(node),
        dependencies,
        completion_state,
        runtime_status,
        resume_action,
        reason,
        rerun_dependencies,
    )
}

fn benchmark_job(
    id: &str,
    completion_state: &str,
    runtime_status: Option<&str>,
    resume_action: &str,
    reason: &str,
) -> BenchmarkLocalHpcResumeSimulationJob {
    job(
        id,
        "benchmark_result",
        Some(&format!("result:{id}")),
        None,
        None,
        Vec::new(),
        completion_state,
        runtime_status,
        resume_action,
        reason,
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
fn job(
    id: &str,
    kind: &str,
    result_id: Option<&str>,
    pipeline_id: Option<&str>,
    node_id: Option<&str>,
    dependencies: Vec<String>,
    completion_state: &str,
    runtime_status: Option<&str>,
    resume_action: &str,
    reason: &str,
    rerun_dependencies: Vec<String>,
) -> BenchmarkLocalHpcResumeSimulationJob {
    let manifest_valid = !matches!(
        completion_state,
        "invalid_stage_result_manifest" | "missing_stage_result_manifest"
    );
    let complete = completion_state == "valid_completed";
    BenchmarkLocalHpcResumeSimulationJob {
        job_id_local: id.to_string(),
        job_kind: kind.to_string(),
        result_id: result_id.map(str::to_string),
        pipeline_id: pipeline_id.map(str::to_string),
        node_id: node_id.map(str::to_string),
        stage_id: "vcf.qc".to_string(),
        tool_id: "bcftools".to_string(),
        dependency_count: dependencies.len(),
        dependencies,
        completion_state: completion_state.to_string(),
        completion_detail: completion_state.to_string(),
        manifest_valid,
        runtime_status: runtime_status.map(str::to_string),
        declared_output_count: 1,
        present_output_count: usize::from(complete),
        missing_output_paths: if complete {
            Vec::new()
        } else {
            vec![format!("artifacts/runs/bench/results/{id}/output.txt")]
        },
        resume_action: resume_action.to_string(),
        reason: reason.to_string(),
        rerun_dependencies,
        result_root: format!("artifacts/runs/bench/results/{id}"),
        stage_result_manifest_path: format!("artifacts/runs/bench/results/{id}/stage-result.json"),
    }
}

fn report() -> BenchmarkLocalHpcResumeSimulationReport {
    let jobs = vec![
        pipeline_job(
            BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID,
            "vcf.ibd",
            Vec::new(),
            "failed_stage_result_manifest",
            Some("failed"),
            "rerun",
            "failed_stage_result_manifest",
            Vec::new(),
        ),
        pipeline_job(
            BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID,
            "vcf.demography",
            vec![BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID.to_string()],
            "valid_completed",
            Some("succeeded"),
            "rerun",
            "upstream_dependency_rerun",
            vec![BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID.to_string()],
        ),
        pipeline_job(
            BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
            "vcf.roh",
            Vec::new(),
            "valid_completed",
            Some("succeeded"),
            "skip",
            "valid_completed",
            Vec::new(),
        ),
        benchmark_job(
            BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
            "missing_stage_result_manifest",
            None,
            "rerun",
            "missing_stage_result_manifest",
        ),
        benchmark_job(
            BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID,
            "stale_partial_outputs",
            Some("succeeded"),
            "rerun",
            "stale_partial_outputs",
        ),
        benchmark_job(
            BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
            "valid_completed",
            Some("succeeded"),
            "skip",
            "valid_completed",
        ),
    ];
    BenchmarkLocalHpcResumeSimulationReport {
        schema_version: BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1.to_string(),
        manifest_path: "artifacts/runs/bench/hpc-dry-run/job-graph.json".to_string(),
        output_path: "artifacts/runs/bench/hpc-dry-run/resume-simulation.json".to_string(),
        simulation_root: "artifacts/runs/bench/hpc-dry-run/resume-simulation-tree".to_string(),
        job_count: 6,
        benchmark_job_count: 3,
        essential_pipeline_job_count: 3,
        dependency_count: 1,
        valid_completed_job_count: 3,
        failed_stage_result_job_count: 1,
        invalid_stage_result_job_count: 0,
        missing_stage_result_job_count: 1,
        stale_partial_output_job_count: 1,
        skip_job_count: 2,
        rerun_job_count: 4,
        dependency_rerun_job_count: 1,
        behavior: BenchmarkLocalHpcResumeSimulationBehavior {
            failed_stage_result_job_id: BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID.to_string(),
            dependency_rerun_job_id: BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID.to_string(),
            unrelated_valid_job_id: BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID.to_string(),
            missing_manifest_job_id: BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID.to_string(),
            stale_partial_output_job_id: BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID.to_string(),
            valid_completed_job_id: BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID.to_string(),
            failed_stage_result_reruns: true,
            dependency_rerun_propagates: true,
            unrelated_valid_job_skips: true,
            missing_manifest_reruns: true,
            stale_partial_output_reruns: true,
            valid_completed_job_skips: true,
            proven: true,
        },
        jobs,
    }
}

#[test]
fn local_hpc_resume_simulation_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_resume_simulation(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcResumeSimulationReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_resume_simulation_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-resume-simulation.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC resume-simulation schema drifted");
    Ok(())
}

#[test]
fn local_hpc_resume_simulation_rejects_counter_order_and_evidence_drift() {
    let mut counter = report();
    counter.rerun_job_count -= 1;
    assert!(validate_benchmark_local_hpc_resume_simulation(&counter).is_err());

    let mut order = report();
    order.jobs.swap(0, 1);
    assert!(validate_benchmark_local_hpc_resume_simulation(&order).is_err());

    let mut evidence = report();
    evidence.jobs[5].runtime_status = Some("failed".to_string());
    assert!(validate_benchmark_local_hpc_resume_simulation(&evidence).is_err());
}
