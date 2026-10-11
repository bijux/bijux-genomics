//! Owner: decoded evidence identities supplied to pure movement admission.
use super::super::ResultFileIdentity;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct LayoutEvidenceLock {
    pub(super) run_id: String,
    pub(super) tool: String,
    pub(super) source_result: String,
    pub(super) source_files: Vec<ResultFileIdentity>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AttemptLedgerView {
    pub(super) schema_version: String,
    pub(super) attempts: Vec<AttemptLedgerRowView>,
}

#[derive(Debug, Deserialize)]
pub(super) struct AttemptLedgerRowView {
    pub(super) run_id: String,
    pub(super) tool: String,
    pub(super) evidence_lock_sha256: String,
    pub(super) superseded_by: String,
}
