//! Owner: bijux-dna-bench
//! Registry-driven benchmark case-matrix planning.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_defined_interface_campaign_alignment, validate_stage_campaign_registry,
    validate_stage_interface_catalog,
};
use bijux_dna_bench_model::{BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{ToolBinding, ToolBindingRegistry};

use super::{
    BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1, BENCHMARK_CASE_MATRIX_SCHEMA_V1,
};

pub struct BenchmarkCaseMatrixRequest<'a> {
    pub repository_commit: &'a str,
    pub campaign_registry_sha256: &'a str,
    pub interface_catalog_sha256: &'a str,
    pub tool_binding_registry_sha256: &'a str,
    pub campaigns: &'a BenchmarkStageCampaignRegistry,
    pub interfaces: &'a BenchmarkStageInterfaceCatalog,
    pub bindings: &'a ToolBindingRegistry,
    pub corpus_snapshots: &'a [BenchmarkCorpusSnapshotV1],
}

/// Join declared campaign, interface, binding, corpus, measurement, and science authorities.
///
/// The planner contains no stage or tool dispatch. Adding a stage or tool changes only validated
/// registry inputs and therefore produces a new case without lifecycle code changes.
///
/// # Errors
/// Returns an error when source identities, authority alignment, corpus closure, bindings, or
/// resource fairness are incomplete or contradictory.
pub fn plan_benchmark_case_matrix(
    request: &BenchmarkCaseMatrixRequest<'_>,
) -> Result<BenchmarkCaseMatrixV1> {
    require_identity(request.repository_commit, 40, "repository commit")?;
    for (value, label) in [
        (request.campaign_registry_sha256, "campaign registry digest"),
        (request.interface_catalog_sha256, "interface catalog digest"),
        (request.tool_binding_registry_sha256, "tool binding registry digest"),
    ] {
        require_identity(value, 64, label)?;
    }
    validate_stage_campaign_registry(request.campaigns)
        .context("validate generic campaign registry")?;
    validate_stage_interface_catalog(request.interfaces)
        .context("validate generic stage interfaces")?;
    validate_defined_interface_campaign_alignment(request.interfaces, request.campaigns)
        .context("align generic campaign and interface authorities")?;

    let snapshots = snapshot_index(request.corpus_snapshots)?;
    let mut campaigns = request.campaigns.campaigns.iter().collect::<Vec<_>>();
    campaigns.sort_by(|left, right| left.run_id.cmp(&right.run_id));
    let mut cases = Vec::new();
    for campaign in campaigns {
        let interface = request
            .interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == campaign.stage_id)
            .context("campaign stage lacks an interface")?;
        let stage_id = StageId::new(&campaign.stage_id);
        let mut tools = campaign.tool_ids.clone();
        tools.sort();
        let mut corpora = campaign.corpus_catalog_ids.clone();
        corpora.sort();
        for tool in tools {
            let tool_id = ToolId::new(&tool);
            let binding = request.bindings.binding(&stage_id, &tool_id).with_context(|| {
                format!("campaign {} lacks binding for tool {tool}", campaign.run_id)
            })?;
            validate_binding_interface_join(binding, interface)?;
            let allocated_cpus =
                campaign.threads_by_tool.get(&tool).copied().or(campaign.threads).with_context(
                    || format!("campaign {} lacks CPU allocation", campaign.run_id),
                )?;
            if binding.resources().threads != allocated_cpus {
                bail!(
                    "campaign {} CPU allocation differs from binding {}",
                    campaign.run_id,
                    binding.binding_id()
                );
            }
            for corpus_id in &corpora {
                let corpus = snapshots.get(corpus_id.as_str()).with_context(|| {
                    format!("campaign {} lacks corpus snapshot {corpus_id}", campaign.run_id)
                })?;
                cases.push(BenchmarkCaseDefinitionV1 {
                    case_id: case_id(&campaign.run_id, &tool, corpus_id)?,
                    run_id: campaign.run_id.clone(),
                    domain: campaign.domain.clone(),
                    stage_id: campaign.stage_id.clone(),
                    tool_id: tool.clone(),
                    corpus: (*corpus).clone(),
                    campaign: campaign.clone(),
                    binding: binding.clone(),
                    interface: interface.clone(),
                    measurement: BenchmarkCaseMeasurementPlanV1 {
                        allocated_cpus,
                        warmup_repetitions: campaign.warmup_repetitions,
                        measured_repetitions: campaign.measured_repetitions,
                        cache_policy: campaign.cache_policy,
                        performance_metrics: campaign.performance_metrics.clone(),
                    },
                    science: BenchmarkCaseSciencePlanV1 {
                        evidence_tier: campaign.evidence_tier,
                        profile_id: campaign.profile_id.clone(),
                        scientific_question: campaign.scientific_question.clone(),
                        comparison_scope: campaign.comparison_scope.clone(),
                        accuracy_claim: campaign.accuracy_claim.clone(),
                        interpretation: campaign.interpretation.clone(),
                        scientific_metrics: campaign.scientific_metrics.clone(),
                        comparison_metrics: campaign.comparison_metrics.clone(),
                        limitations: campaign.limitations.clone(),
                    },
                });
            }
        }
    }
    if cases.is_empty() {
        bail!("benchmark case matrix is empty");
    }
    Ok(BenchmarkCaseMatrixV1 {
        schema_version: BENCHMARK_CASE_MATRIX_SCHEMA_V1.to_owned(),
        repository_commit: request.repository_commit.to_owned(),
        campaign_registry_sha256: request.campaign_registry_sha256.to_owned(),
        interface_catalog_sha256: request.interface_catalog_sha256.to_owned(),
        tool_binding_registry_sha256: request.tool_binding_registry_sha256.to_owned(),
        cases,
    })
}

/// Reconstruct every case authority and require exact pure-planner replay.
///
/// # Errors
/// Returns an error when persisted cases are incomplete, conflicting, malformed, or not in the
/// canonical planner order.
pub fn validate_benchmark_case_matrix(matrix: &BenchmarkCaseMatrixV1) -> Result<()> {
    if matrix.schema_version != BENCHMARK_CASE_MATRIX_SCHEMA_V1 {
        bail!("benchmark case matrix schema is unsupported");
    }
    let mut campaigns = BTreeMap::new();
    let mut interfaces = BTreeMap::new();
    let mut bindings = BTreeMap::new();
    let mut snapshots = BTreeMap::new();
    for case in &matrix.cases {
        insert_identical(&mut campaigns, &case.run_id, &case.campaign, "campaign")?;
        insert_identical(&mut interfaces, &case.stage_id, &case.interface, "interface")?;
        insert_identical(&mut bindings, &case.binding.binding_id, &case.binding, "binding")?;
        insert_identical(&mut snapshots, &case.corpus.corpus_id, &case.corpus, "corpus")?;
    }
    let campaigns = BenchmarkStageCampaignRegistry {
        schema_version: bijux_dna_bench_model::BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1.to_string(),
        campaigns: campaigns.into_values().cloned().collect(),
    };
    let interfaces = BenchmarkStageInterfaceCatalog {
        schema_version: bijux_dna_bench_model::BENCHMARK_STAGE_INTERFACES_SCHEMA_V1.to_string(),
        interfaces: interfaces.into_values().cloned().collect(),
    };
    let mut registry = ToolBindingRegistry::default();
    for binding in bindings.into_values() {
        registry.insert(binding.clone())?;
    }
    let snapshots = snapshots.into_values().cloned().collect::<Vec<_>>();
    let replay = plan_benchmark_case_matrix(&BenchmarkCaseMatrixRequest {
        repository_commit: &matrix.repository_commit,
        campaign_registry_sha256: &matrix.campaign_registry_sha256,
        interface_catalog_sha256: &matrix.interface_catalog_sha256,
        tool_binding_registry_sha256: &matrix.tool_binding_registry_sha256,
        campaigns: &campaigns,
        interfaces: &interfaces,
        bindings: &registry,
        corpus_snapshots: &snapshots,
    })?;
    if replay != *matrix {
        bail!("benchmark case matrix differs from canonical planner replay");
    }
    Ok(())
}

fn insert_identical<'a, T: PartialEq>(
    values: &mut BTreeMap<&'a str, &'a T>,
    key: &'a str,
    value: &'a T,
    label: &str,
) -> Result<()> {
    if values.insert(key, value).is_some_and(|existing| existing != value) {
        bail!("benchmark case matrix contains conflicting {label} authority");
    }
    Ok(())
}

fn snapshot_index(
    snapshots: &[BenchmarkCorpusSnapshotV1],
) -> Result<BTreeMap<&str, &BenchmarkCorpusSnapshotV1>> {
    let mut indexed = BTreeMap::new();
    for snapshot in snapshots {
        require_component(&snapshot.corpus_id, "corpus id")?;
        require_identity(&snapshot.snapshot_sha256, 64, "corpus snapshot digest")?;
        if snapshot.sample_ids.is_empty() {
            bail!("corpus snapshot samples are invalid");
        }
        for sample_id in &snapshot.sample_ids {
            require_component(sample_id, "sample id")?;
        }
        if snapshot.sample_ids.iter().collect::<BTreeSet<_>>().len() != snapshot.sample_ids.len() {
            bail!("corpus snapshot samples are invalid");
        }
        if indexed.insert(snapshot.corpus_id.as_str(), snapshot).is_some() {
            bail!("duplicate corpus snapshot id");
        }
    }
    Ok(indexed)
}

fn validate_binding_interface_join(
    binding: &impl ToolBinding,
    interface: &bijux_dna_bench_model::BenchmarkStageInterfaceSpec,
) -> Result<()> {
    let selected_backend = binding.tool_id().as_str();
    let expected_adapter = interface
        .adapters
        .get(selected_backend)
        .with_context(|| format!("stage interface lacks adapter for backend {selected_backend}"))?;
    if binding.normalizer_id() != expected_adapter {
        bail!("binding {} uses a non-interface normalizer", binding.binding_id());
    }
    if binding.downstream_probe_id() != interface.downstream_probe.consumer_stage_id {
        bail!("binding {} uses a non-interface downstream probe", binding.binding_id());
    }
    if !binding
        .inputs()
        .iter()
        .any(|input| interface.accepted_input_contracts.contains(&input.artifact_contract_id))
    {
        bail!("binding {} accepts no declared stage input", binding.binding_id());
    }
    if !binding
        .outputs()
        .iter()
        .any(|output| output.artifact_contract_id == interface.output_contract)
    {
        bail!("binding {} emits no declared stage output", binding.binding_id());
    }
    Ok(())
}

fn case_id(campaign_id: &str, backend_id: &str, corpus_id: &str) -> Result<String> {
    for (value, label) in [
        (campaign_id, "campaign identity"),
        (backend_id, "backend identity"),
        (corpus_id, "corpus identity"),
    ] {
        require_component(value, label)?;
    }
    Ok([campaign_id, backend_id, corpus_id].join("--"))
}

fn require_identity(value: &str, length: usize, label: &str) -> Result<()> {
    if value.len() != length
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} is malformed");
    }
    Ok(())
}

fn require_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        bail!("{label} is malformed");
    }
    Ok(())
}
