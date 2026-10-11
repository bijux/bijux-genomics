/// A stage declaration or persisted record violates its semantic contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageContractError {
    reason: String,
}
impl StageContractError {
    pub(crate) fn invalid(reason: String) -> Self {
        Self { reason }
    }
    /// Retain the canonical diagnostic in compatibility error adapters.
    #[must_use]
    pub fn into_reason(self) -> String {
        self.reason
    }
}
impl std::fmt::Display for StageContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid policy: {}", self.reason)
    }
}
impl std::error::Error for StageContractError {}
