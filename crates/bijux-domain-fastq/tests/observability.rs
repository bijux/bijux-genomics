use bijux_domain_fastq::{create_run_layout, InputAssessmentV1};

#[test]
fn input_assessment_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<InputAssessmentV1>(json).is_err());
}

#[test]
fn run_layout_paths_match_contract() {
    let temp_root = std::env::temp_dir().join("bijux-run-layout");
    let (_run_id, layout) = match create_run_layout(&temp_root) {
        Ok(value) => value,
        Err(err) => panic!("layout: {err}"),
    };
    assert!(layout.assessment_path.ends_with("input_assessment.json"));
    assert!(layout.manifest_path.ends_with("execution_manifest.json"));
    assert!(layout.environment_path.ends_with("environment.json"));
    assert!(layout.metadata_path.ends_with("run_metadata.json"));
    assert!(layout.events_path.ends_with("events.jsonl"));
    assert!(layout.summary_dir.ends_with("summary"));
}
