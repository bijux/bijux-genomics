use anyhow::Result;

#[test]
fn checked_in_benchmark_contracts_load_from_rust_authority() -> Result<()> {
    let (registry, portfolio, interfaces) =
        bijux_dna_bench::load_checked_in_benchmark_planning_contracts()?;
    assert_eq!(registry.schema_version, bijux_dna_bench::BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,);
    assert_eq!(portfolio.schema_version, bijux_dna_bench::BENCHMARK_PORTFOLIO_SCHEMA_V2,);
    assert_eq!(portfolio.stage_runs.len(), 71);
    assert!(
        registry.campaigns.len() >= 7,
        "the checked-in stage campaign registry must include the audited seed campaign set"
    );
    assert_eq!(interfaces.schema_version, bijux_dna_bench::BENCHMARK_STAGE_INTERFACES_SCHEMA_V1);
    assert_eq!(interfaces.interfaces.len(), 12);
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
fn checked_in_stage_reports_exclude_historical_evidence() -> Result<()> {
    let reports = bijux_dna_bench::load_checked_in_stage_reports()?;
    assert!(reports.is_empty());

    let repository_root = repository_root()?;
    let registry = bijux_dna_bench::derive_benchmark_publication_registry(&repository_root)?;
    assert_eq!(registry.summary.active_stage_report_count, 0);
    assert_eq!(registry.summary.historical_stage_report_count, 7);
    assert!(registry.stage_reports.iter().all(|report| {
        report.disposition == bijux_dna_bench::BenchmarkStagePublicationDisposition::Historical
            && report.json_path.contains("/historical/")
            && report.json_path.contains(&report.source_commit)
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
    let catalog = bijux_dna_containers::api::load_container_catalog(&repository_root)?;
    let current_commit = bijux_dna_containers::api::repository_head_commit(&repository_root)?;

    let coverage =
        bijux_dna_bench::derive_stage_coverage(&registry, &catalog, &reports, &current_commit)?;

    assert_eq!(coverage.summary.retained_stage_count, 71);
    assert_eq!(coverage.summary.defined_stage_count, 12);
    assert_eq!(coverage.summary.failed_stage_count, 0);
    assert!(
        coverage.summary.admitted_stage_count
            + coverage.summary.stale_stage_count
            + coverage.summary.failed_stage_count
            <= coverage.summary.defined_stage_count
    );
    assert_eq!(
        coverage.summary.defined_stage_count + coverage.summary.missing_stage_count,
        coverage.summary.retained_stage_count
    );
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "bam.genotyping"
            && row.campaign_run_id.as_deref() == Some("run-2001")
            && row.retained_tools == ["bcftools"]
            && row.report_repository_commit.is_none()
            && row.report_sha256.is_none()
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::NotAdmitted
    }));
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "bam.coverage"
            && row.campaign_run_id.as_deref() == Some("run-2002")
            && row.retained_tools == ["bedtools", "mosdepth", "samtools"]
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::NotAdmitted
    }));
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "bam.endogenous_content"
            && row.campaign_run_id.as_deref() == Some("run-2003")
            && row.retained_tools == ["samtools"]
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::NotAdmitted
    }));
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "bam.mapping_summary"
            && row.campaign_run_id.as_deref() == Some("run-2004")
            && row.retained_tools == ["picard", "samtools"]
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::NotAdmitted
    }));
    assert!(coverage.stages.iter().any(|row| {
        row.stage_id == "fastq.profile_read_lengths"
            && row.campaign_run_id.as_deref() == Some("run-2042")
            && row.retained_tools == ["seqfu", "seqkit"]
            && row.execution_status == bijux_dna_bench::BenchmarkStageExecutionStatus::NotAdmitted
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
fn checked_in_stage_universe_closes_every_domain_stage() -> Result<()> {
    let reports_dir = bijux_dna_bench::bench_reports_dir();
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    let portfolio = bijux_dna_bench::load_benchmark_portfolio(
        &repository_root.join("benchmarks/contracts/portfolio.toml"),
    )?;
    let universe = bijux_dna_bench::derive_stage_universe(repository_root, &portfolio)?;

    assert_eq!(universe.summary.stage_count, 89);
    assert_eq!(universe.summary.benchmark_required_count, 71);
    assert_eq!(universe.summary.qualification_only_count, 9);
    assert_eq!(universe.summary.orchestration_only_count, 9);
    assert_eq!(universe.summary.test_only_count, 0);
    assert_eq!(universe.summary.retired_count, 0);
    assert_eq!(universe.summary.excluded_count, 0);
    assert!(universe.stages.iter().all(|stage| {
        !stage.benchmark_disposition_reason.trim().is_empty()
            && (stage.benchmark_disposition
                == bijux_dna_bench::BenchmarkDisposition::BenchmarkRequired)
                == stage.reserved_run_id.is_some()
    }));
    Ok(())
}

#[test]
fn checked_in_stage_universe_publications_have_no_drift() -> Result<()> {
    let reports_dir = bijux_dna_bench::bench_reports_dir();
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    bijux_dna_bench::publish_checked_in_stage_universe(
        repository_root,
        bijux_dna_bench::BenchmarkUniverseWriteMode::Check,
    )?;
    Ok(())
}

#[test]
fn checked_in_run_registry_closes_the_permanent_namespace() -> Result<()> {
    let reports_dir = bijux_dna_bench::bench_reports_dir();
    let repository_root = reports_dir
        .ancestors()
        .find(|root| root.join(".git").exists())
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))?;
    let registry = bijux_dna_bench::publish_checked_in_benchmark_run_registry(
        repository_root,
        bijux_dna_bench::BenchmarkRunRegistryWriteMode::Check,
    )?;

    assert_eq!(registry.summary.run_count, 71);
    assert_eq!(registry.summary.active_count, 12);
    assert_eq!(registry.summary.reserved_count, 59);
    assert_eq!(registry.namespace.next_run_number, 2072);
    assert_eq!(registry.runs.first().map(|row| row.run_id.as_str()), Some("run-2001"));
    assert_eq!(registry.runs.last().map(|row| row.run_id.as_str()), Some("run-2071"));
    assert!(registry.runs.windows(2).all(|rows| rows[0].run_id < rows[1].run_id));
    assert!(registry.runs.iter().all(|row| {
        !row.tool_ids.is_empty()
            && !row.required_corpus_roles.is_empty()
            && row.introduced_at_commit.len() == 40
    }));
    assert!(registry
        .runs
        .iter()
        .filter(|row| row.state == bijux_dna_bench::BenchmarkRunRegistryState::Active)
        .all(|row| {
            !row.accepted_input_contracts.is_empty()
                && row.output_contract.is_some()
                && row.normalized_result_contract.is_some()
        }));
    Ok(())
}

#[test]
fn checked_in_publication_registry_has_no_drift() -> Result<()> {
    let repository_root = repository_root()?;
    let registry = bijux_dna_bench::publish_checked_in_benchmark_publication_registry(
        &repository_root,
        bijux_dna_bench::BenchmarkPublicationRegistryWriteMode::Check,
    )?;
    assert_eq!(registry.summary.current_product_count, 4);
    assert_eq!(registry.summary.active_stage_report_count, 0);
    assert_eq!(registry.summary.historical_stage_report_count, 7);
    assert_eq!(registry.summary.quarantined_surface_count, 0);
    assert_eq!(registry.summary.quarantined_file_count, 0);
    assert!(registry.quarantined_surfaces.is_empty());
    Ok(())
}

#[test]
fn stage_report_admission_rejects_incomplete_tool_closure() -> Result<()> {
    let mut report = historical_stage_report()?;
    report.report.tools.pop();

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report.report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("exact tool closure")));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_verdict_drift() -> Result<()> {
    let mut report = historical_stage_report()?;
    report.report.verdict = bijux_dna_bench::BenchmarkReportVerdict::Failed;

    let error = bijux_dna_bench_model::contract::validate_stage_report(&report.report)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("verdict disagrees")));
    Ok(())
}

#[test]
fn stage_report_admission_rejects_abbreviated_identities() -> Result<()> {
    let mut admission = historical_stage_report()?;
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
    let mut report = historical_stage_report()?.report;
    report.comparability = None;
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
    let mut report = historical_stage_report()?.report;
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

fn repository_root() -> Result<std::path::PathBuf> {
    bijux_dna_bench::bench_reports_dir()
        .ancestors()
        .find(|root| root.join(".git").exists())
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root"))
}

fn historical_stage_report() -> Result<bijux_dna_bench::AdmittedBenchmarkStageReport> {
    bijux_dna_bench::load_stage_report(
        &repository_root()?.join(
            "benchmarks/reports/historical/520efb95cf753b65fca168f0610b1b4138932031/run-2042-fastq-profile-read-lengths.json",
        ),
    )
}
