use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_local_corpus_run_schema, validate_benchmark_local_corpus_run,
    BenchmarkCorpusRunManifest, BenchmarkCorpusSampleRun, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
};

fn manifest() -> BenchmarkCorpusRunManifest {
    BenchmarkCorpusRunManifest {
        schema_version: BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1.to_string(),
        generated_at_utc: "2026-08-12T12:00:00Z".to_string(),
        corpus_id: "ancient-dna-fastq-v1".to_string(),
        stage_id: "fastq.trim_adapters".to_string(),
        scenario_id: "ancient-dna".to_string(),
        sample_scope: "full".to_string(),
        tool_kind: "benchmark".to_string(),
        platform: "local".to_string(),
        tools: vec!["cutadapt".to_string()],
        threads: 2,
        jobs: 1,
        sample_jobs: 1,
        sample_limit: None,
        dry_run: false,
        config_path: "benchmarks/config.toml".to_string(),
        publication_config_path: "benchmarks/publication.toml".to_string(),
        repo_root: "/workspace".to_string(),
        corpus_root: "/study/corpus".to_string(),
        out_root: "/study/attempt".to_string(),
        stage_args: vec![],
        samples_total: 1,
        samples_failed: 0,
        runs: vec![BenchmarkCorpusSampleRun {
            sample_id: "sample-001".to_string(),
            r1: "/study/corpus/sample-001_R1.fastq.gz".to_string(),
            r2: None,
            layout: "se".to_string(),
            status: "completed".to_string(),
            exit_code: 0,
            command: vec!["bijux-dna".to_string(), "fastq".to_string()],
            report_json: "/study/attempt/sample-001/report.json".to_string(),
            extra_fields: BTreeMap::new(),
        }],
        extra_fields: BTreeMap::new(),
    }
}

#[test]
fn corpus_run_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let manifest = manifest();
    validate_benchmark_local_corpus_run(&manifest)?;
    let bytes = serde_json::to_vec(&manifest)?;
    assert_eq!(serde_json::from_slice::<BenchmarkCorpusRunManifest>(&bytes)?, manifest);

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_corpus_run_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/local-corpus-run.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "FASTQ corpus run schema drifted from model authority");
    Ok(())
}

#[test]
fn corpus_run_rejects_counter_status_and_identity_drift() {
    let mut stale_total = manifest();
    stale_total.samples_total = 2;
    assert!(validate_benchmark_local_corpus_run(&stale_total).is_err());

    let mut contradictory = manifest();
    contradictory.runs[0].status = "failed".to_string();
    assert!(validate_benchmark_local_corpus_run(&contradictory).is_err());

    let mut duplicate = manifest();
    duplicate.runs.push(duplicate.runs[0].clone());
    duplicate.samples_total = 2;
    assert!(validate_benchmark_local_corpus_run(&duplicate).is_err());

    let mut colliding_extra = manifest();
    colliding_extra.extra_fields.insert("stage_id".to_string(), serde_json::json!("other"));
    assert!(validate_benchmark_local_corpus_run(&colliding_extra).is_err());
}
