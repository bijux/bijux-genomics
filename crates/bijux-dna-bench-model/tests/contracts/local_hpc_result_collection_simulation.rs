use bijux_dna_bench_model::{
    benchmark_local_hpc_result_collection_simulation_schema,
    validate_benchmark_local_hpc_result_collection_simulation_report,
    BenchmarkLocalHpcResultCollectionSimulationBehavior,
    BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResultCollectionSimulationRow,
    BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
};

fn row(status: &str, job_id: &str) -> BenchmarkLocalHpcResultCollectionSimulationRow {
    let unavailable = status == "unavailable";
    BenchmarkLocalHpcResultCollectionSimulationRow {
        record_id: if unavailable {
            format!("unavailable:{job_id}")
        } else {
            format!("result:{job_id}")
        },
        source_surface: if unavailable {
            "execution_resolver".to_string()
        } else {
            "stage_result_manifest".to_string()
        },
        collection_status: status.to_string(),
        job_id_local: job_id.to_string(),
        domain: "vcf".to_string(),
        stage_id: "vcf.qc".to_string(),
        tool_id: "bcftools".to_string(),
        result_id: (!unavailable).then(|| format!("result-{status}")),
        pipeline_id: None,
        node_id: None,
        evidence_path: format!("runs/bench/hpc-dry-run/{status}"),
        manifest_path: matches!(status, "complete" | "failed" | "insufficient")
            .then(|| format!("runs/bench/hpc-dry-run/{status}/stage-result.json")),
        declared_output_count: usize::from(!unavailable),
        present_output_count: usize::from(matches!(status, "complete" | "insufficient")),
        detail: format!("governed {status} evidence"),
        insufficient_data_reason: (status == "insufficient")
            .then(|| "not_enough_segments".to_string()),
        unavailable_reason: unavailable.then(|| "external artifact unavailable".to_string()),
    }
}

fn report() -> BenchmarkLocalHpcResultCollectionSimulationReport {
    let identities = [
        ("complete", "job-complete"),
        ("failed", "job-failed"),
        ("missing", "job-missing"),
        ("insufficient", "job-insufficient"),
        ("unavailable", "job-unavailable"),
    ];
    BenchmarkLocalHpcResultCollectionSimulationReport {
        schema_version: BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1.to_string(),
        output_path: "runs/bench/hpc-dry-run/result-collection-simulation.json".to_string(),
        simulation_root: "runs/bench/hpc-dry-run/result-collection-simulation-tree".to_string(),
        execution_resolver_path: "runs/bench/hpc-dry-run/execution-resolver.json".to_string(),
        row_count: 5,
        complete_row_count: 1,
        failed_row_count: 1,
        missing_row_count: 1,
        insufficient_row_count: 1,
        unavailable_row_count: 1,
        behavior: BenchmarkLocalHpcResultCollectionSimulationBehavior {
            complete_result_job_id: identities[0].1.to_string(),
            failed_result_job_id: identities[1].1.to_string(),
            missing_result_job_id: identities[2].1.to_string(),
            insufficient_result_job_id: identities[3].1.to_string(),
            unavailable_result_job_id: identities[4].1.to_string(),
            complete_row_present: true,
            failed_row_present: true,
            missing_row_present: true,
            insufficient_row_present: true,
            unavailable_row_present: true,
            failed_distinct_from_missing: true,
            insufficient_distinct_from_unavailable: true,
            proven: true,
        },
        rows: identities.into_iter().map(|(status, job_id)| row(status, job_id)).collect(),
    }
}

#[test]
fn local_hpc_result_collection_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_result_collection_simulation_report(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcResultCollectionSimulationReport>(
            &serde_json::to_vec(&report)?
        )?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated =
        serde_json::to_value(benchmark_local_hpc_result_collection_simulation_schema())?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-hpc-result-collection-simulation.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC result-collection schema drifted");
    Ok(())
}

#[test]
fn local_hpc_result_collection_rejects_status_and_lineage_drift() {
    let mut status_drift = report();
    status_drift.rows[0].collection_status = "failed".to_string();
    assert!(
        validate_benchmark_local_hpc_result_collection_simulation_report(&status_drift).is_err()
    );

    let mut reason_drift = report();
    reason_drift.rows[3].insufficient_data_reason = None;
    assert!(
        validate_benchmark_local_hpc_result_collection_simulation_report(&reason_drift).is_err()
    );

    let mut behavior_drift = report();
    behavior_drift.behavior.missing_result_job_id = "job-failed".to_string();
    assert!(
        validate_benchmark_local_hpc_result_collection_simulation_report(&behavior_drift).is_err()
    );
}
