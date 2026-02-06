//! Public front door for the Bijux API.

pub use crate::args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, PlanRequest, PlanResponse,
    RunStatus,
};
pub use crate::explain::{ExplainResponse, ExplainToolSelection, PlanExplainV1};
pub use crate::run::{dry_run, execute, execute_and_report, plan, policy_audit, status};

/// Build an explainability bundle for a planned graph.
#[must_use]
pub fn explain(
    plan: &bijux_core::contract::ExecutionGraph,
    defaults_ledger: Option<&serde_json::Value>,
) -> ExplainResponse {
    crate::explain::explain_bundle(plan, defaults_ledger)
}
