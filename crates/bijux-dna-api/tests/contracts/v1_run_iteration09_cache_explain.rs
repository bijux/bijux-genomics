use anyhow::{anyhow, Result};
use bijux_dna_api::v1::api::run::{
    execute_local_fastq_workflow, explain_cache_hit_miss,
};

#[test]
fn cache_explain_reports_policy_and_environment_miss_reasons() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let original = execute_local_fastq_workflow(&temp.path().join("original"))?;
    let replay = execute_local_fastq_workflow(&temp.path().join("replay"))?;
    let replay_run_dir = replay
        .manifest_path
        .parent()
        .ok_or_else(|| anyhow!("replay manifest missing parent"))?;

    let policy_path = replay_run_dir.join("runtime_policy.json");
    let mut policy: serde_json::Value = serde_json::from_slice(&std::fs::read(&policy_path)?)?;
    policy["deterministic_scheduler"] = serde_json::Value::Bool(true);
    bijux_dna_infra::atomic_write_json(&policy_path, &policy)?;

    let environment_path = replay_run_dir.join("environment.json");
    let mut environment: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&environment_path)?)?;
    environment["runner"] = serde_json::Value::String("local-mutated".to_string());
    bijux_dna_infra::atomic_write_json(&environment_path, &environment)?;

    let explain = explain_cache_hit_miss(
        original
            .manifest_path
            .parent()
            .ok_or_else(|| anyhow!("original manifest missing parent"))?,
        replay_run_dir,
    )?;
    assert_eq!(explain.get("status").and_then(serde_json::Value::as_str), Some("miss"));
    let reasons = explain
        .get("unsafe_miss_reasons")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(reasons.iter().any(|entry| entry.get("reason_code") == Some(&serde_json::json!("policy_changed"))));
    assert!(reasons.iter().any(|entry| entry.get("reason_code") == Some(&serde_json::json!("environment_changed"))));
    Ok(())
}
