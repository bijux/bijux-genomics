/// Canonical binding identity; multiplicity is retained until owner set admission.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingObservation {
    pub stage_id: String,
    pub tool_id: String,
}
/// Expected-result rows retain their domain until the BAM slice is selected.
#[derive(Debug, Clone)]
pub struct ExpectedBindingObservation {
    pub stage_id: String,
    pub tool_id: String,
    pub domain: String,
}
