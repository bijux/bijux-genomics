//! Owner: bijux-dna-api
//! Thin application composition for the complete benchmark stage-plugin set.

use anyhow::Result;
use bijux_dna_bench::BenchmarkStagePluginRegistry;
use bijux_dna_stage_contract::{canonical_tool_bindings, ToolBindingRegistry};
use bijux_dna_stages_bam::BamStagePlugin;
use bijux_dna_stages_fastq::FastqStagePlugin;
use bijux_dna_stages_vcf::VcfStagePlugin;

/// Concrete domain plugin ownership at the public application boundary.
pub struct BenchmarkStagePluginComposition {
    fastq: FastqStagePlugin,
    bam: BamStagePlugin,
    vcf: VcfStagePlugin,
}

impl BenchmarkStagePluginComposition {
    #[must_use]
    pub const fn new() -> Self {
        Self { fastq: FastqStagePlugin, bam: BamStagePlugin, vcf: VcfStagePlugin }
    }

    #[must_use]
    pub fn registry(&self) -> BenchmarkStagePluginRegistry<'_> {
        BenchmarkStagePluginRegistry::new(vec![&self.fastq, &self.bam, &self.vcf])
    }

    /// Validates that every binding-backed stage resolves to one concrete domain plugin.
    ///
    /// # Errors
    /// Returns an error when plugin ownership is missing or ambiguous.
    pub fn validate_for_bindings(&self, bindings: &ToolBindingRegistry) -> Result<()> {
        self.registry().validate_for_bindings(bindings)
    }
}

impl Default for BenchmarkStagePluginComposition {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates the production composition against the canonical binding authority.
///
/// # Errors
/// Returns an error when the canonical stage/tool closure has missing or ambiguous plugin
/// ownership.
pub fn validate_benchmark_stage_plugin_composition() -> Result<()> {
    BenchmarkStagePluginComposition::new().validate_for_bindings(&canonical_tool_bindings()?)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bijux_dna_stage_contract::{canonical_tool_bindings, ToolBinding};

    use super::*;

    #[test]
    fn production_composition_closes_every_canonical_stage() -> Result<()> {
        let bindings = canonical_tool_bindings()?;
        let stages = bindings
            .bindings()
            .map(|binding| binding.stage_id().as_str().to_string())
            .collect::<BTreeSet<_>>();
        let composition = BenchmarkStagePluginComposition::new();

        composition.validate_for_bindings(&bindings)?;
        assert_eq!(stages.len(), 71);
        Ok(())
    }
}
