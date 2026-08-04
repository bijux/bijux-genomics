use anyhow::Result;

#[test]
fn checked_in_benchmark_contracts_load_from_rust_authority() -> Result<()> {
    let (registry, portfolio, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    assert_eq!(registry.schema_version, bijux_dna_bench::BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,);
    assert_eq!(portfolio.schema_version, bijux_dna_bench::BENCHMARK_PORTFOLIO_SCHEMA_V1,);
    assert_eq!(portfolio.stage_runs.len(), 71);
    assert!(
        registry.campaigns.len() >= 6,
        "the checked-in stage campaign registry must include the audited seed campaign set"
    );
    assert_eq!(interfaces.schema_version, bijux_dna_bench::BENCHMARK_STAGE_INTERFACES_SCHEMA_V1);
    assert_eq!(interfaces.interfaces.len(), 2);
    Ok(())
}

#[test]
fn checked_in_benchmark_contracts_keep_run_2042_reserved_and_assigned() -> Result<()> {
    let (registry, portfolio, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let run_2042 =
        registry.campaigns.iter().find(|campaign| campaign.run_id == "run-2042").ok_or_else(
            || anyhow::anyhow!("run-2042 is absent from the checked-in benchmark registry"),
        )?;
    assert_eq!(run_2042.stage_id, "fastq.profile_read_lengths");
    assert_eq!(
        portfolio.stage_runs.get("fastq.profile_read_lengths"),
        Some(&"run-2042".to_string())
    );
    assert_eq!(run_2042.profile_id, "fastq-observational-qc");
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == run_2042.stage_id)
        .ok_or_else(|| anyhow::anyhow!("run-2042 is absent from the stage interface catalog"))?;
    assert_eq!(interface.tool_ids, run_2042.tool_ids);
    Ok(())
}

#[test]
fn stage_interface_admission_uses_the_declared_repository_root() -> Result<()> {
    let path = bijux_dna_bench::bench_stage_interfaces_path();
    let repo_root = path
        .ancestors()
        .nth(3)
        .ok_or_else(|| anyhow::anyhow!("stage-interface path is outside a repository"))?;
    bijux_dna_bench::load_stage_interface_catalog_from_repo(&path, repo_root)?;

    let error = bijux_dna_bench::load_stage_interface_catalog_from_repo(
        &path,
        &repo_root.join("artifacts/nonexistent-repository-root"),
    )
    .err()
    .map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| { message.contains("validate stage interface repository paths") }));
    Ok(())
}

#[test]
fn checked_in_stage_reports_are_admitted_by_rust_contracts() -> Result<()> {
    let reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    assert_eq!(reports.len(), 6);
    assert!(reports.iter().any(|admission| admission.report.run_id == "run-2042"));
    assert!(reports.iter().all(|admission| {
        admission.schema_version == bijux_dna_bench::BENCHMARK_REPORT_ADMISSION_SCHEMA_V1
            && admission.source_sha256.len() == 64
            && admission.report_sha256.len() == 64
            && admission.report.schema_version == bijux_dna_bench::BENCHMARK_STAGE_REPORT_SCHEMA_V1
    }));
    Ok(())
}

#[test]
fn checked_in_stage_coverage_derives_from_catalog_and_admitted_reports() -> Result<()> {
    let (registry, _, _) = bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let repository_root = bijux_dna_bench::bench_reports_dir()
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?
        .to_path_buf();
    let catalog = bijux_dna_environment::api::load_container_catalog(&repository_root)?;
    let current_commit = bijux_dna_environment::api::repository_head_commit(&repository_root)?;

    let coverage =
        bijux_dna_bench::derive_stage_coverage(&registry, &catalog, &reports, &current_commit)?;

    assert_eq!(coverage.summary.retained_stage_count, 71);
    assert_eq!(coverage.summary.defined_stage_count, 6);
    assert_eq!(coverage.summary.admitted_stage_count, 0);
    assert_eq!(coverage.summary.stale_stage_count, 6);
    assert_eq!(coverage.summary.failed_stage_count, 0);
    assert_eq!(coverage.summary.missing_stage_count, 65);
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "fastq.profile_read_lengths"
            && row.campaign_run_id.as_deref() == Some("run-2042")
            && row.retained_tools == ["seqfu", "seqkit"]
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::Stale
    }));
    Ok(())
}

#[test]
fn checked_in_stage_coverage_publications_have_no_drift() -> Result<()> {
    let reports_dir = bijux_dna_bench::bench_reports_dir();
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    bijux_dna_bench::publish_checked_in_stage_coverage(
        repository_root,
        bijux_dna_bench::BenchmarkCoverageWriteMode::Check,
    )?;
    Ok(())
}

#[test]
fn stage_report_admission_rejects_incomplete_tool_closure() -> Result<()> {
    let mut reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut report =
        reports.pop().ok_or_else(|| anyhow::anyhow!("checked-in stage report set is empty"))?;
    report.report.tools.pop();

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report.report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("exact tool closure")));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_verdict_drift() -> Result<()> {
    let mut reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut report =
        reports.pop().ok_or_else(|| anyhow::anyhow!("checked-in stage report set is empty"))?;
    report.report.verdict = bijux_dna_bench::BenchmarkReportVerdict::Failed;

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report.report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("verdict disagrees")));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_abbreviated_identities() -> Result<()> {
    let mut reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut admission =
        reports.pop().ok_or_else(|| anyhow::anyhow!("checked-in stage report set is empty"))?;
    let report_sha256 = admission.report_sha256.clone();
    admission.report_sha256 = "1234567".to_string();
    let error = bijux_dna_bench_model::contract::validate_report_admission(&admission)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("invalid digest identity")));

    admission.report_sha256 = report_sha256;
    admission.report.limitations.push("mutated after admission".to_string());
    let error = bijux_dna_bench_model::contract::validate_report_admission(&admission)
        .err()
        .map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("does not match parsed report")));

    admission.report.limitations.pop();
    admission.report.tools[0].repository_commit = "1234567".to_string();
    let error = bijux_dna_bench_model::contract::validate_stage_report(&admission.report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("invalid commit identity")));
    Ok(())
}

#[test]
fn stage_report_admission_requires_data_locks() -> Result<()> {
    let reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut report = reports
        .iter()
        .find(|admission| admission.report.comparability.is_none())
        .ok_or_else(|| anyhow::anyhow!("checked-in reports contain no qualification campaign"))?
        .report
        .clone();
    for tool in &mut report.tools {
        tool.input_locks.clear();
    }

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report)
        .err()
        .map(|error| error.to_string());
    assert!(error
        .as_deref()
        .is_some_and(|message| message.contains("lacks comparative or per-tool input locks")));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_stale_catalog_identity() -> Result<()> {
    let (registry, _, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    let mut reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut report = reports
        .pop()
        .ok_or_else(|| anyhow::anyhow!("checked-in stage report set is empty"))?
        .report;
    report.catalog_sha256 = "f".repeat(64);

    let reports_dir = bijux_dna_bench::bench_reports_dir();
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    let artifact_root = repository_root.join("artifacts/tests/benchmark-report-admission");
    bijux_dna_infra::ensure_dir(&artifact_root)?;
    let publication = bijux_dna_infra::temp_dir_in(&artifact_root, "stale-catalog-")?;
    bijux_dna_infra::atomic_write_json(&publication.path().join("run-stale.json"), &report)?;

    let error =
        bijux_dna_bench::load_stage_reports_from_dir(publication.path(), &registry, &interfaces)
            .err()
            .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("catalog identity is stale")));
    Ok(())
}
