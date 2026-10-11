//! Typed admission and publication for selected container run evidence.

mod model;
mod report;
mod result_layout;
mod schema;
mod validation;
mod workflow;

pub use model::{
    ArchivedResultCategory, AttemptExecutionStatus, ContainerAttemptIdentity, FinalRunEvidencePlan,
    ResultFileIdentity, ResultLayoutMove, ResultLayoutPlan, ResultLayoutPlanRequest,
    ResultMovementState, RunAttemptLedger, RunAttemptLedgerRow, RunAttemptLedgerSummary,
    RunEvidenceFileIdentity, RunEvidenceImportRequest, RunEvidenceIndex, RunEvidenceLock,
    RunEvidenceRecord, RunEvidenceSummary, ScientificAcceptanceStatus, SupersededResultArchivePlan,
    SupersededResultArchivePlanRequest, SupersededResultMove,
    CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1, CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1,
    CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2, CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1,
    CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1,
};
pub use report::render_run_evidence_markdown;
pub use result_layout::{
    apply_result_layout, apply_superseded_result_archive, derive_result_layout_plan,
    derive_superseded_result_archive_plan, rollback_result_layout,
    rollback_superseded_result_archive, verify_result_layout, verify_superseded_result_archive,
};
pub use schema::{
    container_attempt_layout_schema, result_layout_plan_schema, run_evidence_index_schema,
    superseded_result_archive_plan_schema, write_result_movement_schemas,
    write_run_evidence_index_schema,
};
pub use validation::{
    load_run_evidence_index, load_run_evidence_lock, validate_run_attempt_ledger,
    validate_run_evidence_index, validate_run_evidence_root,
};
pub use workflow::{
    apply_final_run_evidence_plan, derive_run_evidence_index, ensure_container_attempt_layout,
    import_run_evidence, plan_final_run_evidence, publish_run_evidence_book,
    replay_run_evidence_book, verify_run_evidence_publication,
};
