use bijux_dna_bench_model::{
    benchmark_published_results_status_schema, validate_benchmark_published_results_status,
    PublishedResultsStageReport, PublishedResultsStatusReport, StageResultIssue,
    BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
};

fn report() -> PublishedResultsStatusReport {
    PublishedResultsStatusReport {
        schema_version: BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1.to_string(),
        corpus_id: "ancient-dna-fastq-v1".to_string(),
        applicable_stage_count: 2,
        published_stage_count: 1,
        complete_stage_count: 1,
        incomplete_stage_count: 1,
        issue_count: 1,
        stages: vec![
            PublishedResultsStageReport {
                stage_id: "fastq.quality_control".to_string(),
                status: "complete".to_string(),
                issue_count: 0,
                reported_run_root: "/study/reported".to_string(),
                selected_run_root: "/study/current".to_string(),
                newest_available_run_root: "/study/current".to_string(),
                selected_run_root_is_newest: true,
                available_run_roots: vec!["/study/current".to_string()],
                issues: vec![],
            },
            PublishedResultsStageReport {
                stage_id: "fastq.trimming".to_string(),
                status: "incomplete".to_string(),
                issue_count: 1,
                reported_run_root: String::new(),
                selected_run_root: String::new(),
                newest_available_run_root: String::new(),
                selected_run_root_is_newest: false,
                available_run_roots: vec![],
                issues: vec![StageResultIssue {
                    stage_id: "fastq.trimming".to_string(),
                    issue_id: "missing-published-summary".to_string(),
                    detail: "summary.json is absent".to_string(),
                }],
            },
        ],
    }
}

#[test]
fn published_results_status_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_published_results_status(&report)?;
    let bytes = serde_json::to_vec(&report)?;
    assert_eq!(serde_json::from_slice::<PublishedResultsStatusReport>(&bytes)?, report);

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_published_results_status_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/published-results-status.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "published-results status schema drifted from model authority");
    Ok(())
}

#[test]
fn published_results_status_rejects_counter_identity_and_root_drift() -> anyhow::Result<()> {
    let mut stale_counter = report();
    stale_counter.issue_count = 2;
    assert!(validate_benchmark_published_results_status(&stale_counter).is_err());

    let mut stale_schema = report();
    stale_schema.schema_version = "bijux.bench.published_results_status.v0".to_string();
    assert!(validate_benchmark_published_results_status(&stale_schema).is_err());

    let mut unpublished_complete = report();
    unpublished_complete.published_stage_count = 0;
    assert!(validate_benchmark_published_results_status(&unpublished_complete).is_err());

    let mut stale_published = report();
    stale_published.published_stage_count = 2;
    assert!(validate_benchmark_published_results_status(&stale_published).is_err());

    let mut duplicate_stage = report();
    duplicate_stage.stages[1].stage_id = duplicate_stage.stages[0].stage_id.clone();
    assert!(validate_benchmark_published_results_status(&duplicate_stage).is_err());

    let mut wrong_issue_owner = report();
    wrong_issue_owner.stages[1].issues[0].stage_id = "fastq.other".to_string();
    assert!(validate_benchmark_published_results_status(&wrong_issue_owner).is_err());

    let mut duplicate_issue = report();
    let repeated_issue = duplicate_issue.stages[1].issues[0].clone();
    duplicate_issue.stages[1].issues.push(repeated_issue);
    duplicate_issue.stages[1].issue_count = 2;
    duplicate_issue.issue_count = 2;
    assert!(validate_benchmark_published_results_status(&duplicate_issue).is_err());

    let mut contradictory_root = report();
    contradictory_root.stages[0].newest_available_run_root = "/study/newer".to_string();
    assert!(validate_benchmark_published_results_status(&contradictory_root).is_err());

    let mut unknown_field = serde_json::to_value(report())?;
    unknown_field
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("published-results report did not serialize as an object"))?
        .insert("unowned_lifecycle_state".to_string(), serde_json::json!(true));
    assert!(serde_json::from_value::<PublishedResultsStatusReport>(unknown_field).is_err());
    Ok(())
}

#[test]
fn checked_in_published_results_status_is_admitted() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root.join("docs/30-operations/benchmark/corpus-01-results-status.json");
    let report: PublishedResultsStatusReport = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_benchmark_published_results_status(&report)?;
    Ok(())
}
