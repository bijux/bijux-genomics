use bijux_dna_bench::{
    load_benchmark_stage_result_manifest, validate_benchmark_stage_result_path,
    BenchmarkStageResultStatus, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
    BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
};

#[test]
fn stage_result_loading_and_validation_report_are_owner_derived() -> anyhow::Result<()> {
    let repository = tempfile::tempdir()?;
    let manifest_path = repository.path().join("artifacts/results/stage-result.json");
    std::fs::create_dir_all(manifest_path.parent().expect("manifest parent"))?;
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": BENCHMARK_STAGE_RESULT_SCHEMA_V2,
            "stage_id": "fastq.profile_read_lengths",
            "tool": {"id": "seqkit"},
            "command": {"rendered": "seqkit stats reads.fastq.gz"},
            "runtime": {
                "mode": "local",
                "status": "succeeded",
                "started_at": "2026-08-12T00:00:00Z",
                "finished_at": "2026-08-12T00:00:01Z",
                "elapsed_seconds": 1.0,
                "exit_code": 0
            },
            "resource_metrics": {
                "source": "measured",
                "memory_mb": 64.0,
                "cpu_threads": 1
            },
            "outputs": [{
                "artifact_id": "report_json",
                "declared_path": "report.json",
                "realized_path": "outputs/report.json",
                "role": "report",
                "optional": false,
                "exists": true
            }]
        }))?,
    )?;

    let loaded = load_benchmark_stage_result_manifest(&manifest_path)?;
    assert_eq!(loaded.runtime.status, BenchmarkStageResultStatus::Succeeded);
    let report = validate_benchmark_stage_result_path(repository.path(), &manifest_path)?;
    assert_eq!(report.schema_version, BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1);
    assert_eq!(report.manifest_path, "artifacts/results/stage-result.json");
    assert_eq!(report.stage_id, "fastq.profile_read_lengths");
    assert_eq!(report.tool_id, "seqkit");
    assert_eq!(report.output_count, 1);
    assert!(report.valid);
    Ok(())
}
