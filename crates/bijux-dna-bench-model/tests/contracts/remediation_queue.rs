use bijux_dna_bench_model::{
    benchmark_remediation_queue_schema, validate_benchmark_remediation_queue,
    BenchmarkRemediationIssue, BenchmarkRemediationIssueGroup, BenchmarkRemediationQueue,
    BenchmarkRemediationStageEntry, BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1,
};

fn queue() -> BenchmarkRemediationQueue {
    BenchmarkRemediationQueue {
        schema_version: BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1.to_string(),
        corpus_id: "corpus-01".to_string(),
        stage_count: 2,
        open_stage_count: 1,
        clear_stage_count: 1,
        stages: vec![
            BenchmarkRemediationStageEntry {
                stage_id: "fastq.validate_reads".to_string(),
                owner: "benchmark-governance".to_string(),
                status: "clear".to_string(),
                issue_count: 0,
                issue_group_count: 0,
                recommended_action: "none".to_string(),
                publication_status: "complete".to_string(),
                results_status: "complete".to_string(),
                sample_scope: "full".to_string(),
                published_generated_at_utc: Some("2026-08-12T14:00:00Z".to_string()),
                run_root_source: Some("local-results-root".to_string()),
                issue_groups: vec![],
                issues: vec![],
            },
            BenchmarkRemediationStageEntry {
                stage_id: "fastq.trim_reads".to_string(),
                owner: "benchmark-governance".to_string(),
                status: "open".to_string(),
                issue_count: 2,
                issue_group_count: 1,
                recommended_action: "rerun-benchmark-stage".to_string(),
                publication_status: "incomplete".to_string(),
                results_status: "incomplete".to_string(),
                sample_scope: "full".to_string(),
                published_generated_at_utc: Some("2026-08-12T14:00:00Z".to_string()),
                run_root_source: Some("local-cache-mirror".to_string()),
                issue_groups: vec![BenchmarkRemediationIssueGroup {
                    issue_id: "sample-failures".to_string(),
                    count: 2,
                    sources: vec!["publication".to_string(), "results".to_string()],
                    severity: "error".to_string(),
                    example_details: vec![
                        "published samples_failed=20".to_string(),
                        "run samples_failed=20".to_string(),
                    ],
                    additional_detail_count: 0,
                }],
                issues: vec![
                    BenchmarkRemediationIssue {
                        issue_id: "sample-failures".to_string(),
                        detail: "published samples_failed=20".to_string(),
                        severity: "error".to_string(),
                        source: "publication".to_string(),
                    },
                    BenchmarkRemediationIssue {
                        issue_id: "sample-failures".to_string(),
                        detail: "run samples_failed=20".to_string(),
                        severity: "error".to_string(),
                        source: "results".to_string(),
                    },
                ],
            },
        ],
    }
}

#[test]
fn remediation_queue_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let queue = queue();
    validate_benchmark_remediation_queue(&queue)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkRemediationQueue>(&serde_json::to_vec(&queue)?)?,
        queue
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_remediation_queue_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/remediation-queue.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "remediation queue schema drifted from model authority");
    Ok(())
}

#[test]
fn remediation_queue_rejects_counter_status_and_group_drift() -> anyhow::Result<()> {
    let mut counter_drift = queue();
    counter_drift.open_stage_count = 2;
    assert!(validate_benchmark_remediation_queue(&counter_drift).is_err());

    let mut status_drift = queue();
    status_drift.stages[0].results_status = "missing".to_string();
    assert!(validate_benchmark_remediation_queue(&status_drift).is_err());

    let mut group_drift = queue();
    group_drift.stages[1].issue_groups[0].count = 1;
    assert!(validate_benchmark_remediation_queue(&group_drift).is_err());

    let mut action_drift = queue();
    action_drift.stages[1].recommended_action = "render-or-publish-dossier".to_string();
    assert!(validate_benchmark_remediation_queue(&action_drift).is_err());

    let mut metadata_drift = queue();
    metadata_drift.stages[0].published_generated_at_utc = None;
    metadata_drift.stages[0].run_root_source = None;
    assert!(validate_benchmark_remediation_queue(&metadata_drift).is_err());

    let mut unknown_field = serde_json::to_value(queue())?;
    unknown_field
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("remediation queue did not serialize as an object"))?
        .insert("scheduler_state".to_string(), serde_json::json!("running"));
    assert!(serde_json::from_value::<BenchmarkRemediationQueue>(unknown_field).is_err());
    Ok(())
}

#[test]
fn checked_in_remediation_queue_is_admitted() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("docs/30-operations/benchmark/corpus-01-remediation-queue.json");
    let queue: BenchmarkRemediationQueue = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_benchmark_remediation_queue(&queue)?;
    Ok(())
}
