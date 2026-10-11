//! Owner: bijux-dna-bench-model
//! Validation for the machine-evaluable benchmark data-fitness rubric.

use std::collections::BTreeSet;

use crate::model::{
    BenchmarkDataFitnessProfile, BenchmarkDataFitnessRubric, BenchmarkFitnessExceptionDimension,
    BenchmarkPortfolio, BenchmarkTruthEvidenceRequirement,
};
use crate::BenchError;

use super::BENCHMARK_DATA_FITNESS_SCHEMA_V1;

/// Validate completeness, dimensional coverage, and scientific exception discipline.
///
/// # Errors
/// Returns [`BenchError`] when the rubric can admit incomplete evidence or an unjustified minimum.
pub fn validate_data_fitness_rubric(rubric: &BenchmarkDataFitnessRubric) -> Result<(), BenchError> {
    if rubric.schema_version != BENCHMARK_DATA_FITNESS_SCHEMA_V1 {
        return invalid("unsupported data-fitness schema");
    }
    let defaults = &rubric.defaults;
    if defaults.minimum_real_samples == 0
        || defaults.minimum_cases_per_tool == 0
        || defaults.minimum_distinct_studies == 0
        || defaults.minimum_input_bytes == 0
    {
        return invalid("data-fitness defaults must be positive");
    }
    let required_exception_fields = [
        "dimension",
        "default_minimum",
        "permitted_minimum",
        "scientific_rationale",
        "compensating_evidence",
        "allowed_claims",
    ];
    let fields =
        rubric.exception_policy.required_fields.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if !required_exception_fields.iter().all(|field| fields.contains(field))
        || rubric.exception_policy.review_authority.trim().is_empty()
        || !rubric.exception_policy.claim_restriction_required
    {
        return invalid("data-fitness exception policy is incomplete");
    }
    let mut profile_ids = BTreeSet::new();
    for profile in &rubric.profiles {
        if profile.profile_id.trim().is_empty() || !profile_ids.insert(profile.profile_id.as_str())
        {
            return invalid("duplicate or empty data-fitness profile id");
        }
        validate_profile(rubric, profile)?;
    }
    if rubric.profiles.is_empty() {
        return invalid("data-fitness rubric has no profiles");
    }
    Ok(())
}

/// Require one fitness profile for every benchmark scientific profile and reject stale extras.
///
/// # Errors
/// Returns [`BenchError`] when the rubric and portfolio profile universes differ.
pub fn validate_data_fitness_portfolio_alignment(
    rubric: &BenchmarkDataFitnessRubric,
    portfolio: &BenchmarkPortfolio,
) -> Result<(), BenchError> {
    validate_data_fitness_rubric(rubric)?;
    let rubric_ids =
        rubric.profiles.iter().map(|profile| profile.profile_id.as_str()).collect::<BTreeSet<_>>();
    let portfolio_ids =
        portfolio.profiles.iter().map(|profile| profile.id.as_str()).collect::<BTreeSet<_>>();
    if rubric_ids != portfolio_ids {
        return invalid("data-fitness and portfolio profile universes differ");
    }
    Ok(())
}

fn validate_profile(
    rubric: &BenchmarkDataFitnessRubric,
    profile: &BenchmarkDataFitnessProfile,
) -> Result<(), BenchError> {
    if profile.minimum_real_samples == 0
        || profile.minimum_cases_per_tool == 0
        || profile.minimum_distinct_studies == 0
        || profile.minimum_input_bytes == 0
        || profile.representativeness_strata.is_empty()
        || profile.required_layouts.is_empty()
        || profile.required_controls.is_empty()
        || profile.bias_assessments.is_empty()
        || profile.required_limitations.is_empty()
        || profile.allowed_claims.is_empty()
    {
        return invalid("data-fitness profile leaves a required dimension empty");
    }
    let access = &profile.access;
    if !access.stable_source_identity
        || !access.license_identity
        || !access.redistribution_classification
        || !access.checksum_before_execution
    {
        return invalid("data-fitness access and identity controls must all be required");
    }
    if profile.truth_requirement == BenchmarkTruthEvidenceRequirement::ObservationalPerformanceOnly
        && profile
            .allowed_claims
            .iter()
            .any(|claim| claim != "performance" && claim != "consistency")
    {
        return invalid("observational-only profiles may claim only performance and consistency");
    }
    validate_minimum(
        profile,
        BenchmarkFitnessExceptionDimension::MinimumRealSamples,
        u64::from(rubric.defaults.minimum_real_samples),
        u64::from(profile.minimum_real_samples),
    )?;
    validate_minimum(
        profile,
        BenchmarkFitnessExceptionDimension::MinimumCasesPerTool,
        u64::from(rubric.defaults.minimum_cases_per_tool),
        u64::from(profile.minimum_cases_per_tool),
    )?;
    validate_minimum(
        profile,
        BenchmarkFitnessExceptionDimension::MinimumDistinctStudies,
        u64::from(rubric.defaults.minimum_distinct_studies),
        u64::from(profile.minimum_distinct_studies),
    )?;
    validate_minimum(
        profile,
        BenchmarkFitnessExceptionDimension::MinimumInputBytes,
        rubric.defaults.minimum_input_bytes,
        profile.minimum_input_bytes,
    )
}

fn validate_minimum(
    profile: &BenchmarkDataFitnessProfile,
    dimension: BenchmarkFitnessExceptionDimension,
    default_minimum: u64,
    observed_minimum: u64,
) -> Result<(), BenchError> {
    let exceptions = profile
        .scientific_exceptions
        .iter()
        .filter(|exception| exception.dimension == dimension)
        .collect::<Vec<_>>();
    if observed_minimum >= default_minimum {
        if !exceptions.is_empty() {
            return invalid("data-fitness exception is declared without a reduced minimum");
        }
        return Ok(());
    }
    if exceptions.len() != 1 {
        return invalid("reduced data-fitness minimum lacks exactly one scientific exception");
    }
    let exception = exceptions[0];
    if exception.default_minimum != default_minimum
        || exception.permitted_minimum != observed_minimum
        || exception.scientific_rationale.trim().is_empty()
        || exception.compensating_evidence.is_empty()
        || exception.allowed_claims.is_empty()
        || exception.allowed_claims.iter().any(|claim| !profile.allowed_claims.contains(claim))
    {
        return invalid("data-fitness scientific exception is incoherent");
    }
    Ok(())
}

fn invalid<T>(message: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.to_string()))
}
