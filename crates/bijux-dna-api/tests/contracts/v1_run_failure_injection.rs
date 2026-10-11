use anyhow::Result;
use bijux_dna_api::v1::api::run::run_local_failure_injection;

#[test]
fn local_failure_injection_covers_all_runtime_failure_scenarios() -> Result<()> {
    let temp = tempfile::tempdir()?;
    for scenario in [
        "timeout",
        "cancel",
        "missing_output",
        "corrupt_output",
        "nonzero_exit",
        "interrupted_process",
        "partial_files",
    ] {
        let result = run_local_failure_injection(&temp.path().join(scenario), scenario)?;
        assert_eq!(result.get("scenario").and_then(serde_json::Value::as_str), Some(scenario));
        assert!(result.get("failure_path").is_some());
        assert!(result.get("failure").is_some());
        if scenario == "timeout" {
            assert_eq!(
                result
                    .get("failure")
                    .and_then(|failure| failure.get("failure_code"))
                    .and_then(serde_json::Value::as_str),
                Some("step_timeout_exceeded"),
                "timeout must remain distinct from an unclassified runner failure: {result}",
            );
        }
        if scenario == "cancel" {
            assert_eq!(
                result.get("state").and_then(serde_json::Value::as_str),
                Some("cancelled"),
                "cancel control must remain distinct from execution failure: {result}",
            );
            let run_dir = result
                .get("run_dir")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("cancel result must identify its run directory"))?;
            let queue: bijux_dna_runtime::run_layout::RunQueueStateV1 =
                serde_json::from_slice(&std::fs::read(format!("{run_dir}/queue_state.json"))?)?;
            assert_eq!(
                queue
                    .transitions
                    .iter()
                    .filter(|transition| {
                        transition.to_state
                            == bijux_dna_runtime::run_layout::RunQueueLifecycleStateV1::Cancelled
                    })
                    .count(),
                1,
                "cancel must be admitted through one immutable transition",
            );
            let control: bijux_dna_runtime::run_layout::RunControlStateV1 =
                serde_json::from_slice(&std::fs::read(format!("{run_dir}/run_control.json"))?)?;
            assert_eq!(control.audit_log.len(), 1, "cancel audit must not be overwritten");
        }
        assert!(
            matches!(
                result.get("state").and_then(serde_json::Value::as_str),
                Some("failed") | Some("cancelled")
            ),
            "scenario {scenario} should fail or cancel: {result}",
        );
    }
    Ok(())
}
