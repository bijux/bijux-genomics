use std::path::PathBuf;

#[test]
fn vcf_toy_pipeline_fixture_bundle_exists() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("pipelines")
        .join("vcf-to-vcf__minimal__v1");
    for required in [
        "CASE.json",
        "defaults_ledger.json",
        "facts.jsonl",
        "report.json",
    ] {
        let path = base.join(required);
        assert!(path.exists(), "missing fixture file {}", path.display());
    }
}
