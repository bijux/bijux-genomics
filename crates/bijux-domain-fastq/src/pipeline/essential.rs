#[must_use]
pub fn essential_stages() -> Vec<&'static str> {
    crate::domain::CANONICAL_STAGE_ORDER.to_vec()
}
