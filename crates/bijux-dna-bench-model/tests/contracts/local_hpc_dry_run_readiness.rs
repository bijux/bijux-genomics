use bijux_dna_bench_model::{
    benchmark_local_hpc_dry_run_readiness_schema, validate_benchmark_local_hpc_dry_run_readiness,
    BenchmarkLocalHpcDryRunReadinessBehavior, BenchmarkLocalHpcDryRunReadinessGoalCheck,
    BenchmarkLocalHpcDryRunReadinessReport, BenchmarkLocalHpcDryRunReadinessSummary,
    BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1,
};

fn behavior() -> BenchmarkLocalHpcDryRunReadinessBehavior {
    BenchmarkLocalHpcDryRunReadinessBehavior {
        goals_481_to_489_validated: true,
        benchmark_jobs_keep_known_assets: true,
        benchmark_jobs_keep_scratch_layout: true,
        benchmark_jobs_keep_explicit_execution_resolution: true,
        pipeline_nodes_keep_dependency_manifests: true,
        dependency_failures_block_descendants_only: true,
        resume_rules_are_explicit: true,
        result_collection_statuses_are_distinct: true,
        first_run_candidate_is_small_and_governed: true,
        no_manual_steps_or_ambiguity: true,
        proven: true,
    }
}

fn summary() -> BenchmarkLocalHpcDryRunReadinessSummary {
    BenchmarkLocalHpcDryRunReadinessSummary {
        benchmark_job_count: 12,
        essential_pipeline_job_count: 8,
        staged_input_count: 6,
        execution_resolver_row_count: 9,
        execution_unclassified_tool_count: 0,
        stage_benchmark_row_count: 12,
        pipeline_node_row_count: 8,
        dependency_case_count: 2,
        resume_rerun_job_count: 3,
        result_collection_row_count: 5,
        candidate_job_count: 6,
        candidate_representative_group_count: 3,
        candidate_selected_input_bytes: 4096,
    }
}

fn report() -> BenchmarkLocalHpcDryRunReadinessReport {
    let checks = (481..=489)
        .map(|goal_id| BenchmarkLocalHpcDryRunReadinessGoalCheck {
            goal_id,
            surface: format!("governed HPC goal {goal_id}"),
            output_path: Some(format!("artifacts/benchmark-smoke/readiness/hpc/{goal_id}.json")),
            ok: true,
            detail: "governed readiness evidence admitted".to_string(),
        })
        .collect::<Vec<_>>();
    BenchmarkLocalHpcDryRunReadinessReport {
        schema_version: BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/hpc/HPC_DRY_RUN_LOCAL_READY.json"
            .to_string(),
        checked_goal_count: 9,
        passed_goal_count: 9,
        failed_goal_count: 0,
        failing_goal_ids: Vec::new(),
        ready_for_first_hpc_run: true,
        summary: summary(),
        behavior: behavior(),
        checks,
    }
}

#[test]
fn local_hpc_dry_run_readiness_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_dry_run_readiness(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcDryRunReadinessReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_dry_run_readiness_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-dry-run-readiness.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC dry-run readiness schema drifted");
    Ok(())
}

#[test]
fn local_hpc_dry_run_readiness_rejects_goal_behavior_and_summary_drift() {
    let mut goal = report();
    goal.checks.swap(0, 1);
    assert!(validate_benchmark_local_hpc_dry_run_readiness(&goal).is_err());

    let mut behavior = report();
    behavior.behavior.resume_rules_are_explicit = false;
    assert!(validate_benchmark_local_hpc_dry_run_readiness(&behavior).is_err());

    let mut summary = report();
    summary.summary.stage_benchmark_row_count = 11;
    assert!(validate_benchmark_local_hpc_dry_run_readiness(&summary).is_err());

    let mut counters = report();
    counters.failed_goal_count = 1;
    assert!(validate_benchmark_local_hpc_dry_run_readiness(&counters).is_err());
}

#[test]
fn local_hpc_dry_run_readiness_admits_coherent_failed_evidence() -> anyhow::Result<()> {
    let mut report = report();
    report.checks[4].ok = false;
    report.passed_goal_count = 8;
    report.failed_goal_count = 1;
    report.failing_goal_ids = vec![485];
    report.ready_for_first_hpc_run = false;
    report.behavior.goals_481_to_489_validated = false;
    report.behavior.no_manual_steps_or_ambiguity = false;
    report.behavior.proven = false;

    validate_benchmark_local_hpc_dry_run_readiness(&report)?;
    Ok(())
}
