use bijux_dna_bench_model::{
    benchmark_citation_record_schema, benchmark_evidence_book_schema,
    benchmark_publication_receipt_schema, validate_benchmark_citation_record,
    validate_benchmark_evidence_book, validate_benchmark_publication_receipt,
    BenchmarkAttemptIdentity, BenchmarkCitationAssetIdentity, BenchmarkCitationRecord,
    BenchmarkCitationToolIdentity, BenchmarkEvidenceBook, BenchmarkEvidenceBookEntry,
    BenchmarkEvidenceBookStatus, BenchmarkEvidenceReference, BenchmarkPublicationReceipt,
    BenchmarkRunIdentity, BENCHMARK_CITATION_RECORD_SCHEMA_V1, BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
    BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
};

fn run_identity(run_id: &str) -> BenchmarkRunIdentity {
    BenchmarkRunIdentity {
        run_id: run_id.to_string(),
        domain: "fastq".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
    }
}

fn reference(uri: &str, digest: char) -> BenchmarkEvidenceReference {
    BenchmarkEvidenceReference {
        uri: uri.to_string(),
        sha256: digest.to_string().repeat(64),
        size_bytes: 42,
    }
}

fn publication() -> BenchmarkPublicationReceipt {
    let run = run_identity("run-2042");
    BenchmarkPublicationReceipt {
        schema_version: BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1.to_string(),
        publication_id: "run-2042-publication-20260806".to_string(),
        published_at_utc: "2026-08-06T12:00:00Z".to_string(),
        run: run.clone(),
        case_ids: vec![
            "run-2042-seqfu-real-fastq".to_string(),
            "run-2042-seqkit-real-fastq".to_string(),
        ],
        selected_attempts: vec![
            BenchmarkAttemptIdentity {
                campaign_id: "all-domains-20260806".to_string(),
                run: run.clone(),
                attempt_id: "attempt-001".to_string(),
                tool_id: "seqfu".to_string(),
            },
            BenchmarkAttemptIdentity {
                campaign_id: "all-domains-20260806".to_string(),
                run,
                attempt_id: "attempt-001".to_string(),
                tool_id: "seqkit".to_string(),
            },
        ],
        repository_commit: "a".repeat(40),
        catalog_sha256: "b".repeat(64),
        run_definition_sha256: "c".repeat(64),
        campaign_registry_sha256: "d".repeat(64),
        report: reference("study://results/benchmarks/run-2042/report.json", 'e'),
        decision: reference("study://results/benchmarks/run-2042/decision.json", 'f'),
        evidence: vec![
            reference("study://results/benchmarks/run-2042/seqfu/evidence.json", '1'),
            reference("study://results/benchmarks/run-2042/seqkit/evidence.json", '2'),
        ],
        limitations: vec!["Evidence covers the governed FASTQ portfolio only.".to_string()],
    }
}

fn book() -> BenchmarkEvidenceBook {
    BenchmarkEvidenceBook {
        schema_version: BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1.to_string(),
        book_id: "benchmark-evidence-20260806".to_string(),
        generated_at_utc: "2026-08-06T12:30:00Z".to_string(),
        repository_commit: "a".repeat(40),
        catalog_sha256: "b".repeat(64),
        status: BenchmarkEvidenceBookStatus::Incomplete,
        entries: vec![BenchmarkEvidenceBookEntry {
            publication_id: "run-2042-publication-20260806".to_string(),
            run: run_identity("run-2042"),
            citation_key: "run-2042-fastq-profile-read-lengths".to_string(),
            publication_receipt: reference(
                "study://evidence/benchmarks/run-2042/publication-receipt.json",
                '3',
            ),
        }],
        missing_run_ids: vec!["run-2043".to_string()],
        limitations: vec!["The remaining benchmark protocols are not yet admitted.".to_string()],
    }
}

fn citation() -> BenchmarkCitationRecord {
    let receipt = publication();
    BenchmarkCitationRecord {
        schema_version: BENCHMARK_CITATION_RECORD_SCHEMA_V1.to_string(),
        citation_key: "run-2042-fastq-profile-read-lengths".to_string(),
        run: receipt.run,
        protocol_title: "FASTQ read-length profiling benchmark".to_string(),
        protocol_version: "1.0.0".to_string(),
        protocol_sha256: "4".repeat(64),
        publication_id: receipt.publication_id,
        selected_attempts: receipt.selected_attempts,
        repository_commit: receipt.repository_commit,
        tools: vec![
            BenchmarkCitationToolIdentity {
                tool_id: "seqfu".to_string(),
                tool_version: "1.22.3".to_string(),
                image_sha256: "5".repeat(64),
            },
            BenchmarkCitationToolIdentity {
                tool_id: "seqkit".to_string(),
                tool_version: "2.10.0".to_string(),
                image_sha256: "6".repeat(64),
            },
        ],
        assets: vec![BenchmarkCitationAssetIdentity {
            role: "comparative_cohort".to_string(),
            asset_id: "ancient-dna-fastq-cohort".to_string(),
            sha256: "7".repeat(64),
        }],
        report: receipt.report,
        evidence_book: reference("study://evidence/benchmarks/evidence-book.json", '8'),
        publication_receipt: reference(
            "study://evidence/benchmarks/run-2042/publication-receipt.json",
            '9',
        ),
        limitations: receipt.limitations,
        access_statement: "Controlled corpus access is governed by the source data agreements."
            .to_string(),
    }
}

#[test]
fn publication_and_book_validate_and_round_trip() -> anyhow::Result<()> {
    let receipt = publication();
    validate_benchmark_publication_receipt(&receipt)?;
    let actual: BenchmarkPublicationReceipt =
        serde_json::from_slice(&serde_json::to_vec(&receipt)?)?;
    assert_eq!(actual, receipt);

    let evidence_book = book();
    validate_benchmark_evidence_book(&evidence_book)?;
    let actual: BenchmarkEvidenceBook =
        serde_json::from_slice(&serde_json::to_vec(&evidence_book)?)?;
    assert_eq!(actual, evidence_book);

    let citation_record = citation();
    validate_benchmark_citation_record(&citation_record)?;
    let actual: BenchmarkCitationRecord =
        serde_json::from_slice(&serde_json::to_vec(&citation_record)?)?;
    assert_eq!(actual, citation_record);
    Ok(())
}

#[test]
fn publication_and_book_reject_unknown_fields_and_states() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(publication())?;
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkPublicationReceipt>(value).is_err());

    let mut value = serde_json::to_value(book())?;
    value["status"] = serde_json::json!("invented");
    assert!(serde_json::from_value::<BenchmarkEvidenceBook>(value).is_err());

    let mut value = serde_json::to_value(citation())?;
    value["tools"][0]["invented"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkCitationRecord>(value).is_err());
    Ok(())
}

#[test]
fn publication_and_book_reject_ambiguous_selection() {
    let mut receipt = publication();
    receipt.selected_attempts[1].tool_id = "seqfu".to_string();
    assert!(validate_benchmark_publication_receipt(&receipt).is_err());

    let mut receipt = publication();
    receipt.report = receipt.decision.clone();
    assert!(validate_benchmark_publication_receipt(&receipt).is_err());

    let mut evidence_book = book();
    evidence_book.status = BenchmarkEvidenceBookStatus::Complete;
    assert!(validate_benchmark_evidence_book(&evidence_book).is_err());

    let mut citation_record = citation();
    citation_record.tools.swap(0, 1);
    assert!(validate_benchmark_citation_record(&citation_record).is_err());

    let mut citation_record = citation();
    citation_record.evidence_book = citation_record.report.clone();
    assert!(validate_benchmark_citation_record(&citation_record).is_err());
}

#[test]
fn checked_publication_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "publication-receipt.schema.json",
            serde_json::to_value(benchmark_publication_receipt_schema())?,
        ),
        ("evidence-book.schema.json", serde_json::to_value(benchmark_evidence_book_schema())?),
        ("citation-record.schema.json", serde_json::to_value(benchmark_citation_record_schema())?),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "publication schema drifted: {name}");
    }
    Ok(())
}
