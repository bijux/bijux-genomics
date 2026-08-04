use bijux_dna_bench_model::{
    validate_attempt_archive, BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths,
    BenchmarkAttemptArchive, BenchmarkAttemptDisposition, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
};

fn archive(disposition: BenchmarkAttemptDisposition) -> BenchmarkAttemptArchive {
    BenchmarkAttemptArchive {
        schema_version: BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1.to_string(),
        run_id: "run-2043".to_string(),
        attempt_id: "slurm-3459974".to_string(),
        tool_id: "seqfu".to_string(),
        slurm_job_id: "3459974".to_string(),
        disposition,
        reason: None,
        archived_at_utc: "2026-08-04T00:00:00Z".to_string(),
        source_paths: BenchmarkArchiveSourcePaths {
            submission: "study://containers/evidence/submissions/run-2043-seqfu".to_string(),
            result: "study://results/benchmarks/run-2043-fastq-profile-reads/seqfu".to_string(),
        },
        evidence_sha256: "a".repeat(64),
        accounting_sha256: "b".repeat(64),
        files: vec![BenchmarkArchiveFileRecord {
            path: "result/evidence.json".to_string(),
            size_bytes: 42,
            sha256: "c".repeat(64),
        }],
    }
}

#[test]
fn failed_attempt_archive_is_identity_complete() {
    assert!(validate_attempt_archive(&archive(BenchmarkAttemptDisposition::Failed)).is_ok());
}

#[test]
fn superseded_attempt_archive_requires_reason() {
    let mut record = archive(BenchmarkAttemptDisposition::Superseded);
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("substantive reason")));

    record.reason = Some("measurement rows used an invalid delimiter".to_string());
    assert!(validate_attempt_archive(&record).is_ok());
}

#[test]
fn attempt_archive_rejects_malformed_digest_and_duplicate_path() {
    let mut record = archive(BenchmarkAttemptDisposition::Failed);
    record.files.push(record.files[0].clone());
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("duplicated")));

    record.files.pop();
    record.accounting_sha256 = "ABC".to_string();
    let error = validate_attempt_archive(&record).err().map(|value| value.to_string());
    assert!(error.is_some_and(|value| value.contains("SHA-256")));
}
