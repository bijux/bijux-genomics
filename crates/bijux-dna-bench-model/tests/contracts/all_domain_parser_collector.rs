use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    all_domain_parser_collector_schema, validate_all_domain_parser_collector,
    AllDomainParserCollectorReport, AllDomainParserCollectorRow,
    AllDomainParserCollectorSourceKind, ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT,
    ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT, ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH,
    ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1,
};

fn row(
    record_id: &str,
    source_kind: AllDomainParserCollectorSourceKind,
    domain: &str,
    stage_id: &str,
    result_id: Option<&str>,
) -> AllDomainParserCollectorRow {
    let tool_id = match stage_id {
        "bam.validate" => "samtools",
        "fastq.validate_reads" => "fastqc",
        "vcf.prepare_reference_panel" | "vcf.stats" => "bcftools",
        _ => "governed_tool",
    };
    AllDomainParserCollectorRow {
        record_id: record_id.to_string(),
        source_kind,
        document_kind: "governed_metrics".to_string(),
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: "governed_corpus".to_string(),
        asset_profile_id: "governed_profile".to_string(),
        result_id: result_id.map(str::to_string),
        parsed_path: format!("artifacts/benchmark-smoke/{record_id}.json"),
        parsed_schema_version: "bijux.test.metrics.v1".to_string(),
        parsed_top_level_key_count: 2,
        parsed_top_level_keys: vec!["metric".to_string(), "schema_version".to_string()],
        manifest_path: None,
        manifest_status: None,
        manifest_exit_code: None,
        declared_output_count: 1,
        normalized_snapshot: BTreeMap::from([("metric".to_string(), serde_json::json!(1))]),
    }
}

fn sample_report() -> AllDomainParserCollectorReport {
    let rows = vec![
        row(
            "fake-run:fastq.validate_reads:fastqc",
            AllDomainParserCollectorSourceKind::FakeRun,
            "fastq",
            "fastq.validate_reads",
            Some("fastq.validate_reads:fastqc"),
        ),
        row(
            "real-smoke:bam.validate",
            AllDomainParserCollectorSourceKind::RealSmoke,
            "bam",
            "bam.validate",
            None,
        ),
        row(
            "real-smoke:fastq.validate_reads",
            AllDomainParserCollectorSourceKind::RealSmoke,
            "fastq",
            "fastq.validate_reads",
            None,
        ),
        row(
            "real-smoke:vcf.prepare_reference_panel",
            AllDomainParserCollectorSourceKind::RealSmoke,
            "vcf",
            "vcf.prepare_reference_panel",
            None,
        ),
        row(
            "real-smoke:vcf.stats",
            AllDomainParserCollectorSourceKind::RealSmoke,
            "vcf",
            "vcf.stats",
            None,
        ),
    ];
    AllDomainParserCollectorReport {
        schema_version: ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1.to_string(),
        output_path: ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH.to_string(),
        fixture_root: ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT.to_string(),
        fake_run_root: ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT.to_string(),
        row_count: rows.len(),
        fake_run_row_count: 1,
        real_smoke_row_count: 4,
        source_kind_counts: BTreeMap::from([
            ("fake_run".to_string(), 1),
            ("real_smoke".to_string(), 4),
        ]),
        domain_counts: BTreeMap::from([
            ("bam".to_string(), 1),
            ("fastq".to_string(), 2),
            ("vcf".to_string(), 2),
        ]),
        document_kind_counts: BTreeMap::from([("governed_metrics".to_string(), 5)]),
        rows,
    }
}

#[test]
fn parser_collector_accepts_complete_aggregate_closure() -> anyhow::Result<()> {
    let report = sample_report();
    validate_all_domain_parser_collector(&report)?;
    assert_eq!(
        serde_json::from_slice::<AllDomainParserCollectorReport>(&serde_json::to_vec(&report)?)?,
        report
    );
    Ok(())
}

#[test]
fn parser_collector_rejects_missing_reference_panel_smoke() {
    let mut report = sample_report();
    report.rows.retain(|row| row.record_id != "real-smoke:vcf.prepare_reference_panel");
    report.row_count -= 1;
    report.real_smoke_row_count -= 1;
    report.source_kind_counts.insert("real_smoke".to_string(), 3);
    report.domain_counts.insert("vcf".to_string(), 1);
    report.document_kind_counts.insert("governed_metrics".to_string(), 4);
    assert!(validate_all_domain_parser_collector(&report).is_err());
}

#[test]
fn parser_collector_rejects_aggregate_and_identity_drift() {
    let mut report = sample_report();
    report.fake_run_row_count = 2;
    assert!(validate_all_domain_parser_collector(&report).is_err());

    let mut report = sample_report();
    report.rows[0].parsed_top_level_keys.reverse();
    assert!(validate_all_domain_parser_collector(&report).is_err());

    let mut report = sample_report();
    report.output_path = "artifacts/benchmark-smoke/readiness/renamed.json".to_string();
    assert!(validate_all_domain_parser_collector(&report).is_err());

    let mut report = sample_report();
    report.rows[1].tool_id = "wrong-tool".to_string();
    assert!(validate_all_domain_parser_collector(&report).is_err());
}

#[test]
fn parser_collector_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/all-domain-parser-collector.schema.json");
    let generated = serde_json::to_value(all_domain_parser_collector_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "all-domain parser-collector schema drifted");
    Ok(())
}

#[test]
fn parser_collector_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(sample_report())?;
    value["rows"][0]["legacy_parser"] = serde_json::json!(true);
    assert!(serde_json::from_value::<AllDomainParserCollectorReport>(value).is_err());
    Ok(())
}
