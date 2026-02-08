//! Public front door for the Bijux API.

pub use crate::explain::{ExplainResponse, ExplainToolSelection, PlanExplainV1};
pub use crate::request_args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, PlanRequest, PlanResponse,
    RenderReportRequest, RenderReportResult, RunStatus,
};
pub use crate::run::{
    dry_run, execute, execute_and_report, plan, policy_audit, render_report, status,
    workspace_edges, write_workspace_audit,
};
pub use crate::v1::report::render_report_bundle_html;

/// Benchmarking helpers (v1).
pub mod bench {
    pub use crate::v1::bench::*;
}

/// Planning helpers (v1).
pub mod plan {
    pub use crate::v1::plan::*;
}

/// Run orchestration helpers (v1).
pub mod run {
    pub use crate::v1::run::*;
}

/// Report helpers (v1).
pub mod report {
    pub use crate::v1::report::*;
}

/// BAM-specific helpers (v1).
pub mod bam {
    pub use crate::v1::bam::*;
}

/// FASTQ-specific helpers (v1).
pub mod fastq {
    pub use crate::v1::fastq::*;
}

/// Environment helpers (v1).
pub mod env {
    pub use crate::v1::env::*;
}

/// Build an explainability bundle for a planned graph.
#[must_use]
pub fn explain(
    plan: &bijux_core::contract::ExecutionGraph,
    defaults_ledger: Option<&serde_json::Value>,
) -> ExplainResponse {
    crate::explain::explain_bundle(plan, defaults_ledger)
}
