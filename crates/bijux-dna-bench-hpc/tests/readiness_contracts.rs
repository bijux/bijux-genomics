use anyhow::{anyhow, Result};
use bijux_dna_bench_hpc::readiness::{validate_hpc_submission_ready, ReadinessContext};
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessReport;
use std::path::PathBuf;
#[path = "support/readiness.rs"]
mod evidence;
use evidence::{paths, UnavailableEvidence};

#[test]
fn unavailable_adapters_preserve_all_readiness_failures_and_publish_the_diagnostic() -> Result<()> {
    let root = tempfile::tempdir()?;
    let backend = UnavailableEvidence::default();
    let context = ReadinessContext { repo_root: root.path(), backend: &backend, paths: paths() };
    let output = PathBuf::from("reports/readiness.json");
    let error = validate_hpc_submission_ready(&context, output.clone())
        .err()
        .ok_or_else(|| anyhow!("expected unavailable provider failure"))?;
    assert!(error.to_string().contains("local HPC submission readiness failed"));
    let report: BenchmarkLocalHpcSubmissionReadinessReport =
        serde_json::from_slice(&std::fs::read(root.path().join(output))?)?;
    assert_eq!(report.checked_goal_count, 99);
    assert_eq!(report.failed_goal_count, 99);
    assert_eq!(report.passed_goal_count, 0);
    assert!(!report.ok);
    assert_eq!(report.failing_goal_ids, (1..=99).collect::<Vec<_>>());
    assert!(report.checks.iter().all(|check| !check.ok));
    assert_eq!(report.output_path, "reports/readiness.json");
    let calls = backend.calls.borrow();
    assert_eq!(calls.iter().filter(|method| **method == "materialize_local_stage").count(), 50);
    assert_eq!(calls.iter().filter(|method| **method == "simulate_dag_watchdog_path").count(), 4);
    assert_eq!(calls.iter().filter(|method| **method == "submit_mock_campaign").count(), 1);
    assert!(calls.contains(&"validate_slurm_dependencies"));
    Ok(())
}
#[test]
fn output_parent_failure_prevents_provider_execution_and_preserves_foreign_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let backend = UnavailableEvidence::default();
    std::fs::write(root.path().join("foreign"), b"retained")?;
    let context = ReadinessContext { repo_root: root.path(), backend: &backend, paths: paths() };
    assert!(
        validate_hpc_submission_ready(&context, PathBuf::from("foreign/readiness.json")).is_err()
    );
    assert!(backend.calls.borrow().is_empty());
    assert_eq!(std::fs::read(root.path().join("foreign"))?, b"retained");
    Ok(())
}
#[test]
fn profile_policy_round_trips_root_quotes_and_backslashes() -> Result<()> {
    let outer = tempfile::tempdir()?;
    let root = outer.path().join("quoted\"root\\path");
    std::fs::create_dir(&root)?;
    let backend = UnavailableEvidence::default();
    let context = ReadinessContext { repo_root: &root, backend: &backend, paths: paths() };
    assert!(validate_hpc_submission_ready(&context, PathBuf::from("readiness.json")).is_err());
    let support = root.join("artifacts/hpc/hpc-submission-ready");
    let policy: toml::Value =
        toml::from_str(&std::fs::read_to_string(support.join("layout.policy.toml"))?)?;
    for (key, child) in [
        ("corpora_root", "corpora"),
        ("encrypted_code_root", "code"),
        ("baselines_root", "baselines"),
    ] {
        assert_eq!(policy["layout"][key].as_str(), support.join(child).to_str());
    }
    assert!(backend.calls.borrow().contains(&"submit_mock_campaign"));
    Ok(())
}
