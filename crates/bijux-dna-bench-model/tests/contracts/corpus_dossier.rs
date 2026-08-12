use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_corpus_dossier_summary_schema, validate_benchmark_corpus_dossier_summary,
    BenchmarkCorpusDossierHeadline, BenchmarkCorpusDossierSummary,
    BenchmarkCorpusDossierToolSummary, BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
};

fn summary() -> BenchmarkCorpusDossierSummary {
    BenchmarkCorpusDossierSummary {
        schema_version: BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1.to_string(),
        corpus_id: "corpus-01".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        scenario_id: "validation_fairness".to_string(),
        generated_at_utc: "2026-08-12T15:00:00Z".to_string(),
        platform: "cluster-apptainer".to_string(),
        corpus_root: "/study/data/corpus-01".to_string(),
        run_root: "/study/results/corpus-01/fastq.validate_reads".to_string(),
        sample_scope: "full".to_string(),
        samples_total: 4,
        samples_failed: 0,
        tools: vec!["fastqvalidator".to_string(), "seqtk".to_string()],
        cohort_counts: BTreeMap::from([
            ("ancient_pe".to_string(), 1),
            ("ancient_se".to_string(), 1),
            ("modern_pe".to_string(), 1),
            ("modern_se".to_string(), 1),
        ]),
        era_counts: BTreeMap::from([("ancient".to_string(), 2), ("modern".to_string(), 2)]),
        layout_counts: BTreeMap::from([("pe".to_string(), 2), ("se".to_string(), 2)]),
        tool_summary: vec![
            BenchmarkCorpusDossierToolSummary {
                tool: "fastqvalidator".to_string(),
                records: 4,
                pass_rate: Some(1.0),
                median_runtime_s: Some(1.0),
                mean_runtime_s: Some(1.1),
                max_runtime_s: Some(1.4),
            },
            BenchmarkCorpusDossierToolSummary {
                tool: "seqtk".to_string(),
                records: 4,
                pass_rate: Some(0.75),
                median_runtime_s: Some(0.5),
                mean_runtime_s: Some(0.6),
                max_runtime_s: Some(0.9),
            },
        ],
        headline: BenchmarkCorpusDossierHeadline {
            fastest_tool: Some("seqtk".to_string()),
            fastest_runtime_s: Some(0.5),
            highest_pass_rate_tool: Some("fastqvalidator".to_string()),
            highest_pass_rate: Some(1.0),
        },
    }
}

#[test]
fn corpus_dossier_summary_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let summary = summary();
    validate_benchmark_corpus_dossier_summary(&summary)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkCorpusDossierSummary>(&serde_json::to_vec(&summary)?)?,
        summary
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_corpus_dossier_summary_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/corpus-dossier-summary.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "corpus dossier summary schema drifted from model authority");
    Ok(())
}

#[test]
fn corpus_dossier_summary_rejects_coverage_metric_and_headline_drift() {
    let mut coverage = summary();
    coverage.cohort_counts.insert("modern_se".to_string(), 2);
    assert!(validate_benchmark_corpus_dossier_summary(&coverage).is_err());

    let mut metric = summary();
    metric.tool_summary[0].pass_rate = Some(1.5);
    assert!(validate_benchmark_corpus_dossier_summary(&metric).is_err());

    let mut headline = summary();
    headline.headline.fastest_tool = Some("fastqvalidator".to_string());
    assert!(validate_benchmark_corpus_dossier_summary(&headline).is_err());
}

#[test]
fn corpus_dossier_summary_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(summary())?;
    value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("corpus dossier summary did not serialize as an object"))?
        .insert("scheduler_state".to_string(), serde_json::json!("running"));
    assert!(serde_json::from_value::<BenchmarkCorpusDossierSummary>(value).is_err());
    Ok(())
}
