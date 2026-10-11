use super::support::{Evidence, STAGES};
use anyhow::{Context, Result};
use bijux_dna_bench_bam_evidence::{
    command_coverage::{render_bam_command_adapter_coverage, BamReadinessGapKind},
    consistency::render_bam_active_row_consistency,
    output_contract::{render_bam_adapter_output_contract, BamAdapterOutputContractStatus},
    parser_coverage::render_bam_parser_fixture_coverage,
    report_map::render_bam_report_map,
};
use std::fs;
use std::path::Path;

#[test]
fn coverage_retains_gap_precedence_and_excludes_unready_parser_rows() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let mut rows = Evidence::serving_rows();
    rows[0].support_status = "planned".to_string();
    rows[0].adapter_status = "missing".to_string();
    rows[0].parser_status = "absent".to_string();
    rows[0].corpus_status = "asset".to_string();
    rows[1].adapter_status = "missing".to_string();
    rows[2].parser_status = "absent".to_string();
    rows[3].corpus_status = "asset".to_string();
    *evidence.serving_override.borrow_mut() = Some(rows);
    let report = render_bam_command_adapter_coverage(
        &evidence.context(root.path()),
        Path::new("reports/coverage.tsv"),
    )?;
    assert_eq!(report.rows[0].readiness_gap, BamReadinessGapKind::Support);
    assert_eq!(report.rows[1].readiness_gap, BamReadinessGapKind::Adapter);
    assert_eq!(report.rows[2].readiness_gap, BamReadinessGapKind::Parser);
    assert_eq!(report.rows[3].readiness_gap, BamReadinessGapKind::Corpus);
    assert_eq!(report.benchmark_ready_row_count, 20);
    let parser = render_bam_parser_fixture_coverage(
        &evidence.context(root.path()),
        Path::new("reports/parser.tsv"),
    )?;
    assert_eq!(parser.row_count, 20);
    assert_eq!(parser.excluded_non_benchmark_ready_row_count, 4);
    assert_eq!(parser.excluded_readiness_gap_counts.len(), 4);
    assert_eq!(parser.missing_row_count, 0);
    assert!(root.path().join("reports/coverage.tsv").is_file());
    Ok(())
}

#[test]
fn output_contract_preserves_planner_failure_as_published_diagnostic() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.fail_output.set(true);
    let report = render_bam_adapter_output_contract(
        &evidence.context(root.path()),
        Path::new("reports/output.tsv"),
    )?;
    assert_eq!(report.incomplete_adapter_row_count, 24);
    for row in report.rows {
        assert_eq!(row.output_contract_status, BamAdapterOutputContractStatus::Incomplete);
        assert!(row.reason.contains("captured planner rejection"));
        assert_eq!(row.missing_declarations, ["stage_contract"]);
        assert!(row.stdout_path_template.is_none());
    }
    assert!(fs::read_to_string(root.path().join("reports/output.tsv"))?
        .contains("captured planner rejection"));
    Ok(())
}

#[test]
fn missing_adapter_never_observes_planner_contract() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let mut rows = Evidence::serving_rows();
    for row in &mut rows {
        row.adapter_status = "missing".to_string();
    }
    *evidence.serving_override.borrow_mut() = Some(rows);
    let report = render_bam_adapter_output_contract(
        &evidence.context(root.path()),
        Path::new("outputs.tsv"),
    )?;
    assert_eq!(report.missing_adapter_row_count, 24);
    assert!(!evidence.calls.borrow().contains(&"assess_output"));
    Ok(())
}

#[test]
fn output_templates_use_explicit_run_identity_and_keep_declared_artifacts() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let report = render_bam_adapter_output_contract(
        &evidence.context(root.path()),
        Path::new("outputs.tsv"),
    )?;
    assert_eq!(report.complete_adapter_row_count, 24);
    assert_eq!(report.rows[0].stdout_path_template.as_deref(),Some("artifacts/runs/bench/slurm-dry-run/runs/admitted-run/{fixture_scope}/bam.align/{sample_scope}/bwa/stdout.log"));
    assert_eq!(report.rows[0].declared_output_ids, ["declared.metrics"]);
    Ok(())
}

#[test]
fn report_acquires_each_input_once_and_reobserves_on_next_request() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let first = render_bam_report_map(&evidence.context(root.path()), Path::new("map.tsv"))?;
    assert_eq!(first.expected_result_row_count, 24);
    assert_eq!(first.section_count, 7);
    assert_eq!(first.summary_table_count, 7);
    assert_eq!(first.rows.iter().map(|row| row.stage_id.as_str()).collect::<Vec<_>>(), STAGES);
    for name in ["stage_admissions", "stage_inventory", "expected_results"] {
        assert_eq!(
            evidence.calls.borrow().iter().filter(|call| **call == name).count(),
            1,
            "{name}"
        );
    }
    evidence.generation.set(1);
    let second = render_bam_report_map(&evidence.context(root.path()), Path::new("map.tsv"))?;
    assert_ne!(
        first.rows[0].scientific_context_required,
        second.rows[0].scientific_context_required
    );
    assert_eq!(
        evidence.calls.borrow().iter().filter(|call| **call == "expected_results").count(),
        2
    );
    Ok(())
}

#[test]
fn duplicate_result_identity_rejects_before_replacing_retained_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.duplicate_expected.set(true);
    let path = root.path().join("map.tsv");
    fs::write(&path, b"retained bytes")?;
    let error = render_bam_report_map(&evidence.context(root.path()), &path)
        .err()
        .context("invalid evidence must reject")?;
    assert!(error.to_string().contains("one row per expected BAM result row"));
    assert_eq!(fs::read(&path)?, b"retained bytes");
    Ok(())
}

#[test]
fn missing_domain_observation_rejects_before_expected_acquisition_or_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.missing_stage.set(true);
    let path = root.path().join("map.tsv");
    fs::write(&path, b"foreign bytes")?;
    let error = render_bam_report_map(&evidence.context(root.path()), &path)
        .err()
        .context("invalid evidence must reject")?;
    assert!(error.to_string().contains("missing domain stage observation"));
    assert!(!evidence.calls.borrow().contains(&"expected_results"));
    assert_eq!(fs::read(&path)?, b"foreign bytes");
    Ok(())
}

#[test]
fn consistency_uses_explicit_root_and_publishes_six_exact_surfaces() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    let context = evidence.context(root.path());
    let report =
        render_bam_active_row_consistency(&context, Path::new("reports/consistency.json"))?;
    assert!(report.ok);
    assert_eq!(report.checked_surface_count, 6);
    assert_eq!(report.passed_surface_count, 6);
    assert_eq!(report.active_row_count, 24);
    assert!(report.failed_surfaces.is_empty());
    assert_eq!(report.output_path, "reports/consistency.json");
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("reports/consistency.json"))?)?;
    assert_eq!(wire["ok"], true);
    assert_eq!(wire["surfaces"].as_array().context("serialized surface array")?.len(), 6);
    assert_eq!(
        evidence.calls.borrow().iter().filter(|call| **call == "publish_commands").count(),
        1
    );
    Ok(())
}

#[test]
fn binding_drift_publishes_missing_and_extra_identity_diagnostics() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.command_drift.set(true);
    let error = render_bam_active_row_consistency(
        &evidence.context(root.path()),
        Path::new("reports/drift.json"),
    )
    .err()
    .context("invalid evidence must reject")?;
    assert!(error.to_string().contains("count-consistent"));
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("reports/drift.json"))?)?;
    assert_eq!(wire["ok"], false);
    assert_eq!(wire["failed_surface_count"], 2);
    for surface in wire["failed_surfaces"].as_array().context("serialized surface array")? {
        assert_eq!(surface["missing_bindings"][0], "bam.align/bwa");
        assert_eq!(surface["extra_bindings"][0], "bam.align/unadmitted-tool");
    }
    Ok(())
}

#[test]
fn duplicate_active_rows_fail_even_when_all_surface_binding_sets_match() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.duplicate_active.set(true);
    assert!(render_bam_active_row_consistency(
        &evidence.context(root.path()),
        Path::new("reports/duplicate.json")
    )
    .is_err());
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(root.path().join("reports/duplicate.json"))?)?;
    assert_eq!(wire["active_row_count"], 25);
    assert_eq!(wire["passed_surface_count"], 6);
    assert_eq!(wire["ok"], false);
    Ok(())
}

#[test]
fn incomplete_planner_assessment_retains_declared_roles_and_missing_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence::default();
    evidence.missing_output.set(true);
    let report = render_bam_adapter_output_contract(
        &evidence.context(root.path()),
        Path::new("incomplete.tsv"),
    )?;
    assert_eq!(report.incomplete_adapter_row_count, 24);
    for row in report.rows {
        assert_eq!(row.output_contract_status, BamAdapterOutputContractStatus::Incomplete);
        assert_eq!(row.missing_declarations, ["execution_contract.raw"]);
        assert_eq!(row.declared_output_ids, ["declared.metrics"]);
        assert_eq!(row.normalized_metrics_output_id.as_deref(), Some("declared.metrics"));
        assert!(row.stdout_path_template.is_some());
        assert!(row.reason.contains("execution_contract.raw"));
    }
    Ok(())
}
