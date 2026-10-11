use super::values::{validate_https, validate_nonempty, validate_nonempty_list};
use super::{invalid, CorpusSourceError};
use crate::source::{BenchmarkCorpusClass, CorpusSourcePolicy};

/// Validate canonical corpus source semantics.
///
/// # Errors
/// Rejects missing, malformed or inconsistent source declarations.
pub fn validate_source_policy(policy: &CorpusSourcePolicy) -> Result<(), CorpusSourceError> {
    if !matches!(
        policy.class,
        BenchmarkCorpusClass::ProductionReal | BenchmarkCorpusClass::QualificationReal
    ) {
        return Err(invalid("corpus source policy class is not publishable real data"));
    }
    for (value, label) in [
        (&policy.access_classification, "access classification"),
        (&policy.redistribution_classification, "redistribution classification"),
        (&policy.attribution, "attribution"),
        (&policy.projection_basis, "projection basis"),
        (&policy.truth_strategy, "truth strategy"),
    ] {
        validate_nonempty(value, label)?;
    }
    validate_https(&policy.access_terms_url, "access terms")?;
    if policy.projected_storage_bytes == 0 || policy.projected_cpu_hours == 0 {
        return Err(invalid("corpus source storage and compute projections must be positive"));
    }
    validate_nonempty_list(&policy.truth_roles, "truth role")?;
    validate_nonempty_list(&policy.known_limitations, "known limitation")?;
    Ok(())
}
