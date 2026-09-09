//! Owner: bijux-dna-bench-model
//! Catalog observations required by suite admission, independent of a concrete domain provider.

use bijux_dna_core::ids::{StageId, ToolId};
use std::collections::BTreeSet;

mod fastq;
pub use fastq::CanonicalFastqSuiteCatalog;

/// Domain catalog port consumed by pure suite validation.
///
/// Missing contracts and missing execution support remain distinct observations.
/// The caller chooses a catalog snapshot; the validator owns admission policy.
pub trait BenchmarkSuiteCatalog {
    fn declares_stage(&self, stage_id: &str) -> bool;
    fn stage_is_plannable(&self, stage_id: &StageId) -> Option<bool>;
    fn declares_binding(&self, stage_id: &StageId, tool_id: &ToolId) -> bool;
    fn admitted_tools(&self, stage_id: &StageId) -> Vec<ToolId>;
    fn input_ids(&self, stage_id: &str) -> Option<BTreeSet<String>>;
    fn output_ids(&self, stage_id: &str) -> Option<BTreeSet<String>>;
    fn parameter_ids(&self, stage_id: &str) -> Option<BTreeSet<String>>;
}
