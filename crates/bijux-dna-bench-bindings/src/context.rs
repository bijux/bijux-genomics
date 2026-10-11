use crate::ports::{ExpectedEvidence, ServingEvidence};
use std::path::Path;
/// A serving request has an explicit root and request-local canonical provider.
pub struct ServingContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn ServingEvidence,
}
/// Result layout values are supplied by the canonical scheduler path authority.
pub struct ExpectedContext<'a> {
    pub repo_root: &'a Path,
    pub evidence: &'a dyn ExpectedEvidence,
    pub result_root: &'a str,
    pub run_id: &'a str,
}
