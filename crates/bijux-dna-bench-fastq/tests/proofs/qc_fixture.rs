use anyhow::Result;
use bijux_dna_bench_fastq::proofs::qc_fixture::{materialize_qc_fixture, LocalReportQcConfig};
use std::{
    fs,
    path::{Path, PathBuf},
};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn config() -> LocalReportQcConfig {
    let fixtures = root().join("assets/toy/core-v1/fastq/report_qc");
    LocalReportQcConfig {
        schema_version: "bijux.bench.fastq.local_report_qc.v1".to_string(),
        tool_id: "multiqc".to_string(),
        fixture_root: fixtures.clone(),
        report_template: fixtures.join("report_qc_report_template.json"),
        manifest_template: fixtures.join("governed_qc_inputs_manifest.json"),
        multiqc_report: fixtures.join("multiqc_report.html"),
        multiqc_data_dir: fixtures.join("multiqc_data"),
        aggregation_engine: Some("multiqc".to_string()),
        aggregation_scope: Some("governed_qc_artifacts".to_string()),
        output_dir: None,
    }
}
#[test]
fn invalid_qc_settings_and_templates_refuse_before_output_creation() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let out = fs::canonicalize(dir.path())?.join("fresh");
    let mut invalid = config();
    invalid.aggregation_engine = Some("unknown".to_string());
    assert!(materialize_qc_fixture(&root(), Path::new("config.toml"), invalid, &out).is_err());
    assert!(!out.exists());
    let mut missing = config();
    missing.report_template = dir.path().join("missing.json");
    assert!(materialize_qc_fixture(&root(), Path::new("config.toml"), missing, &out).is_err());
    assert!(!out.exists());
    Ok(())
}
#[test]
fn qc_bundle_preserves_foreign_files_and_rewrites_governed_reports() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let out = fs::canonicalize(dir.path())?.join("bundle");
    fs::create_dir(&out)?;
    fs::write(out.join("retained-native-evidence"), b"original")?;
    let path = materialize_qc_fixture(&root(), Path::new("config.toml"), config(), &out)?;
    let report: bijux_dna_domain_fastq::ReportQcReportV1 =
        serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(report.tool_id, "multiqc");
    assert!(report.governed_qc_input_count > 0);
    assert_eq!(fs::read(out.join("retained-native-evidence"))?, b"original");
    let manifest: bijux_dna_domain_fastq::GovernedQcInputsManifestV1 =
        serde_json::from_slice(&fs::read(out.join("governed_qc_inputs_manifest.json"))?)?;
    assert_eq!(report.governed_qc_lineage_hash, manifest.lineage_hash);
    Ok(())
}
