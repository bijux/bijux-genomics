use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::contract::RunRecordV1;
use bijux_dna_runtime::ReportSchemaV1;

fn fixture_root() -> PathBuf {
    crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("handshake")
        .join("default")
}

fn analyze_report_fixture() -> PathBuf {
    crate::support::crate_root("bijux-dna-analyze")
        .unwrap_or_else(|err| panic!("resolve analyze crate root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("pipelines")
        .join("fastq-to-fastq__default__v1")
        .join("report.json")
}

#[test]
fn benchmark_handshake_accepts_runtime_and_analyze_artifacts() -> Result<()> {
    let run_record_path = fixture_root().join("run_record.json");
    let report_path = analyze_report_fixture();

    let run_record: RunRecordV1 = serde_json::from_str(&fs::read_to_string(&run_record_path)?)?;
    let report: ReportSchemaV1 = serde_json::from_str(&fs::read_to_string(&report_path)?)?;

    assert!(
        !run_record.stages.is_empty(),
        "run record must contain stages"
    );
    assert!(
        !report.stages.is_empty(),
        "report must contain stage summaries"
    );
    let id_catalog: Vec<&str> = run_record
        .stages
        .iter()
        .map(|stage| stage.stage_id.as_str())
        .collect();
    assert!(
        report
            .stages
            .iter()
            .any(|stage| id_catalog.contains(&stage.stage_id.as_str())),
        "report stages must align with run record stages"
    );
    Ok(())
}
