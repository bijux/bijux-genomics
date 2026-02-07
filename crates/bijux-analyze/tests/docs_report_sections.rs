use std::fs;
use std::path::PathBuf;

#[test]
fn report_sections_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("SCHEMA.md");
    let content = fs::read_to_string(&doc).expect("read SCHEMA.md");

    for section in [
        "qc",
        "final_qc_summary",
        "retention_definition",
        "retention_context",
        "filter_interpretation",
        "reproducibility",
        "method_assumptions",
        "metric_semantics",
        "completeness",
    ] {
        assert!(
            content.contains(section),
            "SCHEMA.md missing report section {}",
            section
        );
    }
}
