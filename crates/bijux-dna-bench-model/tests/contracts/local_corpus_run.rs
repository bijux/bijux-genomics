//! Public benchmark aliases preserve the canonical run schema and error taxonomy.
#[test]
fn corpus_run_compatibility_preserves_schema_and_error_reason() -> anyhow::Result<()> {
    let schema = bijux_dna_bench_model::benchmark_local_corpus_run_schema();
    assert_eq!(
        serde_json::to_value(schema)?,
        serde_json::to_value(bijux_dna_bench_corpus::run::benchmark_local_corpus_run_schema())?
    );
    let invalid: bijux_dna_bench_model::BenchmarkCorpusRunManifest = serde_json::from_value(
        serde_json::json!({
            "schema_version":"unsupported", "generated_at_utc":"", "corpus_id":"", "stage_id":"", "scenario_id":"", "sample_scope":"", "tool_kind":"", "platform":"", "tools":[], "threads":0, "jobs":0, "sample_jobs":0, "sample_limit":null, "dry_run":false, "config_path":"", "publication_config_path":"", "repo_root":"", "corpus_root":"", "out_root":"", "stage_args":[], "samples_total":0, "samples_failed":0, "runs":[]
        }),
    )?;
    let Err(error) = bijux_dna_bench_model::validate_benchmark_local_corpus_run(&invalid) else {
        anyhow::bail!("unsupported schema admitted")
    };
    assert!(
        matches!(error, bijux_dna_bench_model::BenchError::InvalidPolicy(ref reason) if reason == "unsupported FASTQ corpus benchmark run schema")
    );
    let Err(canonical) = bijux_dna_bench_corpus::run::validate_benchmark_local_corpus_run(&invalid)
    else {
        anyhow::bail!("unsupported schema admitted")
    };
    assert_eq!(error.to_string(), canonical.to_string());
    Ok(())
}
