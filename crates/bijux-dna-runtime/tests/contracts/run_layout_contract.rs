use bijux_dna_runtime::run_layout::create_run_layout;

#[test]
fn run_layout_paths_match_contract() {
    let temp = bijux_dna_testkit::tempdir_for("run-layout");
    let temp_root = temp.path().to_path_buf();
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
