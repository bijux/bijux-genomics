pub(crate) const PRODUCTION_READINESS: &str = "production-readiness";

pub(crate) fn is_production_readiness(scope: &str) -> bool {
    scope.eq_ignore_ascii_case(PRODUCTION_READINESS)
}
