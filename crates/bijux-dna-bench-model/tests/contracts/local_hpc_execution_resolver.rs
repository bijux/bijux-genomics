use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_local_hpc_execution_resolver_schema, validate_benchmark_local_hpc_execution_resolver,
    BenchmarkLocalHpcExecutionResolverReport, BenchmarkLocalHpcExecutionResolverRow,
    BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1,
};

fn row(tool: &str, kind: &str) -> BenchmarkLocalHpcExecutionResolverRow {
    let unavailable = kind == "unavailable_with_reason";
    BenchmarkLocalHpcExecutionResolverRow {
        tool_id: tool.to_string(),
        lookup_tool_id: tool.to_string(),
        execution_mode: if unavailable { "external" } else { "containerized" }.to_string(),
        execution_mode_summary: "governed execution mode".to_string(),
        domains: vec!["bam".to_string()],
        selected_stage_ids: vec!["bam.mapping".to_string()],
        selected_job_count: 1,
        selected_job_ids: vec![format!("benchmark:{tool}")],
        selected_result_ids: vec![format!("result:{tool}")],
        selected_pipeline_ids: Vec::new(),
        selected_node_ids: Vec::new(),
        install_kind: if unavailable { "external" } else { "container" }.to_string(),
        resolution_kind: kind.to_string(),
        resolution_target: if unavailable {
            String::new()
        } else {
            format!("containers/{tool}.sif")
        },
        command_entrypoint: Some(tool.to_string()),
        runtime_probe_paths: vec![format!("domain/bam/tools/{tool}.yaml")],
        registry_paths: Vec::new(),
        unavailable_reason: unavailable.then(|| "external artifact admission required".to_string()),
    }
}

fn report() -> BenchmarkLocalHpcExecutionResolverReport {
    let rows = vec![row("angsd", "unavailable_with_reason"), row("samtools", "apptainer_image")];
    BenchmarkLocalHpcExecutionResolverReport {
        schema_version: BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1.to_string(),
        output_path: "artifacts/runs/bench/hpc-dry-run/execution-resolver.tsv".to_string(),
        row_count: rows.len(),
        resolution_counts: BTreeMap::from([
            ("apptainer_image".to_string(), 1),
            ("unavailable_with_reason".to_string(), 1),
        ]),
        unclassified_tool_count: 0,
        rows,
    }
}

#[test]
fn local_hpc_execution_resolver_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_execution_resolver(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcExecutionResolverReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_execution_resolver_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-execution-resolver.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC execution-resolver schema drifted");
    Ok(())
}

#[test]
fn local_hpc_execution_resolver_rejects_counter_scope_and_availability_drift() {
    let mut counter = report();
    counter.row_count = 1;
    assert!(validate_benchmark_local_hpc_execution_resolver(&counter).is_err());
    let mut scope = report();
    scope.rows[0].selected_stage_ids[0] = "fastq.trim".to_string();
    assert!(validate_benchmark_local_hpc_execution_resolver(&scope).is_err());
    let mut availability = report();
    availability.rows[0].resolution_target = "restricted.sif".to_string();
    assert!(validate_benchmark_local_hpc_execution_resolver(&availability).is_err());
}
