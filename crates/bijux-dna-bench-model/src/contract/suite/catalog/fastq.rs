//! Owner: bijux-dna-bench-model
//! Adapts canonical FASTQ catalog observations to suite validation.

use super::BenchmarkSuiteCatalog;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq as fastq;
use std::collections::BTreeSet;

/// Default catalog adapter preserving the existing suite validation entrypoint.
#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalFastqSuiteCatalog;

impl BenchmarkSuiteCatalog for CanonicalFastqSuiteCatalog {
    fn declares_stage(&self, stage_id: &str) -> bool {
        fastq::contract_for_stage(stage_id).is_some()
    }
    fn stage_is_plannable(&self, stage_id: &StageId) -> Option<bool> {
        fastq::execution_support_for_stage(stage_id).map(|support| support.is_plannable())
    }
    fn declares_binding(&self, stage_id: &StageId, tool_id: &ToolId) -> bool {
        fastq::stage_tool_binding(stage_id, tool_id).is_some()
    }
    fn admitted_tools(&self, stage_id: &StageId) -> Vec<ToolId> {
        fastq::admitted_execution_tools_for_stage(stage_id)
    }
    fn input_ids(&self, stage_id: &str) -> Option<BTreeSet<String>> {
        fastq::stage_input_ids(stage_id)
    }
    fn output_ids(&self, stage_id: &str) -> Option<BTreeSet<String>> {
        fastq::stage_output_ids(stage_id)
    }
    fn parameter_ids(&self, stage_id: &str) -> Option<BTreeSet<String>> {
        fastq::stage_parameter_ids(stage_id)
    }
}
