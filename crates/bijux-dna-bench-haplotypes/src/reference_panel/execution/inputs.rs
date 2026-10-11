//! Prepare query and raw reference panel inputs with species identity.
use crate::reference_panel::admission::species::governed_species_context;
use crate::reference_panel::fixtures::inputs::materialize_governed_prepare_reference_panel_raw_fixture;
use crate::reference_panel::fixtures::inputs::write_governed_prepare_reference_panel_input_vcf;
use crate::reference_panel::model::DEFAULT_INPUT_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PreparedPanelInputs {
    pub(crate) input_vcf_path: PathBuf,
    pub(crate) raw_panel_path: PathBuf,
    pub(crate) stage_root: PathBuf,
    pub(crate) species_context: SpeciesContext,
}

pub(crate) fn prepare_inputs(output_root: &Path) -> Result<PreparedPanelInputs> {
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    write_governed_prepare_reference_panel_input_vcf(&input_vcf_path)?;
    let raw_panel_path = materialize_governed_prepare_reference_panel_raw_fixture(&input_root)?;
    let species_context = governed_species_context();

    Ok(PreparedPanelInputs { input_vcf_path, raw_panel_path, stage_root, species_context })
}
