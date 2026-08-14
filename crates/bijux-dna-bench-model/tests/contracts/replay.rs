use bijux_dna_bench_model::{
    benchmark_replay_receipt_schema, validate_benchmark_replay_receipt, BenchmarkAttemptIdentity,
    BenchmarkEvidenceReference, BenchmarkReplayOutcome, BenchmarkReplayReceipt,
    BenchmarkReplaySemanticIdentity, BenchmarkRunIdentity, BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1,
};

fn reference(uri: &str, digest: char) -> BenchmarkEvidenceReference {
    BenchmarkEvidenceReference {
        uri: uri.to_string(),
        sha256: digest.to_string().repeat(64),
        size_bytes: 42,
    }
}

fn semantics() -> BenchmarkReplaySemanticIdentity {
    BenchmarkReplaySemanticIdentity {
        decision_sha256: "a".repeat(64),
        report_sha256: "b".repeat(64),
        evidence_book_sha256: "c".repeat(64),
    }
}

fn receipt() -> BenchmarkReplayReceipt {
    let run = BenchmarkRunIdentity {
        run_id: "run-2042".to_string(),
        domain: "fastq".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
    };
    BenchmarkReplayReceipt {
        schema_version: BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1.to_string(),
        replay_id: "run-2042-replay-20260806".to_string(),
        replayed_at_utc: "2026-08-06T13:00:00Z".to_string(),
        run: run.clone(),
        publication_id: "run-2042-publication-20260806".to_string(),
        selected_attempts: vec![BenchmarkAttemptIdentity {
            campaign_id: "all-domains-20260806".to_string(),
            run,
            case_id: None,
            attempt_id: "attempt-001".to_string(),
            tool_id: "seqkit".to_string(),
        }],
        source_repository_commit: "d".repeat(40),
        replay_repository_commit: "e".repeat(40),
        source_publication: reference(
            "study://evidence/benchmarks/run-2042/publication-receipt.json",
            '1',
        ),
        run_definition: reference("study://contracts/runs/run-2042.json", '2'),
        case_definitions: vec![reference(
            "study://contracts/cases/run-2042-seqkit-real-fastq.json",
            '3',
        )],
        immutable_inputs: vec![reference(
            "study://data/fastq/ancient-dna-fastq-cohort/index.json",
            '4',
        )],
        expected: semantics(),
        observed: Some(semantics()),
        outcome: BenchmarkReplayOutcome::ExactMatch,
        differences: Vec::new(),
        limitations: vec![
            "Replay verifies semantic outputs rather than scheduler identity.".to_string()
        ],
    }
}

#[test]
fn replay_receipt_validates_and_round_trips() -> anyhow::Result<()> {
    let replay = receipt();
    validate_benchmark_replay_receipt(&replay)?;
    let actual: BenchmarkReplayReceipt = serde_json::from_slice(&serde_json::to_vec(&replay)?)?;
    assert_eq!(actual, replay);
    Ok(())
}

#[test]
fn replay_receipt_rejects_unknown_fields_and_states() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(receipt())?;
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkReplayReceipt>(value).is_err());

    let mut value = serde_json::to_value(receipt())?;
    value["outcome"] = serde_json::json!("invented");
    assert!(serde_json::from_value::<BenchmarkReplayReceipt>(value).is_err());
    Ok(())
}

#[test]
fn replay_receipt_rejects_contradictory_outcomes() {
    let mut replay = receipt();
    replay.outcome = BenchmarkReplayOutcome::Mismatch;
    assert!(validate_benchmark_replay_receipt(&replay).is_err());

    let mut replay = receipt();
    replay.outcome = BenchmarkReplayOutcome::Blocked;
    replay.differences = vec!["The governed input corpus is unavailable.".to_string()];
    assert!(validate_benchmark_replay_receipt(&replay).is_err());

    let mut replay = receipt();
    replay.observed = None;
    assert!(validate_benchmark_replay_receipt(&replay).is_err());

    let mut replay = receipt();
    replay.case_definitions.clear();
    assert!(validate_benchmark_replay_receipt(&replay).is_err());
}

#[test]
fn replay_receipt_accepts_truthful_mismatch_and_blocked_outcomes() -> anyhow::Result<()> {
    let mut mismatch = receipt();
    mismatch.outcome = BenchmarkReplayOutcome::Mismatch;
    let mut observed = semantics();
    observed.report_sha256 = "f".repeat(64);
    mismatch.observed = Some(observed);
    mismatch.differences =
        vec!["The regenerated report digest differs from publication.".to_string()];
    validate_benchmark_replay_receipt(&mismatch)?;

    let mut blocked = receipt();
    blocked.outcome = BenchmarkReplayOutcome::Blocked;
    blocked.observed = None;
    blocked.differences =
        vec!["The governed input corpus is unavailable at this site.".to_string()];
    validate_benchmark_replay_receipt(&blocked)?;
    Ok(())
}

#[test]
fn checked_replay_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_replay_receipt_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/replay-receipt.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "replay receipt schema drifted");
    Ok(())
}
