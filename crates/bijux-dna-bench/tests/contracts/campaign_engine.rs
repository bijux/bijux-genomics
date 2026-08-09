use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use bijux_dna_bench::{
    aggregate_benchmark_case_observations, load_checked_in_benchmark_planning_contracts,
    plan_benchmark_case_matrix, validate_benchmark_case_matrix, BenchmarkAdmittedCaseObservationV1,
    BenchmarkCaseMatrixRequest, BenchmarkCaseRepetitionObservationV1, BenchmarkCorpusSnapshotV1,
    BenchmarkScientificValue, BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1,
};
use bijux_dna_core::{
    contract::ToolConstraints,
    ids::{StageId, ToolId},
};
use bijux_dna_stage_contract::{
    BindingInputV1, BindingOutputV1, BindingParameterV1, OutputDiscoveryV1, ToolArtifactSourceV1,
    ToolBindingRegistry, ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};

const EXISTING_CAMPAIGN_COUNT: usize = 9;
const EXISTING_CASE_COUNT: usize = 20;

struct PlanningFixture {
    campaigns: BenchmarkStageCampaignRegistry,
    interfaces: BenchmarkStageInterfaceCatalog,
    bindings: ToolBindingRegistry,
    snapshots: Vec<BenchmarkCorpusSnapshotV1>,
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
    assert!(error.to_string().contains("emits no declared stage output"));
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
    let corpus_ids = campaigns
        .campaigns
        .iter()
        .flat_map(|campaign| campaign.corpus_catalog_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let snapshots = corpus_ids
        .into_iter()
        .enumerate()
        .map(|(index, corpus_id)| BenchmarkCorpusSnapshotV1 {
            corpus_id,
            snapshot_sha256: format!("{index:064x}"),
            sample_ids: vec![format!("sample-{index}")],
        })
        .collect();
    Ok(PlanningFixture { campaigns, interfaces, bindings, snapshots })
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
    matrix: &bijux_dna_bench::BenchmarkCaseMatrixV1,
) -> Vec<BenchmarkAdmittedCaseObservationV1> {
    matrix
        .cases
        .iter()
        .map(|case| BenchmarkAdmittedCaseObservationV1 {
            schema_version: BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1.to_string(),
            case_id: case.case_id.clone(),
            repository_commit: matrix.repository_commit.clone(),
            admission_sha256: "5".repeat(64),
            input_lock_sha256: "6".repeat(64),
            parameter_lock_sha256: "7".repeat(64),
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

fn plan(fixture: &PlanningFixture) -> Result<bijux_dna_bench::BenchmarkCaseMatrixV1> {
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
        corpus_snapshots: &fixture.snapshots,
    })
}
