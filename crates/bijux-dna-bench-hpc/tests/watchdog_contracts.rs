use anyhow::Result;
use bijux_dna_bench_hpc::watchdog::{simulate_dag_watchdog_path, LocalDagWatchdogScenario};
mod support;
use std::path::Path;
use support::dag;
#[test]
fn admitted_scenarios_preserve_scheduling_and_report_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let dag = dag();
    for scenario in [
        LocalDagWatchdogScenario::NoGlobalWait,
        LocalDagWatchdogScenario::FailureIsolation,
        LocalDagWatchdogScenario::PartialResume,
        LocalDagWatchdogScenario::CompletionRules,
    ] {
        let output = root.path().join(scenario.as_str()).join("report.json");
        let report = simulate_dag_watchdog_path(
            root.path(),
            Path::new("pipeline.toml"),
            Path::new("dag.json"),
            &dag,
            scenario,
            &output,
        )?;
        assert_eq!(report.schema_version, "bijux.bench.local_dag_watchdog_simulation.v1");
        assert_eq!(report.scenario, scenario.as_str());
        match scenario {
            LocalDagWatchdogScenario::NoGlobalWait => {
                assert!(report.no_global_wait_proven);
                assert_eq!(report.slow_branch_finish_second, 13);
            }
            LocalDagWatchdogScenario::FailureIsolation => {
                assert!(report.failure_isolation_proven);
                assert_eq!(report.sample_count, 2);
            }
            LocalDagWatchdogScenario::PartialResume => {
                assert!(report.partial_resume_proven);
                assert_eq!(report.reused_valid_node_ids.len(), 3);
            }
            LocalDagWatchdogScenario::CompletionRules => {
                assert!(report.completion_rules_proven);
                assert_eq!(report.completion_checks.len(), 5);
            }
        }
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(output)?)?,
            serde_json::to_value(report)?
        );
    }
    Ok(())
}
#[test]
fn malformed_or_out_of_order_evidence_preserves_existing_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("retained.json");
    std::fs::write(&output, b"retained")?;
    let valid = dag();
    let mut schema = valid.clone();
    schema.schema_version = "foreign".into();
    let mut counts = valid.clone();
    counts.node_count += 1;
    let mut topology = valid.clone();
    let dependency = topology.nodes[1].node_id.clone();
    topology.nodes[0].depends_on.push(dependency);
    topology.nodes[0].dependency_count += 1;
    topology.edge_count += 1;
    let mut order = valid;
    order.nodes.swap(0, 1);
    for report in [schema, counts, topology, order] {
        assert!(simulate_dag_watchdog_path(
            root.path(),
            Path::new("pipeline.toml"),
            Path::new("dag.json"),
            &report,
            LocalDagWatchdogScenario::NoGlobalWait,
            &output
        )
        .is_err());
        assert_eq!(std::fs::read(&output)?, b"retained");
    }
    Ok(())
}
#[test]
fn missing_scenario_stage_rejects_before_creating_output_parent() -> Result<()> {
    let root = tempfile::tempdir()?;
    let output = root.path().join("absent/report.json");
    let mut report = dag();
    report.nodes[1].stage_id = bijux_dna_core::id_catalog::FASTQ_FILTER.to_owned() + "_alternate";
    assert!(simulate_dag_watchdog_path(
        root.path(),
        Path::new("pipeline.toml"),
        Path::new("dag.json"),
        &report,
        LocalDagWatchdogScenario::NoGlobalWait,
        &output
    )
    .is_err());
    assert!(!output.parent().ok_or_else(|| anyhow::anyhow!("output parent"))?.exists());
    Ok(())
}
