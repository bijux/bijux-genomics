use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::{stage_tool_binding, ToolIntegrationLevel};

pub(crate) fn include_fastq_active_benchmark_pair(stage_id: &str, tool_id: &str) -> bool {
    stage_tool_binding(&StageId::new(stage_id), &ToolId::new(tool_id))
        .is_none_or(|binding| binding.integration_level == ToolIntegrationLevel::GovernedContract)
}

#[cfg(test)]
mod tests {
    use super::include_fastq_active_benchmark_pair;

    #[test]
    fn active_scope_is_derived_from_governed_bindings() {
        assert!(include_fastq_active_benchmark_pair("fastq.screen_taxonomy", "kraken2"));
        assert!(include_fastq_active_benchmark_pair("fastq.remove_duplicates", "fastuniq"));
        assert!(!include_fastq_active_benchmark_pair("fastq.screen_taxonomy", "diamond"));
        assert!(include_fastq_active_benchmark_pair("fastq.screen_taxonomy", "unknown-tool"));
    }
}
