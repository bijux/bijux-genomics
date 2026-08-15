use anyhow::{bail, Result};
use bijux_dna_db_ref::{
    validate_imputation_tool_compatibility, MapCatalogEntry, PanelCatalogEntry,
};
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};

use crate::api::VcfPipelineInputs;
use crate::tool_catalog::{
    default_tool, eagle_license_metadata_present, phasing_backend_supports_gl_only_input,
    stage_compat_tools,
};

/// # Errors
/// Returns an error if a tool cannot be selected for the requested stage.
pub fn choose_tool(
    stage: VcfDomainStage,
    inputs: &VcfPipelineInputs,
    resolved_coverage: CoverageRegime,
    panel: &PanelCatalogEntry,
    map: &MapCatalogEntry,
    planned_stages: &[VcfDomainStage],
) -> Result<(String, String)> {
    let key = stage.as_str().to_string();
    if let Some(selected) = inputs.stage_tool_overrides.get(&key) {
        return Ok((selected.clone(), "stage_tool_override".to_string()));
    }
    if matches!(stage, VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute) {
        if resolved_coverage == CoverageRegime::LowCovGl {
            return Ok(("glimpse".to_string(), "lowcov_gl_default_glimpse".to_string()));
        }
        let shared_support = |tool: &str| {
            panel.compatibility.tool_tags.iter().any(|tag| tag == tool)
                && map.compatibility.tool_tags.iter().any(|tag| tag == tool)
        };
        let phased_gt_ready = planned_stages.contains(&VcfDomainStage::Phasing);
        if phased_gt_ready
            && panel.compatibility.supports_minimac_m3vcf
            && shared_support("minimac4")
        {
            return Ok(("minimac4".to_string(), "phased_gt_plus_m3vcf_minimac4".to_string()));
        }
        if shared_support("impute5") {
            return Ok(("impute5".to_string(), "shared_source_format_impute5".to_string()));
        }
        if shared_support("beagle") {
            return Ok(("beagle".to_string(), "shared_source_format_beagle".to_string()));
        }
        if shared_support("glimpse") {
            return Ok(("glimpse".to_string(), "shared_source_format_glimpse".to_string()));
        }
        bail!("no imputation backend shares the selected panel and map source formats");
    }
    Ok((default_tool(stage, resolved_coverage).to_string(), "coverage_regime_default".to_string()))
}

/// # Errors
/// Returns an error if the selected tool is incompatible with the stage or resolved inputs.
pub fn validate_selected_tool(
    stage: VcfDomainStage,
    tool: &str,
    resolved_coverage: CoverageRegime,
    panel_catalog: &PanelCatalogEntry,
    map_catalog: &MapCatalogEntry,
) -> Result<()> {
    if matches!(
        stage,
        VcfDomainStage::Phasing | VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute
    ) {
        validate_imputation_tool_compatibility(tool, panel_catalog, map_catalog)?;
    }
    if stage == VcfDomainStage::Phasing {
        if resolved_coverage == CoverageRegime::LowCovGl
            && !phasing_backend_supports_gl_only_input(tool)
        {
            bail!(
                "planner refusal: tool {} does not support GL-only input for {}",
                tool,
                stage.as_str()
            );
        }
        if matches!(tool, "shapeit" | "shapeit5" | "eagle")
            && resolved_coverage != CoverageRegime::Diploid
        {
            bail!(
                "planner refusal: tool {} requires diploid coverage regime for {}",
                tool,
                stage.as_str()
            );
        }
        if tool == "eagle" && !eagle_license_metadata_present() {
            bail!("planner refusal: eagle license metadata is missing");
        }
    }
    if !stage_compat_tools(stage).contains(&tool) {
        bail!("selected tool {} is not compatible with stage {}", tool, stage.as_str());
    }
    Ok(())
}
