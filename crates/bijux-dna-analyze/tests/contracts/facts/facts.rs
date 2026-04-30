use bijux_dna_runtime::*;
use std::fs;
use std::path::PathBuf;

use bijux_dna_analyze::exports::{
    build_evidence_bundle, compare_evidence_bundles, validate_evidence_bundle_profile,
    verify_evidence_bundle, summarize_facts, write_evidence_bundle_json, write_run_summary_json,
    write_stage_summary_csv, EvidenceBundleProfileV1,
};
use bijux_dna_analyze::load::load_facts;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-analyze__{group}__{name}")
}

fn facts_row(input_hash: &str) -> FactsRowV1 {
    FactsRowV1 {
        schema_version: "bijux.facts.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        tool_version: "0.23.4".to_string(),
        image_digest: Some("sha256:abc".to_string()),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        params_hash: "ph".to_string(),
        input_hash: input_hash.to_string(),
        output_hashes: vec!["oh".to_string()],
        runtime_s: 1.5,
        memory_mb: 42.0,
        exit_code: 0,
        bank_hashes: serde_json::json!({"adapters": "hash"}),
        reads_in: Some(10),
        reads_out: Some(9),
        bases_in: Some(100),
        bases_out: Some(90),
        pairs_in: None,
        pairs_out: None,
        metrics: serde_json::json!({}),
        reports: serde_json::json!({
            "stage_report": "stage_report.json",
            "retention_report": "retention_report.json"
        }),
        artifacts: serde_json::json!({"metrics_envelope": "metrics.json"}),
    }
}

#[test]
fn facts_loader_and_summary_work() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let row = facts_row("ih");
    let payload = serde_json::to_string(&row)?;
    bijux_dna_infra::write_bytes(&path, format!("{payload}\n"))?;

    let rows = load_facts(&path).map_err(|err| anyhow::anyhow!(err.to_string()))?;
    assert_eq!(rows.len(), 1);
    let summary = summarize_facts(&rows);
    assert_eq!(summary.runs, 1);
    assert_eq!(summary.stages, 1);
    assert!((summary.total_runtime_s - 1.5).abs() < 1e-6);

    let summary_path = dir.path().join("run_summary.json");
    write_run_summary_json(&summary_path, &rows)?;
    let summary_json: serde_json::Value = serde_json::from_str(&fs::read_to_string(summary_path)?)?;
    assert_eq!(summary_json["schema_version"], "bijux.run_summary.v1");
    assert_eq!(summary_json["facts_path"], "facts.jsonl");
    assert_eq!(summary_json["report_path"], "report.json");
    assert_eq!(summary_json["telemetry_path"], "telemetry/events.jsonl");
    assert_eq!(summary_json["runs"], 1);
    assert_eq!(summary_json["stages"], 1);
    assert_eq!(summary_json["stage_rows"][0]["tool_version"], "0.23.4");
    assert_eq!(summary_json["stage_rows"][0]["image_digest"], "sha256:abc");
    assert_eq!(summary_json["stage_rows"][0]["bank_hashes"]["adapters"], "hash");

    Ok(())
}

#[test]
fn facts_loader_orders_full_identity_key() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let row_b = facts_row("input-b");
    let row_a = facts_row("input-a");
    bijux_dna_infra::write_bytes(
        &path,
        format!("{}\n{}\n", serde_json::to_string(&row_b)?, serde_json::to_string(&row_a)?),
    )?;

    let rows = load_facts(&path).map_err(|err| anyhow::anyhow!(err.to_string()))?;

    assert_eq!(
        rows.iter().map(|row| row.input_hash.as_str()).collect::<Vec<_>>(),
        vec!["input-a", "input-b"]
    );
    Ok(())
}

#[test]
fn stage_summary_csv_quotes_carriage_returns() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("stage_summary.csv");
    let mut row = facts_row("ih");
    row.tool_version = "0.23\r4".to_string();

    write_stage_summary_csv(&path, &[row])?;
    let csv = fs::read_to_string(&path)?;

    assert!(csv.contains("\"0.23\r4\""));
    Ok(())
}

#[test]
fn run_summary_snapshot_is_stable() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let summary_path = dir.path().join("run_summary.json");
    let rows = vec![
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-2".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: Some("sha256:abc".to_string()),
            trace_id: "trace-2".to_string(),
            span_id: "span-2".to_string(),
            params_hash: "ph2".to_string(),
            input_hash: "ih2".to_string(),
            output_hashes: vec!["oh2".to_string()],
            runtime_s: 2.0,
            memory_mb: 43.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({"adapters": "hash2"}),
            reads_in: Some(20),
            reads_out: Some(18),
            bases_in: Some(200),
            bases_out: Some(180),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({"stage_report": "stage_report.json"}),
            artifacts: serde_json::json!({}),
        },
        FactsRowV1 {
            schema_version: "bijux.facts.v1".to_string(),
            run_id: "run-2".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "fastqvalidator".to_string(),
            tool_version: "1.0".to_string(),
            image_digest: Some("sha256:def".to_string()),
            trace_id: "trace-3".to_string(),
            span_id: "span-3".to_string(),
            params_hash: "ph3".to_string(),
            input_hash: "ih3".to_string(),
            output_hashes: vec!["oh3".to_string()],
            runtime_s: 1.0,
            memory_mb: 30.0,
            exit_code: 0,
            bank_hashes: serde_json::json!({}),
            reads_in: Some(20),
            reads_out: Some(20),
            bases_in: Some(200),
            bases_out: Some(200),
            pairs_in: None,
            pairs_out: None,
            metrics: serde_json::json!({}),
            reports: serde_json::json!({"stage_report": "stage_report.json"}),
            artifacts: serde_json::json!({}),
        },
    ];
    write_run_summary_json(&summary_path, &rows)?;
    let summary_raw = fs::read_to_string(&summary_path)?;
    let summary_value: serde_json::Value = serde_json::from_str(&summary_raw)?;
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(snapshot_name("schemas", "run_summary"))
        .with_extension("json");
    let snapshot_raw = fs::read_to_string(&snapshot_path)?;
    let snapshot_value: serde_json::Value = serde_json::from_str(&snapshot_raw)?;
    assert_eq!(summary_value, snapshot_value);
    Ok(())
}

#[test]
fn evidence_bundle_builds_and_verifies() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let facts_path = dir.path().join("facts.jsonl");
    let rows = vec![facts_row("ih")];
    bijux_dna_infra::write_bytes(
        &facts_path,
        rows.iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
            + "\n",
    )?;
    let summary_path = dir.path().join("run_summary.json");
    write_run_summary_json(&summary_path, &rows)?;
    let report_path = dir.path().join("report.json");
    bijux_dna_infra::atomic_write_json(
        &report_path,
        &serde_json::json!({
            "schema_version": "bijux.run_report.v1",
            "completeness": { "status": "complete", "missing_metrics": [], "missing_reports": [] },
            "pipeline_verdict": { "verdict": "pass", "reasons": [] }
        }),
    )?;
    let graph_path = dir.path().join("graph.json");
    bijux_dna_infra::write_bytes(&graph_path, r#"{"graph":"ok"}"#)?;
    let telemetry_dir = dir.path().join("telemetry");
    bijux_dna_infra::ensure_dir(&telemetry_dir)?;
    let telemetry_path = telemetry_dir.join("events.jsonl");
    let telemetry = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        event_name: TelemetryEventName::StageStart,
        timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-01T00:00:01Z")?.with_timezone(&chrono::Utc),
        duration_ms: Some(10),
        status: "ok".to_string(),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        attrs: AttrMap::new(),
        failure_code: None,
    };
    bijux_dna_infra::write_bytes(&telemetry_path, format!("{}\n", serde_json::to_string(&telemetry)?))?;
    let validated_path = dir.path().join("validated.fastq");
    bijux_dna_infra::write_bytes(&validated_path, "ACGT")?;
    let summary_hash = bijux_dna_infra::hash_file_sha256(&summary_path)?;
    let report_hash = bijux_dna_infra::hash_file_sha256(&report_path)?;
    let artifact_hash = bijux_dna_infra::hash_file_sha256(&validated_path)?;
    let artifact_inventory_path = dir.path().join("artifact_inventory.json");
    bijux_dna_infra::atomic_write_json(
        &artifact_inventory_path,
        &serde_json::json!({
            "schema_version": "bijux.artifact_inventory.v1",
            "run_id": "run-1",
            "artifacts": [{
                "artifact_id": "validated_reads",
                "name": "validated_reads",
                "role": "Reads",
                "path": "validated.fastq",
                "sha256": artifact_hash,
                "input_lineage": ["sha256:input"]
            }]
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("replay_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.replay_manifest.v1",
            "replay_run_id": "run-1",
            "original_run_id": "run-1",
            "selected_stage_ids": ["fastq.trim_reads"],
            "reused_artifact_ids": [],
            "rerun_stage_ids": ["fastq.trim_reads"],
            "expected_outputs": ["validated_reads"],
            "cache_decisions": [],
            "environment_differences": []
        }),
    )?;
    let summary_text_path = dir.path().join("summary").join("run_summary.txt");
    std::fs::create_dir_all(summary_text_path.parent().unwrap_or(dir.path()))?;
    std::fs::write(
        &summary_text_path,
        b"what_was_checked:\n- fastq.trim_reads\nsafe_outputs:\n- validated.fastq\n",
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("run_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_manifest.v3",
            "run_id": "run-1",
            "correlation_id": "corr-run-1",
            "mode": "enforced",
            "graph_hash": "sha256:graph",
            "cache_key": { "semantic_hash": "sha256:sem", "params_hash": "sha256:param", "tool_version": "0.23.4", "image_digest": "sha256:img" },
            "dataset_fingerprints": ["sha256:input"],
            "stages": [{ "stage_id": "fastq.trim_reads" }],
            "output_artifacts": [
                { "name": "run_summary", "path": "run_summary.json", "sha256": summary_hash },
                { "name": "report", "path": "report.json", "sha256": report_hash },
                { "name": "validated_reads", "path": "validated.fastq", "sha256": artifact_hash }
            ],
            "execution_replay_identity": { "tool_image_digest": "sha256:img" }
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("plan_manifest.json"),
        &serde_json::json!({
            "schema_version": "bijux.plan_manifest.v1",
            "pipeline_id": "fastq-to-fastq__default__v1"
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("run_state.json"),
        &serde_json::json!({
            "schema_version": "bijux.run_state.v1",
            "run_id": "run-1",
            "mode": "enforced",
            "state": "succeeded",
            "transitions": []
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("runtime_policy.json"),
        &serde_json::json!({
            "schema_version": "bijux.runtime_policy.v1",
            "run_id": "run-1",
            "mode": "enforced",
            "deterministic_scheduler": true,
            "retry_policy": { "max_attempts": 1, "retry_on_exit_codes": [] },
            "cancellation": {
                "supports_external_cancellation": false,
                "checkpoint_before_cancel": false
            },
            "checkpoint": {
                "strategy": "none",
                "granularity": "stage",
                "resume_from_latest_completed_stage": false
            }
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("evidence_verification.json"),
        &serde_json::json!({
            "schema_version": "bijux.evidence_verification.v1",
            "verified": true,
            "checks": [],
            "missing_paths": [],
            "gap_count": 0
        }),
    )?;
    let manifest_hash = bijux_dna_infra::hash_file_sha256(&dir.path().join("run_manifest.json"))?;
    let summary_text_hash = bijux_dna_infra::hash_file_sha256(&summary_text_path)?;
    bijux_dna_infra::atomic_write_json(
        &dir.path().join("hash_ledger.json"),
        &serde_json::json!({
            "schema_version": "bijux.hash_ledger.v1",
            "run_id": "run-1",
            "root_sha256": "placeholder",
            "entries": [
                {
                    "record_id": "run_manifest.json",
                    "kind": "run_manifest",
                    "path": "run_manifest.json",
                    "sha256": manifest_hash,
                    "previous_entry_sha256": serde_json::Value::Null
                },
                {
                    "record_id": "summary:run_summary.txt",
                    "kind": "run_summary_text",
                    "path": "summary/run_summary.txt",
                    "sha256": summary_text_hash,
                    "previous_entry_sha256": manifest_hash
                }
            ]
        }),
    )?;

    let bundle = build_evidence_bundle(dir.path(), Some(&facts_path))?;
    assert_eq!(bundle.correlation_id, "corr-run-1");
    assert_eq!(bundle.compact_summary.stage_count, 1);
    assert!(bundle.timeline.iter().any(|event| event.event == "cache_key_declared"));
    assert!(bundle.timeline.iter().any(|event| event.event == "artifact_written" || event.event == "stage_start"));
    assert!(bundle.health.gaps.is_empty(), "unexpected gaps: {:?}", bundle.health.gaps);
    assert!(bundle
        .provenance_graph
        .nodes
        .iter()
        .any(|node| node.kind == "input" && node.label == "sha256:input"));

    let bundle_path = write_evidence_bundle_json(dir.path(), Some(&facts_path))?;
    let verification = verify_evidence_bundle(&bundle_path)?;
    assert!(verification.verified, "verification failed: {:?}", verification.checks);
    let operational = validate_evidence_bundle_profile(&bundle, EvidenceBundleProfileV1::Operational);
    assert!(operational.ok, "operational profile failed: {:?}", operational.checks);
    let publication = validate_evidence_bundle_profile(&bundle, EvidenceBundleProfileV1::Publication);
    assert!(publication.ok, "publication profile failed: {:?}", publication.checks);
    Ok(())
}

#[test]
fn evidence_bundle_comparison_surfaces_artifact_drift() -> anyhow::Result<()> {
    let left = bijux_dna_infra::temp_dir("bijux")?;
    let right = bijux_dna_infra::temp_dir("bijux")?;
    for (dir, bytes, correlation) in [
        (left.path(), "AAAA", "corr-a"),
        (right.path(), "CCCC", "corr-b"),
    ] {
        let artifact = dir.join("artifact.txt");
        bijux_dna_infra::write_bytes(&artifact, bytes)?;
        let hash = bijux_dna_infra::hash_file_sha256(&artifact)?;
        bijux_dna_infra::atomic_write_json(
            &dir.join("run_manifest.json"),
            &serde_json::json!({
                "schema_version": "bijux.run_manifest.v3",
                "run_id": format!("run-{correlation}"),
                "correlation_id": correlation,
                "graph_hash": "sha256:graph",
                "dataset_fingerprints": ["sha256:input"],
                "stages": [{ "stage_id": "fastq.trim_reads" }],
                "output_artifacts": [{ "name": "artifact", "path": "artifact.txt", "sha256": hash }]
            }),
        )?;
        let bundle_path = dir.join("evidence_bundle.json");
        bijux_dna_infra::atomic_write_json(
            &bundle_path,
            &build_evidence_bundle(dir, None)?,
        )?;
    }

    let comparison = compare_evidence_bundles(
        &left.path().join("evidence_bundle.json"),
        &right.path().join("evidence_bundle.json"),
    )?;
    assert!(comparison.changed_artifacts.iter().any(|name| name == "artifact"));
    assert!(comparison
        .policy_change_hints
        .iter()
        .any(|hint| hint.contains("artifact inventory or hashes changed")));
    Ok(())
}

#[test]
fn draft_profile_tolerates_missing_publication_material() {
    let bundle = bijux_dna_analyze::exports::EvidenceBundleV1 {
        schema_version: "bijux.evidence_bundle.v1".to_string(),
        run_id: "run-1".to_string(),
        correlation_id: "corr-1".to_string(),
        sources: bijux_dna_analyze::exports::EvidenceSourcesV1 {
            manifest_path: Some("run_manifest.json".to_string()),
            plan_manifest_path: None,
            report_path: None,
            run_summary_path: None,
            facts_path: None,
            graph_path: None,
            environment_path: None,
            runtime_policy_path: None,
            run_state_path: None,
            executor_descriptor_path: None,
            checkpoint_path: None,
            failure_path: None,
            artifact_inventory_path: Some("artifact_inventory.json".to_string()),
            replay_manifest_path: None,
            hash_ledger_path: None,
            evidence_verification_path: None,
            telemetry_paths: vec!["telemetry/events.jsonl".to_string()],
        },
        compact_summary: bijux_dna_analyze::exports::EvidenceCompactSummaryV1 {
            stage_count: 1,
            artifact_count: 1,
            failed_stage_count: 0,
            advisory_gap_count: 2,
            final_outputs: vec!["reads.cleaned".to_string()],
            stage_ids: vec!["fastq.trim_reads".to_string()],
        },
        health: bijux_dna_analyze::exports::EvidenceHealthV1 {
            status: "advisory".to_string(),
            auditable: true,
            checks: Vec::new(),
            gaps: vec![
                bijux_dna_analyze::exports::EvidenceGapV1 {
                    code: "missing_hash_ledger".to_string(),
                    severity: bijux_dna_analyze::exports::EvidenceSeverityV1::Advisory,
                    message: "hash ledger not written yet".to_string(),
                    path: Some("hash_ledger.json".to_string()),
                    blocks_audit: false,
                },
                bijux_dna_analyze::exports::EvidenceGapV1 {
                    code: "missing_report".to_string(),
                    severity: bijux_dna_analyze::exports::EvidenceSeverityV1::Advisory,
                    message: "report not written yet".to_string(),
                    path: Some("report.json".to_string()),
                    blocks_audit: false,
                },
            ],
        },
        metrics: bijux_dna_analyze::exports::EvidenceMetricsV1 {
            queue_time_ms: None,
            run_time_s: 1.0,
            retry_count: 0,
            cache_hit_count: 0,
            cache_miss_count: 1,
            total_timeline_events: 1,
            scientific_failure_classes: std::collections::BTreeMap::new(),
        },
        timeline: Vec::new(),
        artifacts: Vec::new(),
        provenance_graph: bijux_dna_analyze::exports::EvidenceProvenanceGraphV1 {
            nodes: Vec::new(),
            edges: Vec::new(),
        },
    };

    let draft = validate_evidence_bundle_profile(&bundle, EvidenceBundleProfileV1::Draft);
    assert!(draft.ok);
    assert!(draft.blocking_gap_codes.is_empty());

    let publication = validate_evidence_bundle_profile(&bundle, EvidenceBundleProfileV1::Publication);
    assert!(!publication.ok);
    assert!(!publication.required_paths_present || !publication.blocking_gap_codes.is_empty());
}
