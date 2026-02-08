use std::fs;
use std::path::PathBuf;

#[test]
fn recording_truth_set_is_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("RECORDING_TRUTH_SET.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read RECORDING_TRUTH_SET.md: {err}"));

    for required in [
        "effective_config.json",
        "tool_invocation.json",
        "execution_record.json",
        "metrics.json",
        "stage_report.json",
    ] {
        assert!(
            content.contains(required),
            "doc missing required artifact {required}"
        );
    }

    assert!(
        content.contains("tests/recording/recording_completeness.rs"),
        "doc must reference recording/recording_completeness.rs"
    );
}
