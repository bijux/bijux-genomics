use super::ports::{CompatibilityEvidence, CoverageEvidence, InventoryEvidence};
use std::path::Path;
/// Explicit repository and independent canonical observation authorities.
pub struct AssignmentContext<'a> {
    pub repo_root: &'a Path,
    pub inventory: &'a dyn InventoryEvidence,
    pub coverage: &'a dyn CoverageEvidence,
    pub compatibility: &'a dyn CompatibilityEvidence,
}
