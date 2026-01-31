use bijux_analyze::compare::compare_robust_stats;
use bijux_core::FactsRowV1;

fn row(runtime: f64, memory: f64, reads_in: u64, reads_out: u64) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: None,
        trace_id: "t".to_string(),
        span_id: "s".to_string(),
        params_hash: "p".to_string(),
        input_hash: "i".to_string(),
        output_hashes: vec![],
        runtime_s: runtime,
        memory_mb: memory,
        exit_code: 0,
        bank_hashes: serde_json::json!({}),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: None,
        bases_out: None,
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({}),
        artifacts: serde_json::json!({}),
    }
}

#[test]
fn compare_robust_stats_flags_small_sample() -> anyhow::Result<()> {
    let rows = vec![row(1.0, 10.0, 100, 90), row(2.0, 12.0, 100, 80)];
    let stats = compare_robust_stats(&rows)?;
    assert!(stats.flags.contains(&"sample_size_too_small".to_string()));
    Ok(())
}

#[test]
fn compare_robust_stats_detects_outliers() -> anyhow::Result<()> {
    let rows = vec![
        row(1.0, 10.0, 100, 90),
        row(1.1, 11.0, 100, 90),
        row(1.0, 10.0, 100, 90),
        row(10.0, 100.0, 100, 10),
    ];
    let stats = compare_robust_stats(&rows)?;
    assert!(stats.flags.contains(&"outliers_detected".to_string()));
    Ok(())
}
