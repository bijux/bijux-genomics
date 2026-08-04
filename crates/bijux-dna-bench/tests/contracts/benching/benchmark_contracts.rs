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
    assert!(reports.iter().any(|report| report.run_id == "run-2042"));
    assert!(reports.iter().all(|report| {
        report.schema_version == bijux_dna_bench::BENCHMARK_STAGE_REPORT_SCHEMA_V1
    }));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_incomplete_tool_closure() -> Result<()> {
    let mut reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    let mut report =
        reports.pop().ok_or_else(|| anyhow::anyhow!("checked-in stage report set is empty"))?;
    report.tools.pop();

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report)
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
    report.verdict = bijux_dna_bench::BenchmarkReportVerdict::Failed;

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("verdict disagrees")));
    Ok(())
}
