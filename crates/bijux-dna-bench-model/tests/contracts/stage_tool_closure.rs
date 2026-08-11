use bijux_dna_bench_model::{
    validate_benchmark_stage_tool_closure, BenchmarkStageToolClosureDisposition,
    BenchmarkStageToolClosureReport, BenchmarkStageToolClosureRow, BenchmarkStageToolClosureSource,
    BenchmarkStageToolClosureSummary, BenchmarkStageToolProtocolDisposition,
    BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1,
};

#[test]
fn stage_tool_closure_validates_pair_and_summary_invariants() {
    assert!(validate_benchmark_stage_tool_closure(&report()).is_ok());

    let mut drifted = report();
    drifted.rows[0].domain_declared = false;
    assert!(validate_benchmark_stage_tool_closure(&drifted)
        .is_err_and(|error| error.to_string().contains("disposition is inconsistent")));

    let mut summary_drift = report();
    summary_drift.summary.catalog_pair_count = 0;
    assert!(validate_benchmark_stage_tool_closure(&summary_drift)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut noncanonical_source = report();
    noncanonical_source.sources[0].sha256 = "A".repeat(64);
    assert!(validate_benchmark_stage_tool_closure(&noncanonical_source)
        .is_err_and(|error| error.to_string().contains("sources are invalid")));
}

#[test]
fn stage_tool_closure_schema_matches_model() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root.join("benchmarks/contracts/schemas/stage-tool-closure.schema.json");
    let generated =
        serde_json::to_value(bijux_dna_bench_model::benchmark_stage_tool_closure_schema())?;
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "stage/tool closure schema drifted");
    Ok(())
}

fn report() -> BenchmarkStageToolClosureReport {
    BenchmarkStageToolClosureReport {
        schema_version: BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1.to_string(),
        sources: vec![BenchmarkStageToolClosureSource {
            path: "containers/contracts/catalog.toml".to_string(),
            sha256: "a".repeat(64),
        }],
        summary: BenchmarkStageToolClosureSummary {
            supported_tool_count: 1,
            stage_count: 1,
            catalog_pair_count: 1,
            domain_pair_count: 1,
            aligned_pair_count: 1,
            drift_pair_count: 0,
            benchmark_pair_count: 1,
            qualification_only_pair_count: 0,
            external_artifact_pair_count: 0,
        },
        rows: vec![BenchmarkStageToolClosureRow {
            domain: "fastq".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "cutadapt".to_string(),
            product_support: "supported".to_string(),
            artifact_delivery: "redistributable_image".to_string(),
            redistribution_permitted: true,
            domain_declared: true,
            protocol_disposition: BenchmarkStageToolProtocolDisposition::BenchmarkRequired,
            benchmark_run_id: Some("run-2049".to_string()),
            closure: BenchmarkStageToolClosureDisposition::Aligned,
        }],
    }
}
