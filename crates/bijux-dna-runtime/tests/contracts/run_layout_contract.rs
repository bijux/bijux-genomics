use bijux_dna_runtime::run_layout::create_run_layout;

#[test]
fn run_layout_paths_match_contract() {
    let temp = bijux_dna_testkit::tempdir_for("run-layout");
    let temp_root = temp.path().to_path_buf();
    let (run_id, layout) = match create_run_layout(&temp_root) {
        Ok(value) => value,
        Err(err) => panic!("layout: {err}"),
    };
    assert!(run_id.starts_with("run-"), "run layout id must use the public run-* prefix");
    assert!(layout.run_dir.ends_with(&run_id), "run layout directory must end with run id");
    assert!(layout.manifests_dir.ends_with("manifests"));
    assert!(layout.logs_dir.ends_with("logs"));
    assert!(layout.reports_dir.ends_with("reports"));
    assert!(layout.assessment_path.ends_with("input_assessment.json"));
    assert!(layout.graph_path.ends_with("manifests/graph.json"));
    assert!(layout.plan_manifest_path.ends_with("manifests/plan_manifest.json"));
    assert!(layout.manifest_path.ends_with("run_manifest.json"));
    assert!(layout.environment_path.ends_with("environment.json"));
    assert!(layout.metadata_path.ends_with("run_metadata.json"));
    assert!(layout.events_path.ends_with("events.jsonl"));
    assert!(layout.run_state_path.ends_with("run_state.json"));
    assert!(layout.runtime_policy_path.ends_with("runtime_policy.json"));
    assert!(layout.executor_descriptor_path.ends_with("executor_descriptor.json"));
    assert!(layout.backend_descriptor_path.ends_with("backend_descriptor.json"));
    assert!(layout.scheduling_decision_path.ends_with("scheduling_decision.json"));
    assert!(layout.queue_state_path.ends_with("queue_state.json"));
    assert!(layout.lease_path.ends_with("run_lease.json"));
    assert!(layout.control_state_path.ends_with("run_control.json"));
    assert!(layout.health_report_path.ends_with("operator_health.json"));
    assert!(layout.slurm_submission_path.ends_with("slurm_submission.json"));
    assert!(layout.checkpoint_path.ends_with("checkpoints/checkpoint.json"));
    assert!(layout.failure_path.ends_with("run_failure.json"));
    assert!(layout.run_summary_path.ends_with("summary/run_summary.json"));
    assert!(layout.run_summary_text_path.ends_with("summary/run_summary.txt"));
    assert!(layout.artifact_inventory_path.ends_with("artifact_inventory.json"));
    assert!(layout.artifact_inventory_text_path.ends_with("artifact_inventory.txt"));
    assert!(layout.replay_manifest_path.ends_with("replay_manifest.json"));
    assert!(layout.hash_ledger_path.ends_with("hash_ledger.json"));
    assert!(layout.evidence_verification_path.ends_with("evidence_verification.json"));
    assert!(layout.evidence_bundle_path.ends_with("evidence_bundle.json"));
    assert!(layout.summary_dir.ends_with("summary"));
}
