//! Typed admission and publication for selected container run evidence.

mod model;
mod schema;
mod validation;

pub use model::{
    AttemptExecutionStatus, RunEvidenceIndex, RunEvidenceRecord, RunEvidenceSummary,
    ScientificAcceptanceStatus, CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2,
};
pub use schema::{run_evidence_index_schema, write_run_evidence_index_schema};
pub use validation::{load_run_evidence_index, validate_run_evidence_index};
