use anyhow::{Context, Result};
use bijux_dna_bench_hpc::dry_run::{render_hpc_dry_run_ready, validate_hpc_dry_run_ready_path};
use bijux_dna_bench_model::{
    validate_benchmark_local_hpc_dry_run_readiness, BenchmarkLocalHpcDryRunReadinessReport,
};
use std::path::Path;
#[path = "support/dry_run.rs"]
mod evidence;
use evidence::{context, Evidence};

#[test]
fn provider_failures_publish_every_diagnostic_once() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let context = context(root.path(), &evidence);
    let error = render_hpc_dry_run_ready(&context, "reports/readiness.json".into())
        .err()
        .context("unavailable evidence must fail readiness")?;
    assert!(error.to_string().contains("local HPC dry-run readiness failed"));
    let report: BenchmarkLocalHpcDryRunReadinessReport =
        serde_json::from_slice(&std::fs::read(root.path().join("reports/readiness.json"))?)?;
    validate_benchmark_local_hpc_dry_run_readiness(&report)?;
    assert_eq!(report.failing_goal_ids, (481..=489).collect::<Vec<_>>());
    assert_eq!(report.checked_goal_count, 9);
    assert_eq!(report.passed_goal_count, 0);
    assert_eq!(report.failed_goal_count, 9);
    assert!(!report.ready_for_first_hpc_run);
    assert!(!report.behavior.proven);
    assert_eq!(report.output_path, "reports/readiness.json");
    assert!(report.checks.iter().all(|check| check.detail.contains("unavailable at generation 0")));
    assert_eq!(
        *evidence.calls.borrow(),
        vec![
            "asset",
            "scratch",
            "execution",
            "stage_benchmark",
            "pipeline_node",
            "dependency",
            "resume",
            "result_collection",
            "candidate"
        ]
    );
    Ok(())
}
#[test]
fn output_refusal_precedes_all_provider_and_filesystem_effects() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let context = context(root.path(), &evidence);
    let output = root.path().join("foreign.json");
    std::fs::write(&output, b"retained")?;
    for result in [
        render_hpc_dry_run_ready(&context, output.clone()),
        validate_hpc_dry_run_ready_path(&context, &output),
    ] {
        assert!(result
            .err()
            .context("outside output must reject")?
            .to_string()
            .contains("outside admitted report root"));
    }
    assert!(evidence.calls.borrow().is_empty());
    assert_eq!(std::fs::read(output)?, b"retained");
    assert_eq!(std::fs::read_dir(root.path())?.count(), 1);
    Ok(())
}
#[test]
fn malformed_or_semantically_inconsistent_reports_reject_before_acquisition() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let context = context(root.path(), &evidence);
    let path = root.path().join("reports/readiness.json");
    std::fs::create_dir(path.parent().context("report parent")?)?;
    std::fs::write(&path, b"null")?;
    let error = validate_hpc_dry_run_ready_path(&context, &path)
        .err()
        .context("null report must reject")?;
    assert!(format!("{error:#}").contains("parse HPC dry-run readiness report"));
    assert!(evidence.calls.borrow().is_empty());
    assert_eq!(std::fs::read(&path)?, b"null");
    assert!(render_hpc_dry_run_ready(&context, path.clone()).is_err());
    evidence.calls.borrow_mut().clear();
    let mut report: BenchmarkLocalHpcDryRunReadinessReport =
        serde_json::from_slice(&std::fs::read(&path)?)?;
    report.ready_for_first_hpc_run = true;
    let bytes = serde_json::to_vec_pretty(&report)?;
    std::fs::write(&path, &bytes)?;
    let error = validate_hpc_dry_run_ready_path(&context, &path)
        .err()
        .context("contradictory readiness must reject")?;
    assert!(format!("{error:#}").contains("drifted from its declared observations"));
    assert!(evidence.calls.borrow().is_empty());
    assert_eq!(std::fs::read(path)?, bytes);
    Ok(())
}
#[test]
fn replay_reacquires_evidence_and_detects_changed_provider_observations() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let context = context(root.path(), &evidence);
    let path = Path::new("reports/readiness.json");
    assert!(render_hpc_dry_run_ready(&context, path.into()).is_err());
    let retained = std::fs::read(root.path().join(path))?;
    evidence.calls.borrow_mut().clear();
    evidence.generation.set(1);
    let error = validate_hpc_dry_run_ready_path(&context, path)
        .err()
        .context("changed observations must reject")?;
    assert!(format!("{error:#}").contains("drifted from governed dry-run inputs"));
    assert_eq!(evidence.calls.borrow().len(), 9);
    assert_eq!(std::fs::read(root.path().join(path))?, retained);
    Ok(())
}
