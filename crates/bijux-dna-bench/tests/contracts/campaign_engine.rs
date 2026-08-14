use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use bijux_dna_bench::{
    aggregate_benchmark_admitted_case_observation_set, aggregate_benchmark_case_observations,
    build_benchmark_admitted_case_observation_set,
    build_benchmark_stage_report_from_case_aggregations,
    ensure_benchmark_admitted_case_observation_set, ensure_benchmark_case_aggregation_set,
    ensure_benchmark_case_matrix, load_benchmark_admitted_case_observation_set,
    load_benchmark_case_aggregation_set, load_benchmark_case_matrix,
    load_checked_in_benchmark_campaign_plan, load_checked_in_benchmark_planning_contracts,
    plan_benchmark_case_attempts, plan_benchmark_case_matrix, plan_benchmark_case_retry_attempts,
    validate_benchmark_admitted_case_observation_set, validate_benchmark_case_aggregation_set,
    validate_benchmark_case_aggregations, validate_benchmark_case_matrix,
    verify_campaign_plan_source, verify_case_campaign_plan_source,
    BenchmarkAdmittedCaseObservationV2, BenchmarkAttemptTransition,
    BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixRequest,
    BenchmarkCaseRepetitionObservationV1, BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1,
    BenchmarkLifecycleStatus, BenchmarkReportImage, BenchmarkReportInputLock,
    BenchmarkReportSlurmJob, BenchmarkScientificValue, BenchmarkStageCampaignRegistry,
    BenchmarkStageInterfaceCatalog, BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
};
use bijux_dna_bench_model::append_attempt_transition;
use bijux_dna_core::{
    contract::ToolConstraints,
    ids::{StageId, ToolId},
};
use bijux_dna_stage_contract::{
    canonical_tool_bindings, BindingInputV1, BindingOutputV1, BindingParameterV1,
    OutputDiscoveryV1, ToolArtifactSourceV1, ToolBindingRegistry, ToolBindingV1,
    TOOL_BINDING_SCHEMA_V1,
};

const EXISTING_CAMPAIGN_COUNT: usize = 12;
const EXISTING_CASE_COUNT: usize = 24;
const COVERAGE_STAGE: &str = "bam.coverage";

struct PlanningFixture {
    campaigns: BenchmarkStageCampaignRegistry,
    interfaces: BenchmarkStageInterfaceCatalog,
    bindings: ToolBindingRegistry,
    snapshots: Vec<BenchmarkDataSnapshotV1>,
}

#[test]
fn checked_in_campaigns_plan_through_one_order_independent_engine() -> Result<()> {
    let fixture = planning_fixture()?;
    assert_eq!(fixture.campaigns.campaigns.len(), EXISTING_CAMPAIGN_COUNT);

    let matrix = plan(&fixture)?;
    assert_eq!(matrix.cases.len(), EXISTING_CASE_COUNT);
    assert_eq!(
        matrix.cases.iter().map(|case| case.run_id.as_str()).collect::<BTreeSet<_>>().len(),
        EXISTING_CAMPAIGN_COUNT
    );

    let mut reordered = fixture;
    reordered.campaigns.campaigns.reverse();
    reordered.interfaces.interfaces.reverse();
    reordered.snapshots.reverse();
    for campaign in &mut reordered.campaigns.campaigns {
        campaign.corpus_catalog_ids.reverse();
    }
    let reordered_bytes = serde_json::to_vec(&plan(&reordered)?)?;
    assert!(
        serde_json::to_vec(&matrix)? == reordered_bytes,
        "case matrix bytes changed with semantically unordered authority input"
    );
    Ok(())
}

#[test]
fn checked_in_campaigns_close_the_canonical_binding_registry() -> Result<()> {
    let mut fixture = planning_fixture()?;
    fixture.bindings = canonical_tool_bindings()?;

    let matrix = plan(&fixture)?;

    assert_eq!(matrix.cases.len(), EXISTING_CASE_COUNT);
    assert_eq!(
        matrix.cases.iter().map(|case| case.run_id.as_str()).collect::<BTreeSet<_>>().len(),
        EXISTING_CAMPAIGN_COUNT
    );
    Ok(())
}

#[test]
fn generic_cases_have_collision_free_attempt_identities() -> Result<()> {
    let matrix = plan(&planning_fixture()?)?;
    let plan = plan_benchmark_case_attempts(
        &matrix,
        "production-portfolio-20260814",
        "attempt-001",
        "2026-08-14T12:00:00Z",
    )?;

    let matrix_sha256 = bijux_dna_bench::benchmark_case_matrix_sha256(&matrix)?;
    assert_eq!(plan.case_matrix_sha256.as_deref(), Some(matrix_sha256.as_str()));
    verify_case_campaign_plan_source(&plan, &matrix)?;
    assert!(verify_campaign_plan_source(
        &plan,
        &matrix.repository_commit,
        &matrix.campaign_registry_sha256,
    )
    .is_err());
    assert_eq!(plan.cycle.attempts.len(), matrix.cases.len());
    let identities = plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| {
            (
                attempt.identity.case_id.as_deref(),
                attempt.identity.tool_id.as_str(),
                attempt.identity.attempt_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(identities.len(), matrix.cases.len());
    assert!(plan.cycle.attempts.iter().all(|attempt| attempt.identity.case_id.is_some()));
    Ok(())
}

#[test]
fn case_scoped_plan_loading_requires_the_exact_matrix_authority() -> Result<()> {
    let repository = clean_source_repository()?;
    let repository_commit = git_stdout(repository.path(), &["rev-parse", "HEAD"])?;
    let campaign_registry_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository.path().join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let fixture = planning_fixture()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &repository_commit,
        campaign_registry_sha256: &campaign_registry_sha256,
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &fixture.campaigns,
        interfaces: &fixture.interfaces,
        bindings: &fixture.bindings,
        data_snapshots: &fixture.snapshots,
    })?;
    let plan = plan_benchmark_case_attempts(
        &matrix,
        "production-portfolio-20260814",
        "attempt-001",
        "2026-08-14T12:00:00Z",
    )?;
    let output = bijux_dna_infra::temp_dir("case-scoped-plan-authority")?;
    let matrix_path = output.path().join("case-matrix.json");
    let plan_path = output.path().join("campaign-plan.json");
    ensure_benchmark_case_matrix(&matrix_path, &matrix)?;
    fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;

    assert!(load_checked_in_benchmark_campaign_plan(repository.path(), &plan_path, None).is_err());
    assert_eq!(
        load_checked_in_benchmark_campaign_plan(repository.path(), &plan_path, Some(&matrix_path),)?,
        plan
    );
    Ok(())
}

#[test]
fn generic_case_retries_preserve_the_complete_matrix_lineage() -> Result<()> {
    let matrix = plan(&planning_fixture()?)?;
    let initial = plan_benchmark_case_attempts(
        &matrix,
        "production-portfolio-20260814",
        "attempt-001",
        "2026-08-14T12:00:00Z",
    )?;
    let mut terminal = initial.cycle.attempts;
    for attempt in &mut terminal {
        append_attempt_transition(
            attempt,
            BenchmarkAttemptTransition {
                sequence: 2,
                from_status: Some(BenchmarkLifecycleStatus::Planned),
                to_status: BenchmarkLifecycleStatus::BlockedExternal,
                observed_at_utc: "2026-08-14T12:01:00Z".to_string(),
                reason: Some("required execution artifact is not admitted".to_string()),
                jobs: Vec::new(),
                accounting: Vec::new(),
                evidence: Vec::new(),
            },
        )?;
    }

    let retry = plan_benchmark_case_retry_attempts(
        &matrix,
        terminal,
        "attempt-002",
        "2026-08-14T12:02:00Z",
    )?;
    let active = retry
        .cycle
        .attempts
        .iter()
        .filter(|attempt| attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned))
        .collect::<Vec<_>>();
    assert_eq!(active.len(), matrix.cases.len());
    assert!(active.iter().all(|attempt| {
        attempt.supersedes_attempt_id.as_deref() == Some("attempt-001")
            && attempt.identity.case_id.is_some()
    }));
    Ok(())
}

#[test]
fn checked_in_run_2001_plans_through_the_canonical_binding_registry() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2001");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.genotyping");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2001 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(matrix.cases.len(), 1);
    assert_eq!(matrix.cases[0].run_id, "run-2001");
    assert_eq!(matrix.cases[0].tool_id, "bcftools");
    assert_eq!(matrix.data_snapshots.len(), 4);
    assert_eq!(
        matrix.data_snapshots.iter().map(|snapshot| snapshot.role).collect::<BTreeSet<_>>(),
        BTreeSet::from([
            BenchmarkDataSnapshotRoleV1::Corpus,
            BenchmarkDataSnapshotRoleV1::Reference,
            BenchmarkDataSnapshotRoleV1::Database,
        ])
    );
    assert_eq!(
        matrix.cases[0].binding.normalized_output_contract_id,
        matrix.cases[0].interface.output_contract
    );
    Ok(())
}

#[test]
fn checked_in_run_2002_plans_all_coverage_tools_through_canonical_bindings() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2002");
    interfaces.interfaces.retain(|interface| interface.stage_id == COVERAGE_STAGE);
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2002 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(
        matrix.cases.iter().map(|case| case.tool_id.as_str()).collect::<Vec<_>>(),
        ["bedtools", "mosdepth", "samtools"]
    );
    assert!(matrix.cases.iter().all(|case| {
        case.binding.normalized_output_contract_id == case.interface.output_contract
            && case.binding.resources.threads == case.measurement.allocated_cpus
    }));
    Ok(())
}

#[test]
fn checked_in_run_2005_plans_both_aligners_through_canonical_bindings() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2005");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.align");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2005 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(
        matrix.cases.iter().map(|case| case.tool_id.as_str()).collect::<Vec<_>>(),
        ["bowtie2", "bwa"]
    );
    assert!(matrix.cases.iter().all(|case| {
        case.binding.normalized_output_contract_id == case.interface.output_contract
            && case.binding.resources.threads == case.measurement.allocated_cpus
    }));
    Ok(())
}

#[test]
fn checked_in_run_2030_plans_fastqc_through_the_canonical_binding() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2030");
    interfaces.interfaces.retain(|interface| interface.stage_id == "fastq.detect_adapters");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2030 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert!(!matrix.cases.is_empty());
    assert!(matrix.cases.iter().all(|case| {
        case.run_id == "run-2030"
            && case.tool_id == "fastqc"
            && case.binding.resources.threads == 4
            && case.binding.normalized_output_contract_id == case.interface.output_contract
            && case.binding.outputs.iter().any(|output| {
                output.name == "adapter_evidence_dir"
                    && output.artifact_contract_id == "bijux.auxiliary.artifact.directory.v1"
            })
    }));
    Ok(())
}

#[test]
fn checked_in_run_2003_plans_endogenous_content_through_canonical_binding() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2003");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.endogenous_content");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2003 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(matrix.cases.len(), 1);
    assert_eq!(matrix.cases[0].tool_id, "samtools");
    assert_eq!(matrix.cases[0].binding.resources.threads, 4);
    assert_eq!(matrix.data_snapshots.len(), 2);
    assert_eq!(
        matrix.cases[0].binding.normalized_output_contract_id,
        matrix.cases[0].interface.output_contract
    );
    Ok(())
}

#[test]
fn checked_in_run_2004_plans_mapping_summary_tools_through_canonical_bindings() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2004");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.mapping_summary");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2004 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(
        matrix.cases.iter().map(|case| case.tool_id.as_str()).collect::<Vec<_>>(),
        ["picard", "samtools"]
    );
    assert!(matrix.cases.iter().all(|case| {
        case.binding.resources.threads == 1
            && case.binding.normalized_output_contract_id == case.interface.output_contract
    }));
    assert_eq!(matrix.data_snapshots.len(), 1);
    Ok(())
}

#[test]
fn checked_in_run_2011_plans_duplication_metrics_through_canonical_bindings() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2011");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.duplication_metrics");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2011 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(
        matrix.cases.iter().map(|case| case.tool_id.as_str()).collect::<Vec<_>>(),
        ["picard", "samtools"]
    );
    assert_eq!(matrix.cases[0].binding.resources.threads, 1);
    assert_eq!(matrix.cases[1].binding.resources.threads, 4);
    assert_eq!(matrix.data_snapshots.len(), 1);
    Ok(())
}

#[test]
fn checked_in_run_2015_plans_insert_size_through_canonical_binding() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2015");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.insert_size");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2015 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(matrix.cases.len(), 1);
    assert_eq!(matrix.cases[0].tool_id, "picard");
    assert_eq!(matrix.cases[0].binding.resources.threads, 1);
    assert_eq!(
        matrix.cases[0].binding.normalized_output_contract_id,
        "bijux.benchmark.bam.insert_size.bundle.v1"
    );
    assert_eq!(matrix.data_snapshots.len(), 1);
    Ok(())
}

#[test]
fn checked_in_run_2013_plans_gc_bias_through_canonical_binding() -> Result<()> {
    let (mut campaigns, _, mut interfaces) = load_checked_in_benchmark_planning_contracts()?;
    campaigns.campaigns.retain(|campaign| campaign.run_id == "run-2013");
    interfaces.interfaces.retain(|interface| interface.stage_id == "bam.gc_bias");
    let campaign = campaigns
        .campaigns
        .first()
        .ok_or_else(|| anyhow::anyhow!("checked-in run-2013 campaign is absent"))?;
    let snapshots = campaign_snapshots(campaign);
    let bindings = canonical_tool_bindings()?;
    let matrix = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &"1".repeat(40),
        campaign_registry_sha256: &"2".repeat(64),
        interface_catalog_sha256: &"3".repeat(64),
        tool_binding_registry_sha256: &"4".repeat(64),
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &bindings,
        data_snapshots: &snapshots,
    })?;

    assert_eq!(matrix.cases.len(), 1);
    assert_eq!(matrix.cases[0].tool_id, "picard");
    assert_eq!(matrix.cases[0].binding.resources.threads, 1);
    assert_eq!(
        matrix.cases[0].binding.normalized_output_contract_id,
        "bijux.benchmark.bam.gc_bias.bundle.v1"
    );
    assert_eq!(matrix.data_snapshots.len(), 2);
    Ok(())
}

#[test]
fn adding_a_tool_and_stage_changes_only_registry_data() -> Result<()> {
    let mut fixture = planning_fixture()?;
    let original_count = plan(&fixture)?.cases.len();

    let coverage = fixture
        .campaigns
        .campaigns
        .iter_mut()
        .find(|campaign| campaign.stage_id == "bam.coverage")
        .expect("checked-in coverage campaign");
    coverage.tool_ids.push("synthetic-coverage".to_string());
    coverage.threads_by_tool.insert("synthetic-coverage".to_string(), 2);
    let coverage_corpus_count = coverage.corpus_catalog_ids.len();
    let coverage_interface = fixture
        .interfaces
        .interfaces
        .iter_mut()
        .find(|interface| interface.stage_id == "bam.coverage")
        .expect("checked-in coverage interface");
    coverage_interface.tool_ids.push("synthetic-coverage".to_string());
    coverage_interface.adapters.insert(
        "synthetic-coverage".to_string(),
        "benchmark.synthetic.coverage.normalize".to_string(),
    );
    fixture.bindings.insert(binding(coverage_interface, "synthetic-coverage", 2))?;

    let mut added_campaign = fixture.campaigns.campaigns[0].clone();
    added_campaign.run_id = "run-2999".to_string();
    added_campaign.slug = "bam-synthetic-summary".to_string();
    added_campaign.stage_id = "bam.synthetic_summary".to_string();
    added_campaign.tool_ids = vec!["synthetic-summary".to_string()];
    added_campaign.threads = Some(2);
    added_campaign.threads_by_tool.clear();
    let added_stage_corpus_count = added_campaign.corpus_catalog_ids.len();

    let mut added_interface = fixture.interfaces.interfaces[0].clone();
    added_interface.stage_id = added_campaign.stage_id.clone();
    added_interface.output_contract = "bijux.benchmark.bam.synthetic_summary.bundle.v1".to_string();
    added_interface.tool_ids = added_campaign.tool_ids.clone();
    added_interface.adapters = BTreeMap::from([(
        "synthetic-summary".to_string(),
        "benchmark.synthetic.summary.normalize".to_string(),
    )]);
    added_interface.parameter_projections.clear();
    added_interface.downstream_probe.consumer_stage_id =
        "benchmark.bam.synthetic_summary.consume".to_string();
    added_interface.downstream_probe.input_contract = added_interface.output_contract.clone();
    fixture.bindings.insert(binding(&added_interface, "synthetic-summary", 2))?;
    fixture.campaigns.campaigns.push(added_campaign);
    fixture.interfaces.interfaces.push(added_interface);

    let matrix = plan(&fixture)?;
    assert_eq!(
        matrix.cases.len(),
        original_count + coverage_corpus_count + added_stage_corpus_count
    );
    assert!(matrix
        .cases
        .iter()
        .any(|case| { case.stage_id == "bam.coverage" && case.tool_id == "synthetic-coverage" }));
    assert!(matrix.cases.iter().any(|case| {
        case.stage_id == "bam.synthetic_summary" && case.tool_id == "synthetic-summary"
    }));
    Ok(())
}

#[test]
fn campaign_binding_cpu_drift_is_refused() -> Result<()> {
    let mut fixture = planning_fixture()?;
    let campaign = &fixture.campaigns.campaigns[0];
    let interface = fixture
        .interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == campaign.stage_id)
        .expect("campaign interface");
    let tool = &campaign.tool_ids[0];
    let mut replacement = ToolBindingRegistry::default();
    for registered in fixture.bindings.bindings() {
        if registered.stage_id.as_str() == campaign.stage_id && registered.tool_id.as_str() == tool
        {
            replacement.insert(binding(interface, tool, registered.resources.threads + 1))?;
        } else {
            replacement.insert(registered.clone())?;
        }
    }
    fixture.bindings = replacement;

    let error = plan(&fixture).expect_err("resource drift must fail planning");
    assert!(error.to_string().contains("CPU allocation differs"));
    Ok(())
}

#[test]
fn campaign_data_authority_must_be_exact_and_role_correct() -> Result<()> {
    let mut missing = planning_fixture()?;
    missing.snapshots.pop();
    assert!(plan(&missing).is_err_and(|error| {
        error.to_string().contains("do not exactly close campaign data authorities")
    }));

    let mut wrong_role = planning_fixture()?;
    let snapshot = wrong_role.snapshots.first_mut().expect("data snapshot");
    snapshot.role = match snapshot.role {
        BenchmarkDataSnapshotRoleV1::Corpus => BenchmarkDataSnapshotRoleV1::Reference,
        _ => BenchmarkDataSnapshotRoleV1::Corpus,
    };
    assert!(plan(&wrong_role).is_err_and(|error| {
        error.to_string().contains("do not exactly close campaign data authorities")
    }));

    let mut extra = planning_fixture()?;
    extra.snapshots.push(BenchmarkDataSnapshotV1 {
        catalog_id: "unreferenced-production-data".to_string(),
        role: BenchmarkDataSnapshotRoleV1::Database,
        catalog_sha256: "f".repeat(64),
        sample_ids: Vec::new(),
    });
    assert!(plan(&extra).is_err_and(|error| {
        error.to_string().contains("do not exactly close campaign data authorities")
    }));
    Ok(())
}

#[test]
fn campaign_binding_normalized_output_drift_is_refused() -> Result<()> {
    let mut fixture = planning_fixture()?;
    let first = fixture.bindings.bindings().next().expect("checked-in binding").clone();
    let mut replacement = ToolBindingRegistry::default();
    for registered in fixture.bindings.bindings() {
        if registered.binding_id == first.binding_id {
            let mut drifted = registered.clone();
            drifted.normalized_output_contract_id =
                format!("bijux.benchmark.{}.bundle.v1", registered.stage_id);
            drifted.normalized_output_contract_id.push_str(".drift");
            replacement.insert(drifted).expect_err("invalid normalized contract must fail");
        } else {
            replacement.insert(registered.clone())?;
        }
    }

    let interface = fixture
        .interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == first.stage_id.as_str())
        .expect("checked-in interface");
    let mut drifted = first;
    drifted.normalized_output_contract_id = "bijux.fastq.artifact.reads.v1".to_string();
    if interface.output_contract == drifted.normalized_output_contract_id {
        drifted.normalized_output_contract_id = "bijux.auxiliary.artifact.json.v1".to_string();
    }
    replacement.insert(drifted)?;
    fixture.bindings = replacement;

    let error = plan(&fixture).expect_err("normalized stage output drift must fail planning");
    assert!(format!("{error:#}").contains("downstream consumer semantics"));
    Ok(())
}

#[test]
fn campaign_downstream_probe_drift_is_refused() -> Result<()> {
    let mut fixture = planning_fixture()?;
    let interface = fixture.interfaces.interfaces.first_mut().expect("checked-in interface");
    interface.downstream_probe.consumer_stage_id = "benchmark.invalid.consumer.probe".to_string();

    let error = plan(&fixture).expect_err("downstream probe drift must fail planning");
    assert!(error.to_string().contains("canonical downstream probe"));
    Ok(())
}

#[test]
fn admitted_observations_aggregate_all_cases_without_dispatch() -> Result<()> {
    let fixture = planning_fixture()?;
    let matrix = plan(&fixture)?;
    validate_benchmark_case_matrix(&matrix)?;
    let mut observations = admitted_observations(&matrix);
    let aggregations = aggregate_benchmark_case_observations(&matrix, &observations)?;
    assert_eq!(aggregations.len(), EXISTING_CASE_COUNT);
    assert!(aggregations.iter().all(|aggregation| {
        aggregation.repetition_count == 3
            && aggregation.performance.len()
                == matrix
                    .cases
                    .iter()
                    .find(|case| case.case_id == aggregation.case_id)
                    .expect("aggregation case")
                    .measurement
                    .performance_metrics
                    .len()
    }));

    observations.reverse();
    assert_eq!(
        serde_json::to_vec(&aggregations)?,
        serde_json::to_vec(&aggregate_benchmark_case_observations(&matrix, &observations)?)?
    );
    Ok(())
}

#[test]
fn persisted_case_evidence_sets_are_canonical_source_bound_and_no_clobber() -> Result<()> {
    let fixture = planning_fixture()?;
    let matrix = plan(&fixture)?;
    let observations = admitted_observations(&matrix);
    let expected = build_benchmark_admitted_case_observation_set(&matrix, observations.clone())?;
    let mut reversed = observations;
    reversed.reverse();
    let observation_set = build_benchmark_admitted_case_observation_set(&matrix, reversed)?;
    assert_eq!(observation_set, expected);
    validate_benchmark_admitted_case_observation_set(&matrix, &observation_set)?;

    let aggregation_set =
        aggregate_benchmark_admitted_case_observation_set(&matrix, &observation_set)?;
    validate_benchmark_case_aggregation_set(&matrix, &observation_set, &aggregation_set)?;

    let root = tempfile::tempdir()?;
    let matrix_path = root.path().join("case-matrix.json");
    let observations_path = root.path().join("admitted-observations.json");
    let aggregations_path = root.path().join("aggregations.json");
    ensure_benchmark_case_matrix(&matrix_path, &matrix)?;
    ensure_benchmark_admitted_case_observation_set(&observations_path, &matrix, &observation_set)?;
    ensure_benchmark_case_aggregation_set(
        &aggregations_path,
        &matrix,
        &observation_set,
        &aggregation_set,
    )?;
    assert_eq!(load_benchmark_case_matrix(&matrix_path)?, matrix);
    assert_eq!(
        load_benchmark_admitted_case_observation_set(&observations_path, &matrix)?,
        observation_set
    );
    assert_eq!(
        load_benchmark_case_aggregation_set(&aggregations_path, &matrix, &observation_set)?,
        aggregation_set
    );

    ensure_benchmark_case_matrix(&matrix_path, &matrix)?;
    std::fs::write(&matrix_path, b"{}")?;
    assert!(ensure_benchmark_case_matrix(&matrix_path, &matrix).is_err());
    assert_eq!(std::fs::read(&matrix_path)?, b"{}");
    Ok(())
}

#[test]
fn persisted_case_evidence_sets_reject_upstream_digest_drift() -> Result<()> {
    let fixture = planning_fixture()?;
    let matrix = plan(&fixture)?;
    let mut observations =
        build_benchmark_admitted_case_observation_set(&matrix, admitted_observations(&matrix))?;
    observations.case_matrix_sha256 = "f".repeat(64);
    assert!(validate_benchmark_admitted_case_observation_set(&matrix, &observations).is_err());

    let observations =
        build_benchmark_admitted_case_observation_set(&matrix, admitted_observations(&matrix))?;
    let mut aggregations =
        aggregate_benchmark_admitted_case_observation_set(&matrix, &observations)?;
    aggregations.observation_set_sha256 = "e".repeat(64);
    assert!(validate_benchmark_case_aggregation_set(&matrix, &observations, &aggregations).is_err());
    Ok(())
}

#[test]
fn aggregation_refuses_incomplete_or_nondeterministic_evidence() -> Result<()> {
    let fixture = planning_fixture()?;
    let matrix = plan(&fixture)?;
    let mut observations = admitted_observations(&matrix);
    observations.pop();
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("missing case evidence must fail")
        .to_string()
        .contains("do not close"));

    let mut observations = admitted_observations(&matrix);
    observations[0].repetitions[1].normalized_result_sha256 = "9".repeat(64);
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("normalized result drift must fail")
        .to_string()
        .contains("differ across"));

    let mut observations = admitted_observations(&matrix);
    let metric = matrix.cases[0].measurement.performance_metrics[0].clone();
    observations[0].repetitions[0].performance.remove(&metric);
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("metric closure drift must fail")
        .to_string()
        .contains("metric closure"));

    let mut observations = admitted_observations(&matrix);
    observations[0].execution.image.sha256 = "invalid".to_string();
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("invalid image provenance must fail")
        .to_string()
        .contains("image digest"));

    let mut observations = admitted_observations(&matrix);
    observations[0].execution.input_bytes.clear();
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("incomplete input-byte closure must fail")
        .to_string()
        .contains("input-byte closure"));

    let mut observations = admitted_observations(&matrix);
    observations[0].execution.slurm.allocated_cpus = "999".to_string();
    assert!(aggregate_benchmark_case_observations(&matrix, &observations)
        .expect_err("scheduler CPU drift must fail")
        .to_string()
        .contains("CPU evidence"));
    Ok(())
}

#[test]
fn generic_aggregations_build_a_citable_stage_report_without_dispatch() -> Result<()> {
    let fixture = planning_fixture()?;
    let mut matrix = plan(&fixture)?;
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("benchmark crate is inside the repository");
    matrix.repository_commit = bijux_dna_containers::api::repository_head_commit(repository_root)?;
    let observations = admitted_observations(&matrix);
    let aggregations = aggregate_benchmark_case_observations(&matrix, &observations)?;
    validate_benchmark_case_aggregations(&matrix, &aggregations)?;
    let run_aggregations = aggregations
        .iter()
        .filter(|aggregation| aggregation.run_id == "run-2002")
        .cloned()
        .collect::<Vec<_>>();

    let report = build_benchmark_stage_report_from_case_aggregations(
        repository_root,
        &matrix,
        &run_aggregations,
        "run-2002",
    )?;
    assert_eq!(report.run_id, "run-2002");
    assert_eq!(report.stage_id, "bam.coverage");
    assert_eq!(report.tools.len(), 3);
    assert!(report.tools.iter().all(|tool| {
        tool.image.size_bytes == 1024
            && tool.slurm.job_id == "123456"
            && tool.source_evidence_sha256 == "e".repeat(64)
            && tool.input_locks.len() == 1
            && tool.determinism.identical_across_repetitions
    }));
    Ok(())
}

#[test]
fn generic_report_refuses_persisted_aggregation_drift() -> Result<()> {
    let fixture = planning_fixture()?;
    let matrix = plan(&fixture)?;
    let observations = admitted_observations(&matrix);
    let mut aggregations = aggregate_benchmark_case_observations(&matrix, &observations)?;
    aggregations[0].execution.image.sha256 = "f".repeat(64);
    aggregations[0].execution.input_bytes.clear();
    let error = validate_benchmark_case_aggregations(&matrix, &aggregations)
        .expect_err("persisted execution drift must fail validation");
    assert!(error.to_string().contains("input-byte closure"));
    Ok(())
}

fn planning_fixture() -> Result<PlanningFixture> {
    let (campaigns, _, interfaces) = load_checked_in_benchmark_planning_contracts()?;
    let mut bindings = ToolBindingRegistry::default();
    for campaign in &campaigns.campaigns {
        let interface = interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == campaign.stage_id)
            .expect("validated campaign/interface closure");
        for tool in &campaign.tool_ids {
            let threads = campaign
                .threads_by_tool
                .get(tool)
                .copied()
                .or(campaign.threads)
                .expect("validated campaign CPU allocation");
            bindings.insert(binding(interface, tool, threads))?;
        }
    }
    let data_ids = campaigns
        .campaigns
        .iter()
        .flat_map(|campaign| {
            [
                (&campaign.corpus_catalog_ids, BenchmarkDataSnapshotRoleV1::Corpus),
                (&campaign.reference_catalog_ids, BenchmarkDataSnapshotRoleV1::Reference),
                (&campaign.database_catalog_ids, BenchmarkDataSnapshotRoleV1::Database),
            ]
            .into_iter()
            .flat_map(|(ids, role)| ids.iter().cloned().map(move |id| (id, role)))
        })
        .collect::<BTreeSet<_>>();
    let snapshots = data_ids
        .into_iter()
        .enumerate()
        .map(|(index, (catalog_id, role))| BenchmarkDataSnapshotV1 {
            catalog_id,
            role,
            catalog_sha256: format!("{index:064x}"),
            sample_ids: if role == BenchmarkDataSnapshotRoleV1::Corpus {
                vec![format!("sample-{index}")]
            } else {
                Vec::new()
            },
        })
        .collect();
    Ok(PlanningFixture { campaigns, interfaces, bindings, snapshots })
}

fn clean_source_repository() -> Result<tempfile::TempDir> {
    let repository = bijux_dna_infra::temp_dir("case-plan-source-repository")?;
    let registry = repository.path().join("benchmarks/contracts/stage-campaigns.toml");
    fs::create_dir_all(registry.parent().expect("campaign registry parent"))?;
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/contracts/stage-campaigns.toml"),
        &registry,
    )?;
    for arguments in [
        vec!["init", "--initial-branch=main"],
        vec!["config", "user.email", "tests@bijux.invalid"],
        vec!["config", "user.name", "Bijux Tests"],
        vec!["add", "benchmarks/contracts/stage-campaigns.toml"],
        vec!["commit", "-m", "test: seed campaign registry"],
    ] {
        git_stdout(repository.path(), &arguments)?;
    }
    Ok(repository)
}

fn git_stdout(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(repository).args(arguments).output()?;
    anyhow::ensure!(
        output.status.success(),
        "Git fixture command failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn binding(
    interface: &bijux_dna_bench::BenchmarkStageInterfaceSpec,
    tool_id: &str,
    threads: u32,
) -> ToolBindingV1 {
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_string(),
        binding_id: format!("{}__{tool_id}", interface.stage_id),
        stage_id: StageId::new(&interface.stage_id),
        tool_id: ToolId::new(tool_id),
        artifact_source: ToolArtifactSourceV1::CatalogImage {
            catalog_tool_id: ToolId::new(tool_id),
        },
        capabilities: vec!["benchmark_stage_execution".to_string()],
        inputs: vec![BindingInputV1 {
            name: "source".to_string(),
            artifact_contract_id: interface.accepted_input_contracts[0].clone(),
            required: true,
        }],
        parameters: vec![BindingParameterV1 { name: "threads".to_string(), required: true }],
        outputs: vec![BindingOutputV1 {
            name: "result".to_string(),
            artifact_contract_id: interface.accepted_input_contracts[0].clone(),
            discovery: OutputDiscoveryV1::ExactPath,
        }],
        normalized_output_contract_id: interface.output_contract.clone(),
        command_template: vec![
            "${executable}".to_string(),
            "${input:source}".to_string(),
            "${parameter:threads}".to_string(),
            "${output:result}".to_string(),
        ],
        resources: ToolConstraints {
            runtime: "apptainer".to_string(),
            mem_gb: 4,
            tmp_gb: 2,
            threads,
        },
        normalizer_id: interface.adapters[tool_id].clone(),
        downstream_probe_id: interface.downstream_probe.consumer_stage_id.clone(),
        limitations: vec!["synthetic contract fixture".to_string()],
    }
}

fn admitted_observations(
    matrix: &bijux_dna_bench::BenchmarkCaseMatrixV2,
) -> Vec<BenchmarkAdmittedCaseObservationV2> {
    matrix
        .cases
        .iter()
        .map(|case| BenchmarkAdmittedCaseObservationV2 {
            schema_version: BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2.to_string(),
            case_id: case.case_id.clone(),
            repository_commit: matrix.repository_commit.clone(),
            admission_sha256: "5".repeat(64),
            input_lock_sha256: "6".repeat(64),
            parameter_lock_sha256: "7".repeat(64),
            execution: BenchmarkCaseExecutionEvidenceV1 {
                image: BenchmarkReportImage {
                    name: format!("{}.sif", case.tool_id),
                    sha256: "d".repeat(64),
                    size_bytes: 1024,
                },
                slurm: BenchmarkReportSlurmJob {
                    job_id: "123456".to_string(),
                    host: "compute-01".to_string(),
                    elapsed: "00:00:01".to_string(),
                    allocated_cpus: case.measurement.allocated_cpus.to_string(),
                },
                source_evidence_sha256: "e".repeat(64),
                input_locks: vec![BenchmarkReportInputLock {
                    id: case.corpus.corpus_id.clone(),
                    kind: "corpus_snapshot".to_string(),
                    mount_path: "/inputs/corpus".to_string(),
                    sha256: case.corpus.snapshot_sha256.clone(),
                }],
                input_bytes: case
                    .campaign
                    .input_size_keys
                    .iter()
                    .map(|key| (key.clone(), 1024))
                    .collect(),
            },
            repetitions: (1..=case.measurement.measured_repetitions)
                .map(|repetition| BenchmarkCaseRepetitionObservationV1 {
                    repetition,
                    allocated_cpus: case.measurement.allocated_cpus,
                    performance: case
                        .measurement
                        .performance_metrics
                        .iter()
                        .map(|metric| (metric.clone(), f64::from(repetition)))
                        .collect(),
                    scientific: case
                        .science
                        .scientific_metrics
                        .iter()
                        .map(|metric| {
                            (
                                metric.clone(),
                                BenchmarkScientificValue::Number(f64::from(repetition)),
                            )
                        })
                        .collect(),
                    normalized_result_sha256: "8".repeat(64),
                })
                .collect(),
        })
        .collect()
}

fn plan(fixture: &PlanningFixture) -> Result<bijux_dna_bench::BenchmarkCaseMatrixV2> {
    let repository_commit = "1".repeat(40);
    let campaign_digest = "2".repeat(64);
    let interface_digest = "3".repeat(64);
    let binding_digest = "4".repeat(64);
    plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &repository_commit,
        campaign_registry_sha256: &campaign_digest,
        interface_catalog_sha256: &interface_digest,
        tool_binding_registry_sha256: &binding_digest,
        campaigns: &fixture.campaigns,
        interfaces: &fixture.interfaces,
        bindings: &fixture.bindings,
        data_snapshots: &fixture.snapshots,
    })
}

fn campaign_snapshots(
    campaign: &bijux_dna_bench::BenchmarkStageCampaignSpec,
) -> Vec<BenchmarkDataSnapshotV1> {
    [
        (&campaign.corpus_catalog_ids, BenchmarkDataSnapshotRoleV1::Corpus),
        (&campaign.reference_catalog_ids, BenchmarkDataSnapshotRoleV1::Reference),
        (&campaign.database_catalog_ids, BenchmarkDataSnapshotRoleV1::Database),
    ]
    .into_iter()
    .flat_map(|(ids, role)| {
        ids.iter().map(move |id| BenchmarkDataSnapshotV1 {
            catalog_id: id.clone(),
            role,
            catalog_sha256: "a".repeat(64),
            sample_ids: if role == BenchmarkDataSnapshotRoleV1::Corpus {
                vec!["contract-sample".to_string()]
            } else {
                Vec::new()
            },
        })
    })
    .collect()
}
