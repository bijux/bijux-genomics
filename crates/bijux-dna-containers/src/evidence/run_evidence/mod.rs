//! Typed admission and publication for selected container run evidence.

mod model;
mod report;
mod schema;
mod validation;
mod workflow;

pub use model::{
    AttemptExecutionStatus, FinalRunEvidencePlan, RunAttemptLedger, RunAttemptLedgerRow,
    RunAttemptLedgerSummary, RunEvidenceFileIdentity, RunEvidenceImportRequest, RunEvidenceIndex,
    RunEvidenceLock, RunEvidenceRecord, RunEvidenceSummary, ScientificAcceptanceStatus,
    CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1, CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2,
    CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1,
};
pub use report::render_run_evidence_markdown;
pub use schema::{run_evidence_index_schema, write_run_evidence_index_schema};
pub use validation::{
    load_run_evidence_index, validate_run_attempt_ledger, validate_run_evidence_index,
    validate_run_evidence_root,
};
pub use workflow::{
    apply_final_run_evidence_plan, derive_run_evidence_index, import_run_evidence,
    plan_final_run_evidence, verify_run_evidence_publication,
};
