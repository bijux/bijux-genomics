//! Persisted panel contracts contracts and declared identities.

pub const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPanelWorkflowSmokeContract {
    pub(crate) stage: String,
    pub(crate) tool: String,
    pub(crate) corpus: String,
    pub(crate) panel: String,
    pub(crate) map: String,
}
