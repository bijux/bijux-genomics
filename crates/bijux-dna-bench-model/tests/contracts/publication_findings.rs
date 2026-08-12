use bijux_dna_bench_model::{
    benchmark_publication_findings_schema, validate_benchmark_publication_findings,
    BenchmarkPublicationAuditIssue, BenchmarkPublicationFindings,
    BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
};

fn findings() -> BenchmarkPublicationFindings {
    BenchmarkPublicationFindings {
        schema_version: BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1.to_string(),
        corpus_id: "corpus-01".to_string(),
        generated_at_utc: "2026-08-12T16:00:00Z".to_string(),
        findings: vec![BenchmarkPublicationAuditIssue {
            stage_id: "fastq.correct_errors".to_string(),
            issue_id: "missing-run-report-json".to_string(),
            severity: "error".to_string(),
            detail: "the stage run has no report.json".to_string(),
        }],
    }
}

#[test]
fn publication_findings_round_trip_and_match_generated_schema() -> anyhow::Result<()> {
    let findings = findings();
    validate_benchmark_publication_findings(&findings)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkPublicationFindings>(&serde_json::to_vec(&findings)?)?,
        findings
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_publication_findings_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/publication-findings.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "publication findings schema drifted from model authority");
    Ok(())
}

#[test]
fn publication_findings_reject_invalid_freshness_identity_and_severity() {
    let mut invalid_timestamp = findings();
    invalid_timestamp.generated_at_utc = "yesterday".to_string();
    assert!(validate_benchmark_publication_findings(&invalid_timestamp).is_err());

    let mut duplicate = findings();
    duplicate.findings.push(duplicate.findings[0].clone());
    assert!(validate_benchmark_publication_findings(&duplicate).is_err());

    let mut invalid_severity = findings();
    invalid_severity.findings[0].severity = "notice".to_string();
    assert!(validate_benchmark_publication_findings(&invalid_severity).is_err());
}

#[test]
fn checked_in_publication_findings_are_admitted() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("docs/30-operations/benchmark/corpus-01-publication-findings.json");
    let findings: BenchmarkPublicationFindings = serde_json::from_slice(&std::fs::read(path)?)?;
    validate_benchmark_publication_findings(&findings)?;
    Ok(())
}
