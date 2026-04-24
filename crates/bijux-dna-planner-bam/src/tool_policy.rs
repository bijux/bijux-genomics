use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_bam::params::{BamEffectiveParams, ContaminationScope};
use bijux_dna_domain_bam::BamStage;
use serde_json::Value;
use std::path::Path;

/// # Errors
/// Returns an error if the selected tool is incompatible with the stage contract.
pub fn enforce(
    stage: BamStage,
    tool_id: &str,
    params: Option<&Value>,
    reference: Option<&Path>,
) -> Result<()> {
    match stage {
        BamStage::Authenticity if tool_id == id_catalog::TOOL_PMDTOOLS => {
            if reference.is_none() {
                return Err(anyhow!(
                    "{} with pmdtools requires reference input",
                    id_catalog::BAM_AUTHENTICITY
                ));
            }
        }
        BamStage::Contamination => {
            let effective = params
                .map(|value| stage.parse_effective_params(value))
                .transpose()?
                .and_then(|effective| match effective {
                    BamEffectiveParams::Contamination(contamination) => Some(contamination),
                    _ => None,
                });
            let scope = effective
                .as_ref()
                .map(|contamination| contamination.scope)
                .unwrap_or(ContaminationScope::Both);
            match tool_id {
                id_catalog::TOOL_SCHMUTZI
                    if !matches!(scope, ContaminationScope::Mito | ContaminationScope::Both) =>
                {
                    return Err(anyhow!(
                        "{} tool schmutzi requires scope mito/both",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_SCHMUTZI if reference.is_none() => {
                    return Err(anyhow!(
                        "{} tool schmutzi requires mitochondrial reference input",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if !matches!(scope, ContaminationScope::Nuclear | ContaminationScope::Both) =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires scope nuclear/both",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if effective
                        .as_ref()
                        .is_some_and(|contamination| contamination.reference_panels.is_empty()) =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires non-empty reference_panels",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
