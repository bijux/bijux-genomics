use bijux_dna_bench_model::{
    benchmark_publication_status_schema, validate_benchmark_publication_status,
    BenchmarkPublicationAuditIssue, BenchmarkPublicationExcludedStage,
    BenchmarkPublicationStageStatus, BenchmarkPublicationStatusReport,
    BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1,
};

fn report() -> BenchmarkPublicationStatusReport {
    BenchmarkPublicationStatusReport {
        schema_version: BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1.to_string(),
        corpus_id: "ancient-dna-fastq-v1".to_string(),
        docs_root: "docs/benchmark".to_string(),
        benchmarkable_stage_count: 2,
        applicable_stage_count: 1,
        completed_stage_count: 0,
        incomplete_stage_count: 1,
        excluded_stage_count: 1,
        issue_count: 1,
        audit_warning_count: 0,
        audit_warnings: vec![],
        supplemental_findings_generated_at_utc: Some("2026-08-12T14:00:00Z".to_string()),
        excluded_stages: vec![BenchmarkPublicationExcludedStage {
            stage_id: "fastq.index_reference".to_string(),
            reason: "the corpus does not exercise reference indexing".to_string(),
        }],
        stages: vec![BenchmarkPublicationStageStatus {
            stage_id: "fastq.trim_reads".to_string(),
            scenario_id: "trimming-fairness".to_string(),
            sample_scope: "full".to_string(),
            contract_tool_roster: vec!["adapterremoval".to_string(), "cutadapt".to_string()],
            expected_tool_roster: vec!["adapterremoval".to_string(), "cutadapt".to_string()],
            method_path: "benchmark/fastq.trim_reads/method.md".to_string(),
            corpus_path: "benchmark/fastq.trim_reads/corpus".to_string(),
            status: "incomplete".to_string(),
            issue_count: 1,
            results_status: "incomplete".to_string(),
            results_issue_count: 1,
            results_selected_run_root: "/study/results/trim".to_string(),
            results_newest_available_run_root: "/study/results/trim".to_string(),
            results_selected_run_root_is_newest: true,
            issues: vec![BenchmarkPublicationAuditIssue {
                stage_id: "fastq.trim_reads".to_string(),
                issue_id: "sample-failures".to_string(),
                severity: "error".to_string(),
                detail: "one or more samples failed".to_string(),
            }],
        }],
    }
}

#[test]
fn publication_status_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_publication_status(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkPublicationStatusReport>(&serde_json::to_vec(&report)?)?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_publication_status_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/publication-status.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "publication status schema drifted from model authority");
    Ok(())
}

#[test]
fn publication_status_rejects_coverage_issue_and_root_drift() -> anyhow::Result<()> {
    let mut count_drift = report();
    count_drift.issue_count = 2;
    assert!(validate_benchmark_publication_status(&count_drift).is_err());

    let mut overlap = report();
    overlap.excluded_stages[0].stage_id = overlap.stages[0].stage_id.clone();
    assert!(validate_benchmark_publication_status(&overlap).is_err());

    let mut roster_drift = report();
    roster_drift.stages[0].expected_tool_roster = vec!["cutadapt".to_string()];
    assert!(validate_benchmark_publication_status(&roster_drift).is_err());

    let mut root_drift = report();
    root_drift.stages[0].results_newest_available_run_root = "/study/results/newer".to_string();
    assert!(validate_benchmark_publication_status(&root_drift).is_err());

    let mut unknown_field = serde_json::to_value(report())?;
    unknown_field
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("publication status did not serialize as an object"))?
        .insert("scheduler_state".to_string(), serde_json::json!("running"));
    assert!(serde_json::from_value::<BenchmarkPublicationStatusReport>(unknown_field).is_err());
    Ok(())
}

#[test]
fn checked_in_publication_status_is_admitted() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root.join("docs/30-operations/benchmark/corpus-01-status.json");
    let report: BenchmarkPublicationStatusReport = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_benchmark_publication_status(&report)?;
    Ok(())
}
