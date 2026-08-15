//! Owner: bijux-dna-bench-model
//! Semantic validation for benchmark planning contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkCorpusClass, BenchmarkCorpusContract, BenchmarkCorpusStatus, BenchmarkDisposition,
    BenchmarkEvidenceScope, BenchmarkEvidenceTier, BenchmarkLocalHpcJobGraph,
    BenchmarkLocalHpcJobGraphNode, BenchmarkMeasurementContract, BenchmarkPortfolio,
    BenchmarkProfileAssignment, BenchmarkRunRegistry, BenchmarkRunRegistryRow,
    BenchmarkRunRegistryState, BenchmarkScientificProfile, BenchmarkScientificReadinessReport,
    BenchmarkScientificReadinessStage, BenchmarkScientificReadinessStatus,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageDefinitionStatus, BenchmarkStageExecutionStatus,
    BenchmarkStageInterfaceStatus, BenchmarkStageUniverse,
};

use super::{
    BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V2,
    BENCHMARK_RUN_REGISTRY_SCHEMA_V1, BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2,
    BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1, BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
    BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1,
};

const REQUIRED_PERFORMANCE_METRICS: &[&str] =
    &["wall_seconds", "cpu_seconds", "peak_rss_kib", "input_bytes", "output_bytes"];
const COMPARATIVE_PERFORMANCE_METRICS: &[&str] = &[
    "wall_seconds",
    "cpu_seconds",
    "cpu_utilization_percent",
    "peak_rss_kib",
    "filesystem_input_operations",
    "filesystem_output_operations",
    "input_bytes",
    "output_bytes",
    "throughput_input_bytes_per_second",
];
const REQUIRED_RESOURCE_METRICS: &[&str] = &[
    "wall_seconds",
    "cpu_seconds",
    "cpu_utilization_percent",
    "peak_rss_kib",
    "filesystem_input_operations",
    "filesystem_output_operations",
    "input_bytes",
    "output_bytes",
    "throughput_input_bytes_per_second",
];
const REQUIRED_QUALITY_DIMENSIONS: &[&str] =
    &["scientific_correctness", "repeatability", "robustness", "output_contract"];
const CANONICAL_RESULT_ROOT_TEMPLATE: &str = "results/benchmarks/{run_id}-{slug}/{tool_id}";

/// Validate a local HPC benchmark job graph.
///
/// # Errors
/// Returns an error when node identity, dependency closure, ordering, roots, or counters drift.
pub fn validate_benchmark_local_hpc_job_graph(
    graph: &BenchmarkLocalHpcJobGraph,
) -> Result<(), BenchError> {
    if graph.schema_version != BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "unsupported local HPC job-graph schema `{}`",
            graph.schema_version
        )));
    }
    require_text(&graph.manifest_path, "local HPC job-graph manifest_path")?;
    let Some(expected_job_count) =
        graph.benchmark_job_count.checked_add(graph.essential_pipeline_job_count)
    else {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph job count overflowed".to_string(),
        ));
    };
    if graph.job_count == 0
        || graph.job_count != expected_job_count
        || graph.job_count != graph.nodes.len()
        || graph.job_count != graph.topological_order.len()
    {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph counters must close over non-empty nodes and ordering".to_string(),
        ));
    }

    let mut nodes_by_id = BTreeMap::new();
    let mut benchmark_job_count = 0usize;
    let mut essential_pipeline_job_count = 0usize;
    for node in &graph.nodes {
        validate_local_hpc_job_graph_node(node)?;
        if node.job_kind == "benchmark_result" {
            benchmark_job_count += 1;
        } else {
            essential_pipeline_job_count += 1;
        }
        if nodes_by_id.insert(node.job_id_local.as_str(), node).is_some() {
            return Err(BenchError::InvalidPolicy(format!(
                "local HPC job graph repeats job `{}`",
                node.job_id_local
            )));
        }
    }
    if benchmark_job_count != graph.benchmark_job_count
        || essential_pipeline_job_count != graph.essential_pipeline_job_count
    {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph kind counters do not match nodes".to_string(),
        ));
    }
    validate_local_hpc_job_graph_order(graph, &nodes_by_id)?;
    validate_local_hpc_job_graph_edges(graph, &nodes_by_id)
}

fn validate_local_hpc_job_graph_node(
    node: &BenchmarkLocalHpcJobGraphNode,
) -> Result<(), BenchError> {
    for (value, label) in [
        (node.job_id_local.as_str(), "job_id_local"),
        (node.job_kind.as_str(), "job_kind"),
        (node.domain.as_str(), "domain"),
        (node.stage_id.as_str(), "stage_id"),
        (node.tool_id.as_str(), "tool_id"),
        (node.corpus_id.as_str(), "corpus_id"),
        (node.asset_profile_id.as_str(), "asset_profile_id"),
    ] {
        require_text(value, label)?;
    }
    if !matches!(node.domain.as_str(), "fastq" | "bam" | "vcf")
        || !node.stage_id.starts_with(&format!("{}.", node.domain))
    {
        return Err(BenchError::InvalidPolicy(format!(
            "local HPC job `{}` has invalid domain/stage closure",
            node.job_id_local
        )));
    }
    let identity_shape_valid = match node.job_kind.as_str() {
        "benchmark_result" => {
            node.result_id.as_deref().is_some_and(|value| !value.trim().is_empty())
                && node.pipeline_id.is_none()
                && node.node_id.is_none()
        }
        "essential_pipeline_node" => {
            node.result_id.is_none()
                && node.pipeline_id.as_deref().is_some_and(|value| !value.trim().is_empty())
                && node.node_id.as_deref().is_some_and(|value| !value.trim().is_empty())
        }
        _ => false,
    };
    if !identity_shape_valid {
        return Err(BenchError::InvalidPolicy(format!(
            "local HPC job `{}` has invalid job-kind identity",
            node.job_id_local
        )));
    }
    validate_text_list(&node.dependencies, "local HPC job dependencies", true)?;
    validate_text_list(&node.dependents, "local HPC job dependents", true)?;
    if node.dependency_count != node.dependencies.len()
        || node.dependent_count != node.dependents.len()
    {
        return Err(BenchError::InvalidPolicy(format!(
            "local HPC job `{}` has inconsistent edge counters",
            node.job_id_local
        )));
    }
    Ok(())
}

fn validate_local_hpc_job_graph_order(
    graph: &BenchmarkLocalHpcJobGraph,
    nodes_by_id: &BTreeMap<&str, &BenchmarkLocalHpcJobGraphNode>,
) -> Result<(), BenchError> {
    validate_text_list(&graph.topological_order, "local HPC job-graph topological_order", false)?;
    let positions = graph
        .topological_order
        .iter()
        .enumerate()
        .map(|(position, job_id)| (job_id.as_str(), position))
        .collect::<BTreeMap<_, _>>();
    if positions.len() != nodes_by_id.len()
        || positions.keys().copied().collect::<BTreeSet<_>>()
            != nodes_by_id.keys().copied().collect::<BTreeSet<_>>()
    {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph ordering must contain the exact node closure".to_string(),
        ));
    }
    if graph.nodes.iter().map(|node| node.job_id_local.as_str()).collect::<Vec<_>>()
        != graph.topological_order.iter().map(String::as_str).collect::<Vec<_>>()
    {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph nodes must follow topological_order".to_string(),
        ));
    }
    let expected_roots = graph
        .topological_order
        .iter()
        .filter(|job_id| {
            nodes_by_id.get(job_id.as_str()).is_some_and(|node| node.dependencies.is_empty())
        })
        .cloned()
        .collect::<Vec<_>>();
    if graph.root_job_ids != expected_roots {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph roots must be the dependency-free topological nodes".to_string(),
        ));
    }
    for node in &graph.nodes {
        let node_position = positions[&node.job_id_local.as_str()];
        if node.dependencies.iter().any(|dependency| {
            positions.get(dependency.as_str()).is_none_or(|position| *position >= node_position)
        }) {
            return Err(BenchError::InvalidPolicy(format!(
                "local HPC job `{}` violates topological dependency order",
                node.job_id_local
            )));
        }
    }
    Ok(())
}

fn validate_local_hpc_job_graph_edges(
    graph: &BenchmarkLocalHpcJobGraph,
    nodes_by_id: &BTreeMap<&str, &BenchmarkLocalHpcJobGraphNode>,
) -> Result<(), BenchError> {
    let mut dependency_count = 0usize;
    for node in &graph.nodes {
        dependency_count =
            dependency_count.checked_add(node.dependencies.len()).ok_or_else(|| {
                BenchError::InvalidPolicy(
                    "local HPC job-graph dependency count overflowed".to_string(),
                )
            })?;
        for dependency in &node.dependencies {
            let Some(upstream) = nodes_by_id.get(dependency.as_str()) else {
                return Err(BenchError::InvalidPolicy(format!(
                    "local HPC job `{}` references unknown dependency `{dependency}`",
                    node.job_id_local
                )));
            };
            if !upstream.dependents.contains(&node.job_id_local) {
                return Err(BenchError::InvalidPolicy(format!(
                    "local HPC dependency `{dependency}` does not reciprocate `{}`",
                    node.job_id_local
                )));
            }
        }
        for dependent in &node.dependents {
            let Some(downstream) = nodes_by_id.get(dependent.as_str()) else {
                return Err(BenchError::InvalidPolicy(format!(
                    "local HPC job `{}` references unknown dependent `{dependent}`",
                    node.job_id_local
                )));
            };
            if !downstream.dependencies.contains(&node.job_id_local) {
                return Err(BenchError::InvalidPolicy(format!(
                    "local HPC dependent `{dependent}` does not reciprocate `{}`",
                    node.job_id_local
                )));
            }
        }
    }
    if dependency_count != graph.dependency_count {
        return Err(BenchError::InvalidPolicy(
            "local HPC job-graph dependency_count does not match edges".to_string(),
        ));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the campaign registry shape or invariants are invalid.
pub fn validate_stage_campaign_registry(
    registry: &BenchmarkStageCampaignRegistry,
) -> Result<(), BenchError> {
    if registry.schema_version != BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign schema mismatch: {}",
            registry.schema_version
        )));
    }
    if registry.campaigns.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "stage campaign registry contains no campaigns".to_string(),
        ));
    }

    let mut seen_runs = BTreeSet::new();
    let mut seen_stages = BTreeSet::new();
    for campaign in &registry.campaigns {
        validate_stage_campaign(campaign)?;
        if !seen_runs.insert(campaign.run_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "stage campaign run id is duplicated: {}",
                campaign.run_id
            )));
        }
        if !seen_stages.insert(campaign.stage_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "stage campaign stage id is duplicated: {}",
                campaign.stage_id
            )));
        }
    }
    Ok(())
}

/// Validate a current executable campaign registry.
///
/// Historical v1 case matrices may omit the material driver and remain replayable through
/// `validate_stage_campaign_registry`. New planning authorities must use this stricter validator.
///
/// # Errors
/// Returns an error when the base registry is invalid or any campaign omits its stage-bound
/// material driver.
pub fn validate_executable_stage_campaign_registry(
    registry: &BenchmarkStageCampaignRegistry,
) -> Result<(), BenchError> {
    validate_stage_campaign_registry(registry)?;
    if let Some(campaign) =
        registry.campaigns.iter().find(|campaign| campaign.material_driver.is_none())
    {
        return Err(BenchError::InvalidPolicy(format!(
            "executable stage campaign omits its material driver: {}",
            campaign.stage_id
        )));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the portfolio shape or invariants are invalid.
pub fn validate_benchmark_portfolio(portfolio: &BenchmarkPortfolio) -> Result<(), BenchError> {
    if portfolio.schema_version != BENCHMARK_PORTFOLIO_SCHEMA_V2 {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark portfolio schema mismatch: {}",
            portfolio.schema_version
        )));
    }

    validate_measurement(&portfolio.measurement)?;
    if portfolio.stage_runs.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark portfolio stage_runs cannot be empty".to_string(),
        ));
    }

    let mut seen_run_ids = BTreeSet::new();
    for (stage_id, run_id) in &portfolio.stage_runs {
        require_text(stage_id, "portfolio stage id")?;
        if !is_portfolio_run_id(run_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark portfolio run id is outside run-2000..2999: {run_id}"
            )));
        }
        if !seen_run_ids.insert(run_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark portfolio run id is duplicated: {run_id}"
            )));
        }
    }

    let (corpus_ids, known_roles) = validate_corpora(&portfolio.corpora)?;
    let profile_ids = validate_profiles(&portfolio.profiles, &known_roles)?;
    validate_assignments(&portfolio.assignments, &profile_ids, &portfolio.stage_runs, &corpus_ids)?;
    Ok(())
}

/// # Errors
/// Returns an error when the permanent benchmark namespace, source locks, lifecycle, or stage/run
/// closure is malformed.
pub fn validate_benchmark_run_registry(registry: &BenchmarkRunRegistry) -> Result<(), BenchError> {
    if registry.schema_version != BENCHMARK_RUN_REGISTRY_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(
            "benchmark run registry schema mismatch".to_string(),
        ));
    }
    let namespace = &registry.namespace;
    if namespace.prefix != "run-"
        || namespace.minimum != 2001
        || namespace.maximum != 2999
        || namespace.next_run_number <= namespace.minimum
        || namespace.next_run_number > namespace.maximum
        || namespace.allocation_policy != "monotonic_only"
        || namespace.identity_policy != "existing_ids_immutable"
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark run namespace policy is invalid".to_string(),
        ));
    }
    if registry.source_contract_sha256.is_empty()
        || registry.source_contract_sha256.values().any(|digest| !valid_digest(digest))
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark run registry source locks are invalid".to_string(),
        ));
    }
    let mut run_ids = BTreeSet::new();
    let mut stages = BTreeSet::new();
    let mut previous_number = None;
    let mut active = 0;
    let mut reserved = 0;
    let mut retired = 0;
    for row in &registry.runs {
        let number = parse_run_id(&row.run_id)
            .filter(|number| {
                (*number >= namespace.minimum) && (*number < namespace.next_run_number)
            })
            .ok_or_else(|| {
                BenchError::InvalidPolicy(format!(
                    "registry run id is outside the allocated range: {}",
                    row.run_id
                ))
            })?;
        if previous_number.is_some_and(|previous| previous >= number)
            || !run_ids.insert(row.run_id.as_str())
            || !stages.insert(row.stage_id.as_str())
        {
            return Err(BenchError::InvalidPolicy(
                "benchmark registry rows are duplicated or unsorted".to_string(),
            ));
        }
        previous_number = Some(number);
        validate_run_registry_row(row)?;
        update_run_registry_state_counts(row, &mut active, &mut reserved, &mut retired)?;
    }
    let summary = &registry.summary;
    if summary.run_count != registry.runs.len()
        || summary.active_count != active
        || summary.reserved_count != reserved
        || summary.retired_count != retired
        || registry.runs.len() != (namespace.next_run_number - namespace.minimum) as usize
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark run registry summary or allocated range is incomplete".to_string(),
        ));
    }
    Ok(())
}

fn validate_run_registry_row(row: &BenchmarkRunRegistryRow) -> Result<(), BenchError> {
    for value in [
        row.stage_id.as_str(),
        row.domain.as_str(),
        row.title.as_str(),
        row.slug.as_str(),
        row.owner.as_str(),
        row.purpose.as_str(),
        row.profile_id.as_str(),
        row.tool_exclusion_policy.as_str(),
        row.measurement_policy_id.as_str(),
        row.acceptance_policy_id.as_str(),
    ] {
        require_text(value, "benchmark run registry field")?;
    }
    if row.owner != "bijux-dna-bench"
        || !row.stage_id.starts_with(&format!("{}.", row.domain))
        || row.slug != row.stage_id.replace(['.', '_'], "-")
        || row.tool_ids.is_empty()
        || row.required_corpus_roles.is_empty()
        || row.introduced_at_commit.len() != 40
        || !row.introduced_at_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark registry row is incomplete: {}",
            row.run_id
        )));
    }
    validate_text_list(&row.tool_ids, "registry tool_ids", false)?;
    validate_text_list(&row.required_corpus_roles, "registry required_corpus_roles", false)?;
    validate_text_list(&row.accepted_input_contracts, "registry accepted_input_contracts", true)?;
    let mut supersedes = BTreeSet::new();
    for superseded in &row.supersedes_run_ids {
        if !is_portfolio_run_id(superseded)
            || superseded == &row.run_id
            || !supersedes.insert(superseded.as_str())
        {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark registry supersession is invalid: {}",
                row.run_id
            )));
        }
    }
    if row
        .superseded_by_run_id
        .as_deref()
        .is_some_and(|successor| !is_portfolio_run_id(successor) || successor == row.run_id)
    {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark registry successor is invalid: {}",
            row.run_id
        )));
    }
    Ok(())
}

fn update_run_registry_state_counts(
    row: &BenchmarkRunRegistryRow,
    active: &mut usize,
    reserved: &mut usize,
    retired: &mut usize,
) -> Result<(), BenchError> {
    let has_reason = row.state_reason.as_deref().is_some_and(|reason| reason.trim().len() >= 20);
    let has_interface = !row.accepted_input_contracts.is_empty()
        && row.output_contract.is_some()
        && row.normalized_result_contract.is_some();
    match row.state {
        BenchmarkRunRegistryState::Active if row.state_reason.is_none() && has_interface => {
            *active += 1;
        }
        BenchmarkRunRegistryState::Reserved if has_reason && !has_interface => {
            *reserved += 1;
        }
        BenchmarkRunRegistryState::Retired if has_reason => {
            *retired += 1;
        }
        _ => {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark registry lifecycle is inconsistent: {}",
                row.run_id
            )));
        }
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// # Errors
/// Returns an error if campaign and portfolio contracts drift from each other.
pub fn validate_portfolio_campaign_alignment(
    registry: &BenchmarkStageCampaignRegistry,
    portfolio: &BenchmarkPortfolio,
) -> Result<(), BenchError> {
    validate_campaign_portfolio_alignment_impl(registry, portfolio, true)
}

/// # Errors
/// Returns an error if defined campaigns drift from their reserved portfolio identities.
pub fn validate_defined_campaign_portfolio_alignment(
    registry: &BenchmarkStageCampaignRegistry,
    portfolio: &BenchmarkPortfolio,
) -> Result<(), BenchError> {
    validate_campaign_portfolio_alignment_impl(registry, portfolio, false)
}

/// Validate a complete scientific-readiness publication.
///
/// # Errors
/// Returns an error when stage identities, readiness states, domain counts, or summary counts
/// disagree.
pub fn validate_scientific_readiness(
    report: &BenchmarkScientificReadinessReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2 {
        return Err(BenchError::InvalidPolicy(
            "benchmark scientific readiness schema mismatch".to_string(),
        ));
    }
    validate_measurement(&report.measurement)?;
    if report.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark scientific readiness contains no stages".to_string(),
        ));
    }

    validate_scientific_readiness_stages(&report.stages)?;
    for corpus in &report.corpora {
        require_text(&corpus.id, "scientific readiness corpus id")?;
        require_text(&corpus.truth_strategy, "scientific readiness truth strategy")?;
        validate_text_list(&corpus.roles, "scientific readiness corpus roles", false)?;
        validate_production_corpus_class(corpus.class, corpus.evidence_scope, &corpus.id)?;
    }
    validate_scientific_readiness_domains(report)?;
    validate_scientific_readiness_summary(report)
}

fn validate_scientific_readiness_stages(
    stages: &[BenchmarkScientificReadinessStage],
) -> Result<(), BenchError> {
    let mut previous_stage = None;
    for stage in stages {
        require_text(&stage.stage_id, "scientific readiness stage id")?;
        require_text(&stage.profile_id, "scientific readiness profile id")?;
        validate_text_list(&stage.retained_tools, "scientific readiness tools", false)?;
        if stage.retained_tools.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(BenchError::InvalidPolicy(
                "scientific readiness tools are not strictly sorted".to_string(),
            ));
        }
        if previous_stage.is_some_and(|previous| previous >= stage.stage_id.as_str()) {
            return Err(BenchError::InvalidPolicy(
                "scientific readiness stages are not strictly sorted".to_string(),
            ));
        }
        previous_stage = Some(stage.stage_id.as_str());
        let expected_domain = stage.stage_id.split_once('.').map(|(domain, _)| domain);
        if expected_domain != Some(stage.domain.as_str())
            || !matches!(stage.domain.as_str(), "fastq" | "bam" | "vcf")
        {
            return Err(BenchError::InvalidPolicy(
                "scientific readiness stage domain is invalid".to_string(),
            ));
        }
        validate_coverage_run_id(&stage.reserved_run_id)?;
        if let Some(run_id) = &stage.campaign_run_id {
            validate_coverage_run_id(run_id)?;
        }
        let is_ready = stage.readiness == BenchmarkScientificReadinessStatus::Ready;
        let ready_contract = stage.blockers.is_empty()
            && stage.interface_status == BenchmarkStageInterfaceStatus::Closed
            && stage.campaign_run_id.as_deref() == Some(stage.reserved_run_id.as_str())
            && stage.campaign_tier == Some(BenchmarkEvidenceTier::Comparative);
        if is_ready != ready_contract {
            return Err(BenchError::InvalidPolicy(
                "scientific readiness status disagrees with blockers or campaign closure"
                    .to_string(),
            ));
        }
        if !is_ready && stage.blockers.is_empty() {
            return Err(BenchError::InvalidPolicy(
                "blocked scientific readiness stage has no blocker".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_scientific_readiness_domains(
    report: &BenchmarkScientificReadinessReport,
) -> Result<(), BenchError> {
    let mut observed_domains = BTreeMap::new();
    for domain in ["bam", "fastq", "vcf"] {
        let stages = report.stages.iter().filter(|stage| stage.domain == domain);
        let stage_count = stages.clone().count();
        let ready_count = stages
            .clone()
            .filter(|stage| stage.readiness == BenchmarkScientificReadinessStatus::Ready)
            .count();
        observed_domains.insert(domain, (stage_count, ready_count, stage_count - ready_count));
    }
    if report.domains.len() != observed_domains.len()
        || observed_domains.iter().any(|(domain, observed)| {
            report.domains.get(*domain).is_none_or(|declared| {
                *observed != (declared.stage_count, declared.ready_count, declared.blocked_count)
            })
        })
    {
        return Err(BenchError::InvalidPolicy(
            "scientific readiness domain summaries disagree with stages".to_string(),
        ));
    }
    Ok(())
}

fn validate_scientific_readiness_summary(
    report: &BenchmarkScientificReadinessReport,
) -> Result<(), BenchError> {
    let ready_count = report
        .stages
        .iter()
        .filter(|stage| stage.readiness == BenchmarkScientificReadinessStatus::Ready)
        .count();
    let qualification_campaign_count = report
        .stages
        .iter()
        .filter(|stage| stage.campaign_tier == Some(BenchmarkEvidenceTier::Qualification))
        .count();
    let comparative_campaign_count = report
        .stages
        .iter()
        .filter(|stage| stage.campaign_tier == Some(BenchmarkEvidenceTier::Comparative))
        .count();
    let admitted_comparative_corpus_count = report
        .corpora
        .iter()
        .filter(|corpus| {
            corpus.status == BenchmarkCorpusStatus::Admitted
                && corpus.evidence_scope == BenchmarkEvidenceScope::Comparative
                && corpus.class == BenchmarkCorpusClass::ProductionReal
        })
        .count();
    let observed_summary = (
        report.stages.len(),
        report.stages.len(),
        ready_count,
        report.stages.len() - ready_count,
        report.corpora.len(),
        admitted_comparative_corpus_count,
        qualification_campaign_count,
        comparative_campaign_count,
    );
    let summary = &report.summary;
    let declared_summary = (
        summary.stage_count,
        summary.reserved_run_count,
        summary.ready_count,
        summary.blocked_count,
        summary.corpus_count,
        summary.admitted_comparative_corpus_count,
        summary.qualification_campaign_count,
        summary.comparative_campaign_count,
    );
    if observed_summary != declared_summary {
        return Err(BenchError::InvalidPolicy(
            "scientific readiness summary disagrees with report rows".to_string(),
        ));
    }
    Ok(())
}

/// Validate a complete benchmark stage-coverage publication.
///
/// # Errors
/// Returns an error when schema, ordering, identities, statuses, or summary counts disagree.
pub fn validate_stage_coverage(coverage: &BenchmarkStageCoverage) -> Result<(), BenchError> {
    if coverage.schema_version != BENCHMARK_STAGE_COVERAGE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage coverage schema mismatch".to_string(),
        ));
    }
    if coverage.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage coverage contains no retained stages".to_string(),
        ));
    }

    let mut previous_stage = None;
    for row in &coverage.stages {
        require_text(&row.stage_id, "coverage stage id")?;
        validate_text_list(&row.retained_tools, "coverage retained tools", false)?;
        if row.retained_tools.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(BenchError::InvalidPolicy(
                "benchmark coverage retained tools are not strictly sorted".to_string(),
            ));
        }
        if previous_stage.is_some_and(|previous| previous >= row.stage_id.as_str()) {
            return Err(BenchError::InvalidPolicy(
                "benchmark coverage stages are not strictly sorted".to_string(),
            ));
        }
        previous_stage = Some(row.stage_id.as_str());
        validate_stage_coverage_row(row)?;
    }

    let summary = &coverage.summary;
    let defined = coverage
        .stages
        .iter()
        .filter(|row| row.definition_status != BenchmarkStageDefinitionStatus::Missing)
        .count();
    let count_definition =
        |status| coverage.stages.iter().filter(|row| row.definition_status == status).count();
    let count_execution =
        |status| coverage.stages.iter().filter(|row| row.execution_status == status).count();
    let observed = (
        coverage.stages.len(),
        defined,
        count_definition(BenchmarkStageDefinitionStatus::Qualification),
        count_definition(BenchmarkStageDefinitionStatus::Comparative),
        count_execution(BenchmarkStageExecutionStatus::Admitted),
        count_execution(BenchmarkStageExecutionStatus::Stale),
        count_execution(BenchmarkStageExecutionStatus::Failed),
        coverage.stages.len() - defined,
    );
    let declared = (
        summary.retained_stage_count,
        summary.defined_stage_count,
        summary.qualification_stage_count,
        summary.comparative_stage_count,
        summary.admitted_stage_count,
        summary.stale_stage_count,
        summary.failed_stage_count,
        summary.missing_stage_count,
    );
    if observed != declared {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage coverage summary disagrees with stage rows".to_string(),
        ));
    }
    Ok(())
}

/// Validate the complete canonical domain-stage universe.
///
/// # Errors
/// Returns an error when schema identity, ordering, disposition rationale, monotonic run identity,
/// or summary counts disagree.
pub fn validate_stage_universe(universe: &BenchmarkStageUniverse) -> Result<(), BenchError> {
    if universe.schema_version != BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage universe schema mismatch".to_string(),
        ));
    }
    if universe.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage universe contains no stages".to_string(),
        ));
    }

    let mut previous_stage = None;
    let mut run_ids = BTreeSet::new();
    for row in &universe.stages {
        require_text(&row.stage_id, "stage universe stage id")?;
        require_text(&row.domain, "stage universe domain")?;
        require_text(&row.status, "stage universe status")?;
        require_text(&row.benchmark_disposition_reason, "stage universe disposition reason")?;
        if !matches!(row.domain.as_str(), "bam" | "fastq" | "vcf")
            || !row.stage_id.starts_with(&format!("{}.", row.domain))
        {
            return Err(BenchError::InvalidPolicy(format!(
                "stage universe domain identity disagrees for {}",
                row.stage_id
            )));
        }
        if !matches!(row.status.as_str(), "supported" | "planned" | "out_of_scope") {
            return Err(BenchError::InvalidPolicy(format!(
                "stage universe status is invalid for {}: {}",
                row.stage_id, row.status
            )));
        }
        if previous_stage.is_some_and(|previous| previous >= row.stage_id.as_str()) {
            return Err(BenchError::InvalidPolicy(
                "benchmark stage universe rows are not strictly sorted".to_string(),
            ));
        }
        previous_stage = Some(row.stage_id.as_str());
        match row.benchmark_disposition {
            BenchmarkDisposition::BenchmarkRequired => {
                let run_id = row.reserved_run_id.as_deref().ok_or_else(|| {
                    BenchError::InvalidPolicy(format!(
                        "benchmark-required stage has no reserved run id: {}",
                        row.stage_id
                    ))
                })?;
                validate_coverage_run_id(run_id)?;
                let run_number = parse_run_id(run_id).ok_or_else(|| {
                    BenchError::InvalidPolicy(format!(
                        "stage universe run id is malformed: {run_id}"
                    ))
                })?;
                if !run_ids.insert(run_number) {
                    return Err(BenchError::InvalidPolicy(format!(
                        "stage universe run id is duplicated: {run_id}"
                    )));
                }
            }
            _ if row.reserved_run_id.is_some() => {
                return Err(BenchError::InvalidPolicy(format!(
                    "non-benchmark stage reserves a run id: {}",
                    row.stage_id
                )));
            }
            _ => {}
        }
    }

    validate_monotonic_stage_run_namespace(&run_ids)?;

    let count = |disposition| {
        universe.stages.iter().filter(|row| row.benchmark_disposition == disposition).count()
    };
    let observed = (
        universe.stages.len(),
        count(BenchmarkDisposition::BenchmarkRequired),
        count(BenchmarkDisposition::QualificationOnly),
        count(BenchmarkDisposition::OrchestrationOnly),
        count(BenchmarkDisposition::TestOnly),
        count(BenchmarkDisposition::Retired),
        count(BenchmarkDisposition::Excluded),
    );
    let summary = &universe.summary;
    let declared = (
        summary.stage_count,
        summary.benchmark_required_count,
        summary.qualification_only_count,
        summary.orchestration_only_count,
        summary.test_only_count,
        summary.retired_count,
        summary.excluded_count,
    );
    if observed != declared {
        return Err(BenchError::InvalidPolicy(
            "benchmark stage universe summary disagrees with stage rows".to_string(),
        ));
    }
    Ok(())
}

fn validate_monotonic_stage_run_namespace(run_ids: &BTreeSet<u32>) -> Result<(), BenchError> {
    for (offset, run_number) in run_ids.iter().enumerate() {
        let offset = u32::try_from(offset).map_err(|_| {
            BenchError::InvalidPolicy(
                "stage universe contains too many run reservations".to_string(),
            )
        })?;
        let expected = 2001_u32.checked_add(offset).ok_or_else(|| {
            BenchError::InvalidPolicy("stage universe run reservation range overflowed".to_string())
        })?;
        if *run_number != expected {
            return Err(BenchError::InvalidPolicy(format!(
                "stage universe run reservations must be monotonic and contiguous from run-2001: expected run-{expected}, found run-{run_number}"
            )));
        }
    }
    Ok(())
}

fn validate_stage_coverage_row(row: &BenchmarkStageCoverageRow) -> Result<(), BenchError> {
    let invalid = || {
        BenchError::InvalidPolicy(
            "benchmark coverage definition, execution, and run identity disagree".to_string(),
        )
    };
    if row.definition_status == BenchmarkStageDefinitionStatus::Missing {
        if row.execution_status != BenchmarkStageExecutionStatus::NotDefined
            || row.campaign_run_id.is_some()
            || row.report_repository_commit.is_some()
            || row.report_sha256.is_some()
        {
            return Err(invalid());
        }
        return Ok(());
    }

    let run_id = row.campaign_run_id.as_deref().ok_or_else(invalid)?;
    validate_coverage_run_id(run_id)?;
    match row.execution_status {
        BenchmarkStageExecutionStatus::NotAdmitted
            if row.report_repository_commit.is_none() && row.report_sha256.is_none() => {}
        BenchmarkStageExecutionStatus::Stale
        | BenchmarkStageExecutionStatus::Failed
        | BenchmarkStageExecutionStatus::Admitted => {
            validate_coverage_commit(row.report_repository_commit.as_deref().ok_or_else(invalid)?)?;
            validate_coverage_digest(row.report_sha256.as_deref().ok_or_else(invalid)?)?;
        }
        BenchmarkStageExecutionStatus::NotDefined | BenchmarkStageExecutionStatus::NotAdmitted => {
            return Err(invalid())
        }
    }
    Ok(())
}

fn validate_coverage_commit(value: &str) -> Result<(), BenchError> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark coverage report commit identity is malformed".to_string(),
        ));
    }
    Ok(())
}

fn validate_coverage_digest(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark coverage report digest identity is malformed".to_string(),
        ));
    }
    Ok(())
}

fn validate_coverage_run_id(run_id: &str) -> Result<(), BenchError> {
    let Some(number) = run_id.strip_prefix("run-") else {
        return Err(BenchError::InvalidPolicy(
            "benchmark coverage run identity is malformed".to_string(),
        ));
    };
    if number.len() != 4
        || !number.bytes().all(|byte| byte.is_ascii_digit())
        || number.parse::<u16>().map_or(true, |value| !(2000..3000).contains(&value))
    {
        return Err(BenchError::InvalidPolicy(
            "benchmark coverage run identity is outside run-2000..2999".to_string(),
        ));
    }
    Ok(())
}

fn validate_campaign_portfolio_alignment_impl(
    registry: &BenchmarkStageCampaignRegistry,
    portfolio: &BenchmarkPortfolio,
    require_full_closure: bool,
) -> Result<(), BenchError> {
    let registry_stages = registry
        .campaigns
        .iter()
        .map(|campaign| campaign.stage_id.as_str())
        .collect::<BTreeSet<_>>();
    let portfolio_stages = portfolio.stage_runs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let missing = portfolio_stages.difference(&registry_stages).copied().collect::<Vec<_>>();
    let extra = registry_stages.difference(&portfolio_stages).copied().collect::<Vec<_>>();
    if require_full_closure && (!missing.is_empty() || !extra.is_empty()) {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark campaign/portfolio closure failed: missing={missing:?}, extra={extra:?}"
        )));
    }

    let assigned_profiles = portfolio
        .assignments
        .iter()
        .flat_map(|assignment| {
            assignment
                .stage_ids
                .iter()
                .map(move |stage_id| (stage_id.as_str(), assignment.profile_id.as_str()))
        })
        .collect::<BTreeMap<_, _>>();
    let declared_profiles =
        portfolio.profiles.iter().map(|profile| profile.id.as_str()).collect::<BTreeSet<_>>();
    let corpora = portfolio
        .corpora
        .iter()
        .map(|corpus| (corpus.id.as_str(), corpus))
        .collect::<BTreeMap<_, _>>();

    for campaign in &registry.campaigns {
        let expected_run_id = portfolio.stage_runs.get(&campaign.stage_id).ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "benchmark campaign stage is absent from portfolio: {}",
                campaign.stage_id
            ))
        })?;
        if expected_run_id != &campaign.run_id {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark run id drift for {}: registry={}, portfolio={expected_run_id}",
                campaign.stage_id, campaign.run_id
            )));
        }
        if !declared_profiles.contains(campaign.profile_id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark campaign references unknown profile: {}",
                campaign.profile_id
            )));
        }
        let assigned_profile =
            assigned_profiles.get(campaign.stage_id.as_str()).ok_or_else(|| {
                BenchError::InvalidPolicy(format!(
                    "benchmark campaign stage is not assigned to a portfolio profile: {}",
                    campaign.stage_id
                ))
            })?;
        if assigned_profile != &campaign.profile_id.as_str() {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark profile drift for {}: campaign={}, assignment={assigned_profile}",
                campaign.stage_id, campaign.profile_id
            )));
        }
        if campaign.warmup_repetitions != portfolio.measurement.warmup_repetitions {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark warmup repetitions drift for {}",
                campaign.stage_id
            )));
        }
        if campaign.measured_repetitions != portfolio.measurement.measured_repetitions {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark measured repetitions drift for {}",
                campaign.stage_id
            )));
        }
        for corpus_id in &campaign.corpus_ids {
            if !corpora.contains_key(corpus_id.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "benchmark campaign references unknown corpus: {corpus_id}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_stage_campaign(campaign: &BenchmarkStageCampaignSpec) -> Result<(), BenchError> {
    validate_stage_campaign_identity(campaign)?;
    validate_stage_campaign_execution(campaign)?;
    validate_stage_campaign_metrics(campaign)?;
    validate_thread_configuration(campaign)
}

fn validate_stage_campaign_identity(
    campaign: &BenchmarkStageCampaignSpec,
) -> Result<(), BenchError> {
    require_text(&campaign.run_id, "campaign run_id")?;
    require_text(&campaign.slug, "campaign slug")?;
    require_text(&campaign.domain, "campaign domain")?;
    require_text(&campaign.stage_id, "campaign stage_id")?;
    require_text(&campaign.profile_id, "campaign profile_id")?;
    require_text(&campaign.scientific_question, "campaign scientific_question")?;
    require_text(&campaign.comparison_scope, "campaign comparison_scope")?;
    require_text(&campaign.accuracy_claim, "campaign accuracy_claim")?;
    require_text(&campaign.interpretation, "campaign interpretation")?;

    if !is_stage_campaign_run_id(&campaign.run_id) {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign run id is outside run-2001+: {}",
            campaign.run_id
        )));
    }
    let expected_slug = derive_slug(&campaign.stage_id);
    if campaign.slug != expected_slug {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign slug {:?} must derive from stage {:?}",
            campaign.slug, campaign.stage_id
        )));
    }
    if campaign.material_driver.is_some_and(|driver| driver.stage_id() != campaign.stage_id) {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign material driver {:?} does not implement {}",
            campaign.material_driver, campaign.stage_id
        )));
    }

    validate_text_list(&campaign.corpus_ids, "campaign corpus_ids", false)?;
    validate_text_list(&campaign.tool_ids, "campaign tool_ids", false)?;
    validate_text_list(&campaign.corpus_catalog_ids, "campaign corpus_catalog_ids", false)?;
    validate_text_list(&campaign.reference_catalog_ids, "campaign reference_catalog_ids", true)?;
    validate_text_list(&campaign.database_catalog_ids, "campaign database_catalog_ids", true)?;
    validate_text_list(&campaign.input_size_keys, "campaign input_size_keys", false)?;
    validate_text_list(&campaign.performance_metrics, "campaign performance_metrics", false)?;
    validate_text_list(&campaign.scientific_metrics, "campaign scientific_metrics", false)?;
    validate_text_list(&campaign.limitations, "campaign limitations", false)?;
    Ok(())
}

fn validate_stage_campaign_execution(
    campaign: &BenchmarkStageCampaignSpec,
) -> Result<(), BenchError> {
    if campaign.input_size_keys.iter().any(|value| !is_input_size_key(value)) {
        return Err(BenchError::InvalidPolicy(
            "campaign input_size_keys must name governed byte-count fields".to_string(),
        ));
    }
    if !is_safe_filename(&campaign.result_artifact) {
        return Err(BenchError::InvalidPolicy(
            "campaign result_artifact must be a safe filename".to_string(),
        ));
    }
    if !campaign.container_only {
        return Err(BenchError::InvalidPolicy(
            "stage campaigns must execute container-only tools".to_string(),
        ));
    }
    if campaign.warmup_repetitions < 1 {
        return Err(BenchError::InvalidPolicy(
            "stage campaigns require at least one warmup repetition".to_string(),
        ));
    }
    if campaign.measured_repetitions < 3 {
        return Err(BenchError::InvalidPolicy(
            "stage campaigns require at least three measurements".to_string(),
        ));
    }
    if campaign.result_root_template != CANONICAL_RESULT_ROOT_TEMPLATE {
        return Err(BenchError::InvalidPolicy(
            "stage campaign result root template is not canonical".to_string(),
        ));
    }
    Ok(())
}

fn validate_stage_campaign_metrics(
    campaign: &BenchmarkStageCampaignSpec,
) -> Result<(), BenchError> {
    let performance_metrics =
        campaign.performance_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing_required = REQUIRED_PERFORMANCE_METRICS
        .iter()
        .copied()
        .filter(|metric| !performance_metrics.contains(metric))
        .collect::<Vec<_>>();
    if !missing_required.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign performance metrics are incomplete for {}: {:?}",
            campaign.stage_id, missing_required
        )));
    }
    if matches!(campaign.evidence_tier, BenchmarkEvidenceTier::Comparative) {
        let missing_comparative = COMPARATIVE_PERFORMANCE_METRICS
            .iter()
            .copied()
            .filter(|metric| !performance_metrics.contains(metric))
            .collect::<Vec<_>>();
        if !missing_comparative.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "comparative performance metrics are incomplete for {}: {:?}",
                campaign.stage_id, missing_comparative
            )));
        }
    }

    let scientific_metrics =
        campaign.scientific_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if campaign.tool_ids.len() > 1 && campaign.comparison_metrics.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "multi-tool stage campaigns require comparison metrics".to_string(),
        ));
    }
    let mut seen_comparison_ids = BTreeSet::new();
    for comparison in &campaign.comparison_metrics {
        require_text(&comparison.id, "campaign comparison metric id")?;
        if !scientific_metrics.contains(comparison.id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "comparison metric must reference a scientific metric: {}",
                comparison.id
            )));
        }
        if !seen_comparison_ids.insert(comparison.id.as_str()) {
            return Err(BenchError::InvalidPolicy(format!(
                "comparison metric id is duplicated: {}",
                comparison.id
            )));
        }
        if !comparison.absolute_tolerance.is_finite() || comparison.absolute_tolerance < 0.0 {
            return Err(BenchError::InvalidPolicy(format!(
                "comparison absolute_tolerance must be non-negative: {}",
                comparison.id
            )));
        }
        if !comparison.relative_tolerance.is_finite() || comparison.relative_tolerance < 0.0 {
            return Err(BenchError::InvalidPolicy(format!(
                "comparison relative_tolerance must be non-negative: {}",
                comparison.id
            )));
        }
    }

    Ok(())
}

fn validate_measurement(measurement: &BenchmarkMeasurementContract) -> Result<(), BenchError> {
    if measurement.warmup_repetitions < 1 {
        return Err(BenchError::InvalidPolicy(
            "benchmark measurement requires at least one warmup".to_string(),
        ));
    }
    if measurement.measured_repetitions < 3 {
        return Err(BenchError::InvalidPolicy(
            "benchmark measurement requires at least three repetitions".to_string(),
        ));
    }
    validate_text_list(
        &measurement.required_resource_metrics,
        "measurement required_resource_metrics",
        false,
    )?;
    validate_text_list(
        &measurement.required_quality_dimensions,
        "measurement required_quality_dimensions",
        false,
    )?;

    let resources =
        measurement.required_resource_metrics.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing_resources = REQUIRED_RESOURCE_METRICS
        .iter()
        .copied()
        .filter(|metric| !resources.contains(metric))
        .collect::<Vec<_>>();
    if !missing_resources.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark measurement omits required resource metrics: {missing_resources:?}"
        )));
    }

    let quality =
        measurement.required_quality_dimensions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing_quality = REQUIRED_QUALITY_DIMENSIONS
        .iter()
        .copied()
        .filter(|dimension| !quality.contains(dimension))
        .collect::<Vec<_>>();
    if !missing_quality.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark measurement omits required quality dimensions: {missing_quality:?}"
        )));
    }
    Ok(())
}

fn validate_corpora(
    corpora: &[BenchmarkCorpusContract],
) -> Result<(BTreeSet<String>, BTreeSet<String>), BenchError> {
    if corpora.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark portfolio contains no corpora".to_string(),
        ));
    }
    let mut corpus_ids = BTreeSet::new();
    let mut role_ids = BTreeSet::new();
    for corpus in corpora {
        require_text(&corpus.id, "corpus id")?;
        if !corpus_ids.insert(corpus.id.clone()) {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark corpus is duplicated: {}",
                corpus.id
            )));
        }
        validate_production_corpus_class(corpus.class, corpus.evidence_scope, &corpus.id)?;
        let minimum_sample_count = corpus.minimum_sample_count.unwrap_or(corpus.sample_count);
        if minimum_sample_count < 1 {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark corpus minimum sample count is invalid: {}",
                corpus.id
            )));
        }
        if matches!(corpus.status, BenchmarkCorpusStatus::Admitted)
            && corpus.sample_count < minimum_sample_count
        {
            return Err(BenchError::InvalidPolicy(format!(
                "admitted corpus is below its minimum size: {}",
                corpus.id
            )));
        }
        validate_text_list(&corpus.data_types, "corpus data_types", false)?;
        validate_text_list(&corpus.roles, "corpus roles", false)?;
        validate_text_list(&corpus.source_specs, "corpus source_specs", true)?;
        validate_text_list(&corpus.catalog_ids, "corpus catalog_ids", true)?;
        require_text(&corpus.truth_strategy, "corpus truth_strategy")?;
        require_text(&corpus.scientific_rationale, "corpus scientific_rationale")?;
        validate_text_list(&corpus.limitations, "corpus limitations", false)?;
        if matches!(corpus.status, BenchmarkCorpusStatus::SourceDefined)
            && corpus.source_specs.is_empty()
        {
            return Err(BenchError::InvalidPolicy(format!(
                "source-defined corpus has no source_specs: {}",
                corpus.id
            )));
        }
        if matches!(corpus.status, BenchmarkCorpusStatus::Admitted) && corpus.catalog_ids.is_empty()
        {
            return Err(BenchError::InvalidPolicy(format!(
                "admitted corpus has no catalog_ids: {}",
                corpus.id
            )));
        }
        role_ids.extend(corpus.roles.iter().cloned());
    }
    Ok((corpus_ids, role_ids))
}

fn validate_production_corpus_class(
    class: BenchmarkCorpusClass,
    scope: BenchmarkEvidenceScope,
    id: &str,
) -> Result<(), BenchError> {
    let valid = matches!(
        (class, scope),
        (BenchmarkCorpusClass::ProductionReal, BenchmarkEvidenceScope::Comparative)
            | (BenchmarkCorpusClass::QualificationReal, BenchmarkEvidenceScope::Qualification)
    );
    if !valid {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark production corpus class and evidence scope disagree: {id}"
        )));
    }
    Ok(())
}

fn validate_profiles(
    profiles: &[BenchmarkScientificProfile],
    known_roles: &BTreeSet<String>,
) -> Result<BTreeSet<String>, BenchError> {
    if profiles.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark portfolio contains no scientific profiles".to_string(),
        ));
    }
    let mut profile_ids = BTreeSet::new();
    for profile in profiles {
        require_text(&profile.id, "profile id")?;
        if !profile_ids.insert(profile.id.clone()) {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark profile is duplicated: {}",
                profile.id
            )));
        }
        validate_text_list(&profile.required_corpus_roles, "profile required_corpus_roles", false)?;
        let unknown_roles = profile
            .required_corpus_roles
            .iter()
            .filter(|role| !known_roles.contains(*role))
            .collect::<Vec<_>>();
        if !unknown_roles.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark profile has unknown corpus roles: {unknown_roles:?}"
            )));
        }
        if profile.minimum_real_samples < 10 {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark profile requires at least ten real samples: {}",
                profile.id
            )));
        }
        require_text(&profile.accuracy_strategy, "profile accuracy_strategy")?;
        require_text(&profile.interchangeability, "profile interchangeability")?;
        require_text(&profile.downstream_probe, "profile downstream_probe")?;
    }
    Ok(profile_ids)
}

fn validate_assignments(
    assignments: &[BenchmarkProfileAssignment],
    profile_ids: &BTreeSet<String>,
    stage_runs: &BTreeMap<String, String>,
    corpus_ids: &BTreeSet<String>,
) -> Result<(), BenchError> {
    if assignments.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark portfolio contains no stage assignments".to_string(),
        ));
    }
    let mut assigned_stages = BTreeMap::new();
    for assignment in assignments {
        require_text(&assignment.profile_id, "assignment profile_id")?;
        if !profile_ids.contains(&assignment.profile_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "assignment references unknown profile: {}",
                assignment.profile_id
            )));
        }
        validate_text_list(&assignment.stage_ids, "assignment stage_ids", false)?;
        for stage_id in &assignment.stage_ids {
            if assigned_stages.insert(stage_id.as_str(), assignment.profile_id.as_str()).is_some() {
                return Err(BenchError::InvalidPolicy(format!(
                    "benchmark stage has multiple profile assignments: {stage_id}"
                )));
            }
        }
    }

    let declared_stages = stage_runs.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let assigned = assigned_stages.keys().copied().collect::<BTreeSet<_>>();
    let missing = declared_stages.difference(&assigned).copied().collect::<Vec<_>>();
    let extra = assigned.difference(&declared_stages).copied().collect::<Vec<_>>();
    if !missing.is_empty() || !extra.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "benchmark profile closure failed: missing={missing:?}, extra={extra:?}"
        )));
    }

    if corpus_ids.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "benchmark portfolio must declare at least one corpus".to_string(),
        ));
    }
    Ok(())
}

fn validate_thread_configuration(campaign: &BenchmarkStageCampaignSpec) -> Result<(), BenchError> {
    let tool_ids = campaign.tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if campaign.threads_by_tool.is_empty() {
        let Some(threads) = campaign.threads else {
            return Err(BenchError::InvalidPolicy(
                "stage campaign must declare threads or threads_by_tool".to_string(),
            ));
        };
        if threads < 1 {
            return Err(BenchError::InvalidPolicy(
                "stage campaign threads must be positive".to_string(),
            ));
        }
        return Ok(());
    }
    if campaign.threads.is_some() {
        return Err(BenchError::InvalidPolicy(
            "stage campaign cannot mix threads and threads_by_tool".to_string(),
        ));
    }
    let configured_tools =
        campaign.threads_by_tool.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if configured_tools != tool_ids {
        return Err(BenchError::InvalidPolicy(format!(
            "stage campaign threads_by_tool must cover the exact tool closure for {}",
            campaign.stage_id
        )));
    }
    if campaign.threads_by_tool.values().any(|threads| *threads < 1) {
        return Err(BenchError::InvalidPolicy(
            "stage campaign thread counts must be positive".to_string(),
        ));
    }
    Ok(())
}

fn require_text(value: &str, label: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(BenchError::InvalidPolicy(format!("{label} must be non-empty text")));
    }
    Ok(())
}

fn validate_text_list(values: &[String], label: &str, allow_empty: bool) -> Result<(), BenchError> {
    if values.is_empty() && !allow_empty {
        return Err(BenchError::InvalidPolicy(format!("{label} must be a non-empty text list")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, label)?;
        if !seen.insert(value.as_str()) {
            return Err(BenchError::InvalidPolicy(format!("{label} contains duplicates")));
        }
    }
    Ok(())
}

fn is_stage_campaign_run_id(run_id: &str) -> bool {
    parse_run_id(run_id).is_some_and(|number| number >= 2000)
}

fn is_portfolio_run_id(run_id: &str) -> bool {
    parse_run_id(run_id).is_some_and(|number| (2000..3000).contains(&number))
}

fn parse_run_id(run_id: &str) -> Option<u32> {
    let suffix = run_id.strip_prefix("run-")?;
    if suffix.len() < 4 || !suffix.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    suffix.parse::<u32>().ok()
}

fn derive_slug(stage_id: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in stage_id.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn is_input_size_key(value: &str) -> bool {
    let Some(prefix) = value.strip_suffix("_input_bytes") else {
        return false;
    };
    let mut chars = prefix.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn is_safe_filename(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '.' | '_' | '-')
        })
}
