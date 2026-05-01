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
