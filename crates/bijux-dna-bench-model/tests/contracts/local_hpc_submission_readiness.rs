use bijux_dna_bench_model::{
    benchmark_local_hpc_submission_readiness_schema,
    validate_benchmark_local_hpc_submission_readiness,
    BenchmarkLocalHpcSubmissionReadinessGoalCheck, BenchmarkLocalHpcSubmissionReadinessReport,
    BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1,
};

fn category(goal_id: u32) -> &'static str {
    match goal_id {
        1 | 28 => "stage_matrices",
        2..=27 | 29..=52 => "local_smokes",
        53..=64 => "benchmark_harness",
        65..=76 => "corpora",
        77..=90 => "pipeline_dags",
        91..=99 => "slurm_dry_run",
        _ => unreachable!("test goal outside governed inventory"),
    }
}

fn report() -> BenchmarkLocalHpcSubmissionReadinessReport {
    let checks = (1..=99)
        .map(|goal_id| BenchmarkLocalHpcSubmissionReadinessGoalCheck {
            goal_id,
            category: category(goal_id).to_string(),
            surface: format!("governed submission-readiness goal {goal_id}"),
            output_path: Some(format!(
                "artifacts/benchmark-smoke/readiness/local-ready/{goal_id}.json"
            )),
            ok: true,
            detail: "governed submission evidence admitted".to_string(),
        })
        .collect::<Vec<_>>();
    BenchmarkLocalHpcSubmissionReadinessReport {
        schema_version: BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/local-ready/HPC_SUBMISSION_READY.json"
            .to_string(),
        checked_goal_count: 99,
        passed_goal_count: 99,
        failed_goal_count: 0,
        failing_goal_ids: Vec::new(),
        ok: true,
        checks,
    }
}

#[test]
fn local_hpc_submission_readiness_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_submission_readiness(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcSubmissionReadinessReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_submission_readiness_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-submission-readiness.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC submission-readiness schema drifted");
    Ok(())
}

#[test]
fn local_hpc_submission_readiness_admits_coherent_blockers() -> anyhow::Result<()> {
    let mut report = report();
    report.checks[64].ok = false;
    report.checks[65].ok = false;
    report.passed_goal_count = 97;
    report.failed_goal_count = 2;
    report.failing_goal_ids = vec![65, 66];
    report.ok = false;

    validate_benchmark_local_hpc_submission_readiness(&report)?;
    Ok(())
}

#[test]
fn local_hpc_submission_readiness_rejects_inventory_category_path_and_counter_drift() {
    let mut duplicate = report();
    duplicate.checks[1].goal_id = 1;
    assert!(validate_benchmark_local_hpc_submission_readiness(&duplicate).is_err());

    let mut category = report();
    category.checks[64].category = "fixtures".to_string();
    assert!(validate_benchmark_local_hpc_submission_readiness(&category).is_err());

    let mut path = report();
    path.checks[0].output_path = Some("/tmp/unowned.json".to_string());
    assert!(validate_benchmark_local_hpc_submission_readiness(&path).is_err());

    let mut counters = report();
    counters.passed_goal_count = 98;
    assert!(validate_benchmark_local_hpc_submission_readiness(&counters).is_err());
}
