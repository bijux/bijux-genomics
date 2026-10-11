use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_bench_model::{
    benchmark_local_hpc_dependency_simulation_schema,
    validate_benchmark_local_hpc_dependency_simulation, BenchmarkLocalHpcDependencySimulationCase,
    BenchmarkLocalHpcDependencySimulationJob, BenchmarkLocalHpcDependencySimulationReport,
    BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
};

const BENCHMARK_JOB_ID: &str = "benchmark:vcf:run:vcf.qc:profile:bcftools";

fn scope() -> Vec<(&'static str, Vec<&'static str>, Vec<&'static str>)> {
    vec![
        (
            BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
            Vec::new(),
            vec![BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID],
        ),
        (
            BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
            vec![BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID],
            Vec::new(),
        ),
        (BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID, Vec::new(), Vec::new()),
        (
            BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID,
            Vec::new(),
            vec![BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID],
        ),
        (
            BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID,
            vec![BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID],
            Vec::new(),
        ),
        (BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID, Vec::new(), Vec::new()),
        (BENCHMARK_JOB_ID, Vec::new(), Vec::new()),
    ]
}

fn simulation_case(
    case_id: &str,
    failed_job_id: &str,
    blocked_job_id: &str,
    continued_job_id: &str,
) -> BenchmarkLocalHpcDependencySimulationCase {
    let descendants = BTreeSet::from([blocked_job_id]);
    let mut statuses = BTreeMap::<&str, &str>::new();
    let mut finishes = BTreeMap::<&str, u64>::new();
    let jobs = scope()
        .into_iter()
        .map(|(id, dependencies, dependents)| {
            let blocking_dependencies = dependencies
                .iter()
                .filter(|dependency| {
                    statuses
                        .get(**dependency)
                        .is_some_and(|status| matches!(*status, "failed" | "blocked"))
                })
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>();
            let status = if id == failed_job_id {
                "failed"
            } else if blocking_dependencies.is_empty() {
                "completed"
            } else {
                "blocked"
            };
            statuses.insert(id, status);
            let blocked = status == "blocked";
            let start_second = if blocked {
                0
            } else {
                dependencies
                    .iter()
                    .filter_map(|dependency| finishes.get(dependency).copied())
                    .max()
                    .unwrap_or(0)
            };
            let finish_second = start_second + u64::from(!blocked);
            finishes.insert(id, finish_second);
            BenchmarkLocalHpcDependencySimulationJob {
                job_id_local: id.to_string(),
                job_kind: if id == BENCHMARK_JOB_ID {
                    "benchmark_result".to_string()
                } else {
                    "essential_pipeline_node".to_string()
                },
                result_id: (id == BENCHMARK_JOB_ID).then(|| "vcf:result".to_string()),
                pipeline_id: (id != BENCHMARK_JOB_ID).then(|| "pipeline".to_string()),
                node_id: (id != BENCHMARK_JOB_ID).then(|| id.to_string()),
                stage_id: "vcf.qc".to_string(),
                dependency_count: dependencies.len(),
                dependencies: dependencies.into_iter().map(str::to_string).collect(),
                dependent_count: dependents.len(),
                dependents: dependents.into_iter().map(str::to_string).collect(),
                blocking_dependencies,
                status: status.to_string(),
                descendant_of_failed_job: descendants.contains(id),
                unrelated_to_failed_job: id != failed_job_id && !descendants.contains(id),
                start_second,
                finish_second,
                duration_seconds: u64::from(!blocked),
            }
        })
        .collect::<Vec<_>>();
    let blocked_job_ids = vec![blocked_job_id.to_string()];
    let continued_unrelated_job_ids = jobs
        .iter()
        .filter(|job| job.status == "completed" && job.unrelated_to_failed_job)
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    BenchmarkLocalHpcDependencySimulationCase {
        case_id: case_id.to_string(),
        failed_job_id: failed_job_id.to_string(),
        blocked_descendant_job_id: blocked_job_id.to_string(),
        continued_sibling_job_id: continued_job_id.to_string(),
        descendant_job_ids: vec![blocked_job_id.to_string()],
        blocked_job_ids,
        continued_benchmark_job_ids: vec![BENCHMARK_JOB_ID.to_string()],
        continued_unrelated_job_ids,
        simulated_makespan_seconds: 2,
        blocked_only_descendants: true,
        unrelated_branches_continue: true,
        benchmark_jobs_continue: true,
        proven: true,
        jobs,
    }
}

fn report() -> BenchmarkLocalHpcDependencySimulationReport {
    BenchmarkLocalHpcDependencySimulationReport {
        schema_version: BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1.to_string(),
        manifest_path: "artifacts/runs/bench/hpc-dry-run/job-graph.json".to_string(),
        output_path: "artifacts/runs/bench/hpc-dry-run/slurm-dependency-simulation.json"
            .to_string(),
        job_count: 7,
        benchmark_job_count: 1,
        essential_pipeline_job_count: 6,
        dependency_count: 2,
        case_count: 2,
        all_cases_proven: true,
        cases: vec![
            simulation_case(
                "relatedness_ibd_failure_isolates_demography_from_roh",
                BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
                BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
                BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
            ),
            simulation_case(
                "reference_panel_prepare_failure_blocks_phasing_without_blocking_qc",
                BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID,
                BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID,
                BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
            ),
        ],
    }
}

#[test]
fn local_hpc_dependency_simulation_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_dependency_simulation(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcDependencySimulationReport>(
            &serde_json::to_vec(&report)?
        )?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_dependency_simulation_schema())?;
    let path =
        root.join("benchmarks/contracts/schemas/local-hpc-dependency-simulation.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC dependency-simulation schema drifted");
    Ok(())
}

#[test]
fn local_hpc_dependency_simulation_rejects_scope_and_branch_drift() {
    let mut scope = report();
    scope.cases[1].jobs[0].stage_id = "vcf.stats".to_string();
    assert!(validate_benchmark_local_hpc_dependency_simulation(&scope).is_err());

    let mut branch = report();
    branch.cases[0].jobs[1].status = "completed".to_string();
    assert!(validate_benchmark_local_hpc_dependency_simulation(&branch).is_err());
}
