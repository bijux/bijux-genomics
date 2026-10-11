//! Stage input construction with explicit species and panel observations.
use crate::panel_contracts::governed_vcf_panel_species_context;
use crate::panel_contracts::resolve_governed_vcf_panel_asset_contract;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::phasing::fixtures::inputs::write_governed_phasing_input_vcf;
use crate::phasing::model::DEFAULT_INPUT_VCF_NAME;
use crate::phasing::model::DEFAULT_OUTPUT_PANEL_ASSETS_NAME;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_domain_vcf::contracts::SpeciesContext;
use bijux_dna_stages_vcf::pipeline::PhasingBackend;
use std::path::Path;
use std::path::PathBuf;

pub(crate) struct PreparedPhasingInputs {
    pub(crate) input_vcf_path: PathBuf,
    pub(crate) stage_root: PathBuf,
    pub(crate) panel_assets_path: PathBuf,
    pub(crate) species_context: SpeciesContext,
    pub(crate) backend: PhasingBackend,
}

pub(crate) fn prepare_inputs(
    output_root: &Path,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
) -> Result<PreparedPhasingInputs> {
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    write_governed_phasing_input_vcf(&input_vcf_path)?;
    let panel_assets_report = resolve_governed_vcf_panel_asset_contract()
        .context("resolve governed VCF panel asset contract")?;
    let panel_assets_path = output_root.join(DEFAULT_OUTPUT_PANEL_ASSETS_NAME);
    bijux_dna_infra::atomic_write_json(&panel_assets_path, &panel_assets_report)?;

    let species_context = governed_vcf_panel_species_context();
    let backend = match contract.tool.as_str() {
        "shapeit" => PhasingBackend::Shapeit,
        "shapeit5" => PhasingBackend::Shapeit5,
        "beagle" => PhasingBackend::Beagle,
        "eagle" => PhasingBackend::Eagle,
        other => bail!("unsupported governed VCF phasing backend `{other}`"),
    };
    Ok(PreparedPhasingInputs {
        input_vcf_path,
        stage_root,
        panel_assets_path,
        species_context,
        backend,
    })
}
