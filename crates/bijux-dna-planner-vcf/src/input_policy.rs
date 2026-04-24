use anyhow::{bail, Result};
use bijux_dna_domain_vcf::contracts::{
    validate_entry_vcf_invariants, validate_panel_map_invariants, validate_species_context,
};

use crate::api::VcfPipelineInputs;

/// # Errors
/// Returns an error if the planner input domain or invariants are not valid for VCF planning.
pub fn validate(inputs: &VcfPipelineInputs) -> Result<()> {
    if inputs.pipeline_domain != "vcf" {
        bail!("vcf planner refusal: non-applicable domain `{}`", inputs.pipeline_domain);
    }
    let lowered = inputs.pipeline_domain.to_ascii_lowercase();
    if lowered.contains("edna") || lowered.contains("pollen") {
        bail!("vcf planner refusal: imputation is not applicable to eDNA/pollen domains");
    }
    validate_species_context(&inputs.species_context)?;
    validate_entry_vcf_invariants(&inputs.species_context, &inputs.entry_vcf_invariants)?;
    validate_panel_map_invariants(&inputs.species_context, &inputs.panel_map_invariants)?;
    Ok(())
}
