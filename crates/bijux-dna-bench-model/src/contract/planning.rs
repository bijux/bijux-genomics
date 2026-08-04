//! Owner: bijux-dna-bench-model
//! Semantic validation for benchmark planning contracts.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkCorpusContract, BenchmarkCorpusStatus, BenchmarkEvidenceTier,
    BenchmarkMeasurementContract, BenchmarkPortfolio, BenchmarkProfileAssignment,
    BenchmarkScientificProfile, BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
    BenchmarkStageCoverage, BenchmarkStageCoverageRow, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus,
};

use super::{
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
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

/// # Errors
/// Returns an error if the portfolio shape or invariants are invalid.
pub fn validate_benchmark_portfolio(portfolio: &BenchmarkPortfolio) -> Result<(), BenchError> {
    if portfolio.schema_version != BENCHMARK_PORTFOLIO_SCHEMA_V1 {
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
        if !corpus.real_data {
            return Err(BenchError::InvalidPolicy(format!(
                "benchmark corpus must declare real data: {}",
                corpus.id
            )));
        }
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
