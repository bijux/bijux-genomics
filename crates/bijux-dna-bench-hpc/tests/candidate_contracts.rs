use anyhow::{Context, Result};
use bijux_dna_bench_hpc::candidate::{
    collect_hpc_candidate_run_manifest, render_hpc_candidate_run_manifest,
    validate_hpc_candidate_run_manifest_path,
};
use std::path::Path;
#[path = "support/candidate.rs"]
mod evidence;
use evidence::{context, evidence, input};

#[test]
fn selection_keeps_small_representatives_and_observes_each_result_path_once() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = evidence();
    let context = context(root.path(), &evidence);
    let path = Path::new("reports/candidate.json");
    let report = render_hpc_candidate_run_manifest(&context, path.into())?;
    assert_eq!(
        report.rows.iter().map(|row| row.result_id.as_str()).collect::<Vec<_>>(),
        vec!["bam-single", "fastq-small"]
    );
    assert_eq!(report.eligible_small_job_count, 3);
    assert_eq!(report.selected_job_count, 2);
    assert_eq!(report.selected_staged_input_bytes, 5);
    assert_eq!(
        report.exclusion_reason_counts.get("superseded_by_smaller_representative"),
        Some(&1)
    );
    assert_eq!(*evidence.path_calls.borrow(), vec!["bam-single", "fastq-large", "fastq-small"]);
    for row in &report.rows {
        assert_eq!(
            row.stop_conditions
                .iter()
                .map(|condition| condition.condition_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "missing_stage_result_manifest",
                "failed_stage_result_manifest",
                "stale_partial_outputs",
                "unexpected_collection_status"
            ]
        );
        assert!(row.stop_conditions[0].detail.contains(&row.expected_stage_result_manifest_path));
    }
    assert_eq!(validate_hpc_candidate_run_manifest_path(&context, path)?, report);
    assert_eq!(evidence.path_calls.borrow().len(), 6);
    Ok(())
}
#[test]
fn output_refusal_precedes_all_acquisition_and_preserves_foreign_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = evidence();
    let path = root.path().join("foreign.json");
    std::fs::write(&path, b"retained")?;
    let error = render_hpc_candidate_run_manifest(&context(root.path(), &evidence), path.clone())
        .err()
        .context("outside output must reject")?;
    assert!(error.to_string().contains("outside admitted report root"));
    assert!(evidence.calls.borrow().is_empty());
    assert!(evidence.path_calls.borrow().is_empty());
    assert_eq!(std::fs::read(path)?, b"retained");
    Ok(())
}
#[test]
fn malformed_provider_contracts_reject_before_result_path_observation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut evidence = evidence();
    evidence.assets.jobs[0].staged_inputs[0].checksum_sha256 = "bad".into();
    assert!(collect_hpc_candidate_run_manifest(
        &context(root.path(), &evidence),
        Path::new("reports/candidate.json")
    )
    .is_err());
    assert!(evidence.path_calls.borrow().is_empty());
    assert!(!root.path().join("reports").exists());
    Ok(())
}
#[test]
fn invalid_resource_units_and_overflow_preserve_the_retained_manifest() -> Result<()> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("reports/candidate.json");
    std::fs::create_dir(path.parent().context("report parent")?)?;
    std::fs::write(&path, b"retained")?;
    for time in ["4294967295:00:00", "00:60:00", "00:00:01"] {
        let mut evidence = evidence();
        for hint in evidence.resources.values_mut() {
            hint.time_limit = time.into();
        }
        let error =
            render_hpc_candidate_run_manifest(&context(root.path(), &evidence), path.clone())
                .err()
                .context("invalid duration must reject")?;
        assert!(format!("{error:#}").contains("time limit"));
        assert!(evidence.path_calls.borrow().is_empty());
        assert_eq!(std::fs::read(&path)?, b"retained");
    }
    Ok(())
}
#[test]
fn overflowing_staged_footprint_rejects_before_result_observation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut evidence = evidence();
    evidence.assets.jobs[0].staged_inputs[0].size_bytes = u64::MAX;
    let mut extra = input("z-extra", 1);
    extra.consumer_result_id = "bam-single".into();
    evidence.assets.jobs[0].staged_inputs.push(extra);
    evidence.assets.staged_input_count += 1;
    evidence.assets.unique_source_path_count += 1;
    let error = render_hpc_candidate_run_manifest(
        &context(root.path(), &evidence),
        "reports/candidate.json".into(),
    )
    .err()
    .context("overflow must reject")?;
    assert!(format!("{error:#}").contains("footprint exceeds the byte representation"));
    assert!(evidence.path_calls.borrow().is_empty());
    assert!(!root.path().join("reports").exists());
    Ok(())
}
#[test]
fn result_observation_failure_is_fallible_and_does_not_publish() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut evidence = evidence();
    evidence.reject_paths = true;
    let error = render_hpc_candidate_run_manifest(
        &context(root.path(), &evidence),
        "reports/candidate.json".into(),
    )
    .err()
    .context("bad result paths must reject")?;
    assert!(format!("{error:#}").contains("inconsistent result destinations"));
    assert_eq!(evidence.path_calls.borrow().len(), 1);
    assert!(!root.path().join("reports").exists());
    Ok(())
}
#[test]
fn replay_reacquires_changed_evidence_and_keeps_the_retained_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut evidence = evidence();
    let path = Path::new("reports/candidate.json");
    render_hpc_candidate_run_manifest(&context(root.path(), &evidence), path.into())?;
    let retained = std::fs::read(root.path().join(path))?;
    evidence.assets.jobs[2].staged_inputs[0].size_bytes = 30;
    let error = validate_hpc_candidate_run_manifest_path(&context(root.path(), &evidence), path)
        .err()
        .context("changed selection must reject")?;
    assert!(format!("{error:#}").contains("drifted from governed dry-run inputs"));
    assert_eq!(std::fs::read(root.path().join(path))?, retained);
    assert_eq!(evidence.path_calls.borrow().len(), 6);
    Ok(())
}
