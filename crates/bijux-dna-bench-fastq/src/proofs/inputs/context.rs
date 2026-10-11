use super::ports::{DeclarationEvidence, SchemaEvidence, ValidationEvidence};
use std::path::Path;
/// Canonical evidence addresses are supplied by application composition.
pub struct ProofPaths<'a> {
    pub expected_results: &'a str,
    pub command_argv: &'a str,
    pub comparable_metrics: &'a str,
}
/// Shared declaration and schema sources for trim, filter and duplicate proofs.
pub struct SchemaContext<'a> {
    pub repo_root: &'a Path,
    pub declarations: &'a dyn DeclarationEvidence,
    pub schema: &'a dyn SchemaEvidence,
    pub paths: ProofPaths<'a>,
}
/// Validation proof uses canonical declaration and smoke materialization ports.
pub struct ValidationContext<'a> {
    pub repo_root: &'a Path,
    pub declarations: &'a dyn DeclarationEvidence,
    pub validation: &'a dyn ValidationEvidence,
    pub paths: ProofPaths<'a>,
}
