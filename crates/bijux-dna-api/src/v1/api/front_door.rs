//! Public front door for the Bijux API.

pub use crate::runtime::run::{
    browse_runs, cache_explain, cancel_run, dry_run, execute, execute_and_report, operator_health,
    pause_run, plan, policy_audit, query_run_lineage, render_report, replay_explain, resume_run,
    status, workspace_edges, write_workspace_audit, evidence_gap, operator_diagnosis,
    render_operator_diagnosis_output, render_run_browser_output,
};
pub use crate::surface::explain::{ExplainResponse, ExplainToolSelection, PlanExplainV1};
pub use crate::surface::request_contracts::{
    CacheExplainRequestV1, CacheExplainResponseV1, CacheKeyFingerprintV1, CacheMissReasonV1,
    DryRunRequest, DryRunResponse, EvidenceCheckFailureV1, EvidenceGapRequestV1,
    EvidenceGapResponseV1, ExecuteRequest, ExecuteResponse, PlanRequest, PlanResponse,
    OperatorDiagnosisCommandV1, OperatorDiagnosisRequestV1, OperatorDiagnosisResponseV1,
    OperatorHealthResponse, OutputFormatV1, ReplayExplainRequestV1, ReplayExplainResponseV1,
    RedactionProfileV1, RenderReportRequest, RenderReportResult, RunBrowserFilterV1,
    RunBrowserRequestV1, RunBrowserResponseV1, RunBrowserRowV1, RunControlResponse,
    RunLineageEdgeV1, RunLineageQueryRequestV1, RunLineageQueryResponseV1, RunStatus,
};
pub use crate::surface::versioning::{route_version_inventory, ApiRouteVersionInventoryV1};
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

/// VCF-specific helpers (v1).
pub mod vcf {
    pub use crate::v1::vcf::*;
}

/// Shared helpers (v1).
pub mod shared {
    pub use crate::v1::shared::*;
}

/// Build an explainability bundle for a planned graph.
#[must_use]
pub fn explain(
    plan: &bijux_dna_core::contract::ExecutionGraph,
    defaults_ledger: Option<&serde_json::Value>,
) -> ExplainResponse {
    crate::surface::explain::explain_bundle(plan, defaults_ledger)
}
