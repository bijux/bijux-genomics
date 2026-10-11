use bijux_dna_bench_model::{
    benchmark_dossier_index_schema, validate_benchmark_dossier_index, BenchmarkDossierIndex,
    BenchmarkDossierStageEntry, BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};

fn index() -> BenchmarkDossierIndex {
    BenchmarkDossierIndex {
        schema_version: BENCHMARK_DOSSIER_INDEX_SCHEMA_V1.to_string(),
        corpus_id: "ancient-dna-fastq-v1".to_string(),
        stage_count: 2,
        published_stage_count: 1,
        missing_stage_count: 1,
        stages: vec![
            BenchmarkDossierStageEntry {
                stage_id: "fastq.validate_reads".to_string(),
                sample_scope: "full".to_string(),
                status: "published".to_string(),
                summary_path: "docs/fastq.validate_reads/summary.json".to_string(),
                dossier_path: "docs/fastq.validate_reads/benchmark.md".to_string(),
                expected_remote_run_root: "/study/results/validate".to_string(),
                expected_local_cache_mirror_run_root: "/cache/results/validate".to_string(),
                expected_local_results_run_root: "/local/results/validate".to_string(),
                generated_at_utc: Some("2026-08-12T14:00:00Z".to_string()),
                platform: Some("lunarc-apptainer".to_string()),
                corpus_root: Some("/study/corpora/fastq".to_string()),
                run_root: Some("/cache/results/validate".to_string()),
                run_root_source: Some("local-cache-mirror".to_string()),
            },
            BenchmarkDossierStageEntry {
                stage_id: "fastq.trim_reads".to_string(),
                sample_scope: "full".to_string(),
                status: "missing".to_string(),
                summary_path: "docs/fastq.trim_reads/summary.json".to_string(),
                dossier_path: "docs/fastq.trim_reads/benchmark.md".to_string(),
                expected_remote_run_root: "/study/results/trim".to_string(),
                expected_local_cache_mirror_run_root: "/cache/results/trim".to_string(),
                expected_local_results_run_root: "/local/results/trim".to_string(),
                generated_at_utc: None,
                platform: None,
                corpus_root: None,
                run_root: None,
                run_root_source: None,
            },
        ],
    }
}

#[test]
fn dossier_index_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let index = index();
    validate_benchmark_dossier_index(&index)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkDossierIndex>(&serde_json::to_vec(&index)?)?,
        index
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_dossier_index_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/dossier-index.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "dossier index schema drifted from model authority");
    Ok(())
}

#[test]
fn dossier_index_rejects_counter_identity_and_metadata_drift() -> anyhow::Result<()> {
    let mut count_drift = index();
    count_drift.published_stage_count = 2;
    assert!(validate_benchmark_dossier_index(&count_drift).is_err());

    let mut duplicate_stage = index();
    duplicate_stage.stages[1].stage_id = duplicate_stage.stages[0].stage_id.clone();
    assert!(validate_benchmark_dossier_index(&duplicate_stage).is_err());

    let mut missing_metadata = index();
    missing_metadata.stages[0].run_root = None;
    assert!(validate_benchmark_dossier_index(&missing_metadata).is_err());

    let mut contradictory_metadata = index();
    contradictory_metadata.stages[1].platform = Some("local".to_string());
    assert!(validate_benchmark_dossier_index(&contradictory_metadata).is_err());

    let mut unknown_source = index();
    unknown_source.stages[0].run_root_source = Some("scheduler-cache".to_string());
    assert!(validate_benchmark_dossier_index(&unknown_source).is_err());

    let mut source_root_drift = index();
    source_root_drift.stages[0].run_root = Some("/local/results/validate".to_string());
    assert!(validate_benchmark_dossier_index(&source_root_drift).is_err());

    let mut unknown_field = serde_json::to_value(index())?;
    unknown_field
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("dossier index did not serialize as an object"))?
        .insert("unowned_scheduler_state".to_string(), serde_json::json!("running"));
    assert!(serde_json::from_value::<BenchmarkDossierIndex>(unknown_field).is_err());
    Ok(())
}

#[test]
fn checked_in_dossier_index_is_admitted() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root.join("docs/30-operations/benchmark/corpus-01-dossier-index.json");
    let index: BenchmarkDossierIndex = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_benchmark_dossier_index(&index)?;
    Ok(())
}
