use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct BindingKey {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RetainedVcfBinding {
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) tool_statuses: BTreeSet<String>,
}
