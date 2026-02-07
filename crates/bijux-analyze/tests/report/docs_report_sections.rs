use std::fs;
use std::path::PathBuf;

#[test]
fn report_sections_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("REPORT_CONTRACT.md");
    let content = fs::read_to_string(&doc).expect("read REPORT_CONTRACT.md");

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
            "REPORT_CONTRACT.md missing report section {}",
            section
        );
    }
}
