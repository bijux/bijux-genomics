use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_local_corpus_stage_compatibility_schema,
    validate_benchmark_local_corpus_stage_compatibility_report,
    BenchmarkLocalCorpusStageCompatibilityEntry, BenchmarkLocalCorpusStageCompatibilityReport,
    BenchmarkLocalCorpusStageValidatedFixture,
    BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
};

fn report() -> BenchmarkLocalCorpusStageCompatibilityReport {
    BenchmarkLocalCorpusStageCompatibilityReport {
        schema_version: BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1.to_string(),
        matrix_path: "configs/bench/local/corpus-stage-compatibility.toml".to_string(),
        fixture_count: 2,
        stage_count: 3,
        fixture_backed_stage_count: 1,
        asset_backed_stage_count: 1,
        planner_only_stage_count: 1,
        corpus_family_counts: BTreeMap::from([("fastq-general".to_string(), 1)]),
        valid: true,
        fixtures: vec![
            BenchmarkLocalCorpusStageValidatedFixture {
                corpus_family_id: "fastq-general".to_string(),
                fixture_id: "corpus-fastq".to_string(),
                fixture_manifest: "benchmarks/tests/fixtures/fastq.toml".to_string(),
                summary: "general FASTQ fixture".to_string(),
            },
            BenchmarkLocalCorpusStageValidatedFixture {
                corpus_family_id: "bam-general".to_string(),
                fixture_id: "corpus-bam".to_string(),
                fixture_manifest: "benchmarks/tests/fixtures/bam.toml".to_string(),
                summary: "general BAM fixture".to_string(),
            },
        ],
        stages: vec![
            BenchmarkLocalCorpusStageCompatibilityEntry {
                stage_id: "fastq.trim_reads".to_string(),
                readiness_kind: "dry_or_smoke".to_string(),
                compatibility_kind: "fixture".to_string(),
                corpus_family_id: Some("fastq-general".to_string()),
                fixture_id: Some("corpus-fastq".to_string()),
                benchmark_scope_id: None,
                compatibility_note: "validated with paired FASTQ input".to_string(),
            },
            BenchmarkLocalCorpusStageCompatibilityEntry {
                stage_id: "fastq.index_reference".to_string(),
                readiness_kind: "dry_or_smoke".to_string(),
                compatibility_kind: "asset_backed".to_string(),
                corpus_family_id: None,
                fixture_id: None,
                benchmark_scope_id: Some("reference-index-assets".to_string()),
                compatibility_note: "validated by governed reference assets".to_string(),
            },
            BenchmarkLocalCorpusStageCompatibilityEntry {
                stage_id: "bam.merge".to_string(),
                readiness_kind: "dry_run".to_string(),
                compatibility_kind: "planner_only".to_string(),
                corpus_family_id: None,
                fixture_id: None,
                benchmark_scope_id: None,
                compatibility_note: "validated by planner contract".to_string(),
            },
        ],
    }
}

#[test]
fn local_corpus_stage_compatibility_round_trips_and_matches_generated_schema() -> anyhow::Result<()>
{
    let report = report();
    validate_benchmark_local_corpus_stage_compatibility_report(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalCorpusStageCompatibilityReport>(
            &serde_json::to_vec(&report)?
        )?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_corpus_stage_compatibility_schema())?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-corpus-stage-compatibility.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local corpus-stage compatibility schema drifted");
    Ok(())
}

#[test]
fn local_corpus_stage_compatibility_rejects_lineage_and_count_drift() {
    let mut family_drift = report();
    family_drift.stages[0].corpus_family_id = Some("bam-general".to_string());
    assert!(validate_benchmark_local_corpus_stage_compatibility_report(&family_drift).is_err());

    let mut count_drift = report();
    count_drift.asset_backed_stage_count = 0;
    assert!(validate_benchmark_local_corpus_stage_compatibility_report(&count_drift).is_err());

    let mut disposition_drift = report();
    disposition_drift.stages[2].benchmark_scope_id = Some("unexpected".to_string());
    assert!(validate_benchmark_local_corpus_stage_compatibility_report(&disposition_drift).is_err());
}
