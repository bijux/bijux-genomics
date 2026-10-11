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

fn selected_job_graph_with_order(
    reverse: bool,
) -> anyhow::Result<bijux_dna_bench_model::BenchmarkLocalHpcJobGraph> {
    use bijux_dna_bench_model::{
        build_benchmark_local_hpc_job_graph, BenchmarkLocalAllDomainSlurmSubmitJob,
        BenchmarkLocalSlurmSubmitResources,
    };
    let mut jobs: Vec<_> = scope()
        .into_iter()
        .map(|(id, dependencies, _)| {
            let benchmark = id == BENCHMARK_JOB_ID;
            BenchmarkLocalAllDomainSlurmSubmitJob {
                job_id_local: id.to_string(),
                domain: "vcf".to_string(),
                stage_id: "vcf.qc".to_string(),
                pipeline_id: (!benchmark).then(|| "pipeline".to_string()),
                node_id: (!benchmark).then(|| id.to_string()),
                tool_id: "bcftools".to_string(),
                corpus_id: "corpus".to_string(),
                asset_profile_id: "profile".to_string(),
                result_id: benchmark.then(|| "vcf:result".to_string()),
                script_path: "selected.sh".to_string(),
                stdout: "selected.out".to_string(),
                stderr: "selected.err".to_string(),
                outputs: vec!["selected.json".to_string()],
                dependencies: dependencies.into_iter().map(str::to_string).collect(),
                resources: BenchmarkLocalSlurmSubmitResources {
                    cpus_per_task: 1,
                    memory_mb: 128,
                    time_limit: "00:01:00".to_string(),
                },
            }
        })
        .collect();
    if reverse {
        jobs.reverse();
    }
    build_benchmark_local_hpc_job_graph("artifacts/selected-jobs.json", jobs)
}

#[test]
fn local_hpc_dependency_simulation_builder_preserves_branch_isolation() -> anyhow::Result<()> {
    use bijux_dna_bench_model::build_benchmark_local_hpc_dependency_simulation;
    let graph = selected_job_graph_with_order(false)?;
    let simulation =
        build_benchmark_local_hpc_dependency_simulation(&graph, "artifacts/report.json")?;
    assert_eq!(simulation.job_count, 7);
    assert_eq!(simulation.case_count, 2);
    assert_eq!(simulation.dependency_count, 2);
    assert_eq!(simulation.manifest_path, graph.manifest_path);
    assert!(simulation.all_cases_proven);
    for case in &simulation.cases {
        assert_eq!(
            case.blocked_job_ids.as_slice(),
            std::slice::from_ref(&case.blocked_descendant_job_id)
        );
        assert_eq!(case.jobs.iter().filter(|job| job.status == "failed").count(), 1);
        assert_eq!(case.jobs.iter().filter(|job| job.status == "completed").count(), 5);
        assert_eq!(case.simulated_makespan_seconds, 2);
        assert!(case.continued_benchmark_job_ids.iter().any(|id| id == BENCHMARK_JOB_ID));
        assert!(case.continued_unrelated_job_ids.contains(&case.continued_sibling_job_id));
    }
    validate_benchmark_local_hpc_dependency_simulation(&simulation)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcDependencySimulationReport>(
            &serde_json::to_vec(&simulation)?
        )?,
        simulation
    );
    let reordered = selected_job_graph_with_order(true)?;
    assert_eq!(
        build_benchmark_local_hpc_dependency_simulation(&reordered, "artifacts/report.json")?,
        simulation
    );
    Ok(())
}

#[test]
fn local_hpc_dependency_simulation_builder_rejects_invalid_graph_or_output() -> anyhow::Result<()> {
    use bijux_dna_bench_model::build_benchmark_local_hpc_dependency_simulation;
    let graph = selected_job_graph_with_order(false)?;
    assert!(build_benchmark_local_hpc_dependency_simulation(&graph, "../report.json").is_err());
    let mut counters = graph.clone();
    counters.job_count += 1;
    assert!(build_benchmark_local_hpc_dependency_simulation(&counters, "artifacts/report.json")
        .is_err());
    let mut ordering = graph;
    ordering.topological_order.reverse();
    assert!(build_benchmark_local_hpc_dependency_simulation(&ordering, "artifacts/report.json")
        .is_err());
    Ok(())
}

#[test]
fn local_hpc_dependency_simulation_builder_requires_governed_seed_jobs() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        build_benchmark_local_hpc_dependency_simulation, validate_benchmark_local_hpc_job_graph,
    };
    let mut graph = selected_job_graph_with_order(false)?;
    let rename = |name: &mut String| {
        if name == BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID {
            *name = "pipeline:different:vcf.ibd".to_string();
        }
    };
    for node in &mut graph.nodes {
        rename(&mut node.job_id_local);
        for name in node.dependencies.iter_mut().chain(node.dependents.iter_mut()) {
            rename(name);
        }
    }
    for name in graph.topological_order.iter_mut().chain(graph.root_job_ids.iter_mut()) {
        rename(name);
    }
    validate_benchmark_local_hpc_job_graph(&graph)?;
    assert!(
        build_benchmark_local_hpc_dependency_simulation(&graph, "artifacts/report.json").is_err()
    );
    Ok(())
}
