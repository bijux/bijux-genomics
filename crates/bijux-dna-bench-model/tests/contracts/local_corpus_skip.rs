use bijux_dna_bench_model::{
    benchmark_local_corpus_skip_report_schema, validate_benchmark_local_corpus_skip_report,
    BenchmarkLocalAssetBackedStage, BenchmarkLocalCorpusFixture, BenchmarkLocalCorpusSkip,
    BenchmarkLocalCorpusSkipReport, BenchmarkLocalPlannerOnlyStage,
    BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
};

fn report() -> BenchmarkLocalCorpusSkipReport {
    BenchmarkLocalCorpusSkipReport {
        schema_version: BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1.to_string(),
        matrix_path: "configs/bench/local/corpus-stage-compatibility.toml".to_string(),
        output_path: "artifacts/benchmark-smoke/corpus-skip-report.json".to_string(),
        fixture_count: 2,
        stage_count: 3,
        skip_count: 1,
        asset_backed_stage_count: 1,
        planner_only_stage_count: 1,
        fixtures: vec![
            BenchmarkLocalCorpusFixture {
                corpus_id: "corpus-fastq".to_string(),
                corpus_family_id: "fastq".to_string(),
            },
            BenchmarkLocalCorpusFixture {
                corpus_id: "corpus-bam".to_string(),
                corpus_family_id: "bam".to_string(),
            },
        ],
        skips: vec![BenchmarkLocalCorpusSkip {
            stage_id: "fastq.trim_reads".to_string(),
            readiness_kind: "dry_or_smoke".to_string(),
            corpus_id: "corpus-bam".to_string(),
            corpus_family_id: "bam".to_string(),
            reason: "stage requires FASTQ fixture lineage".to_string(),
            replacement_corpus_id: "corpus-fastq".to_string(),
            replacement_corpus_family_id: "fastq".to_string(),
        }],
        asset_backed_stages: vec![BenchmarkLocalAssetBackedStage {
            stage_id: "vcf.prepare_reference_panel".to_string(),
            readiness_kind: "dry_or_smoke".to_string(),
            benchmark_scope_id: "reference-panel".to_string(),
            reason: "stage consumes a separately governed panel".to_string(),
        }],
        planner_only_stages: vec![BenchmarkLocalPlannerOnlyStage {
            stage_id: "bam.merge".to_string(),
            readiness_kind: "dry_run".to_string(),
            reason: "stage has no single-fixture execution contract".to_string(),
        }],
    }
}

#[test]
fn local_corpus_skip_report_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_corpus_skip_report(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalCorpusSkipReport>(&serde_json::to_vec(&report)?)?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_corpus_skip_report_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-corpus-skip-report.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local corpus-skip report schema drifted");
    Ok(())
}

#[test]
fn local_corpus_skip_report_rejects_reference_and_count_drift() {
    let mut count_drift = report();
    count_drift.skip_count = 2;
    assert!(validate_benchmark_local_corpus_skip_report(&count_drift).is_err());

    let mut reference_drift = report();
    reference_drift.skips[0].replacement_corpus_id = "missing".to_string();
    assert!(validate_benchmark_local_corpus_skip_report(&reference_drift).is_err());

    let mut disposition_drift = report();
    disposition_drift.asset_backed_stages[0].stage_id = "fastq.trim_reads".to_string();
    assert!(validate_benchmark_local_corpus_skip_report(&disposition_drift).is_err());

    let mut readiness_drift = report();
    readiness_drift.skips[0].readiness_kind = "unknown".to_string();
    assert!(validate_benchmark_local_corpus_skip_report(&readiness_drift).is_err());
}
