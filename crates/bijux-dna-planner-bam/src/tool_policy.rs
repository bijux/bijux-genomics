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
    enforce_registered_tool(stage, tool_id)?;
    match stage {
        BamStage::Align => {
            let BamEffectiveParams::Align(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_ALIGN));
            };
            let strategy = bijux_dna_domain_bam::bam_alignment_strategy_for_tool(
                tool_id,
                Some(&effective.preset),
            )
            .ok_or_else(|| anyhow!("{} tool {tool_id} lacks declared alignment strategy", id_catalog::BAM_ALIGN))?;
            if strategy.hidden_default_allowed {
                return Err(anyhow!(
                    "{} must not rely on hidden alignment defaults",
                    id_catalog::BAM_ALIGN
                ));
            }
        }
        BamStage::Authenticity if tool_id == id_catalog::TOOL_PMDTOOLS => {
            if reference.is_none() {
                return Err(anyhow!(
                    "{} with pmdtools requires reference input",
                    id_catalog::BAM_AUTHENTICITY
                ));
            }
        }
        BamStage::Authenticity => {
            let BamEffectiveParams::Authenticity(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_AUTHENTICITY));
            };
            if !effective.evidence_only || !effective.disallow_certification {
                return Err(anyhow!(
                    "{} must remain evidence-only and cannot certify authenticity",
                    id_catalog::BAM_AUTHENTICITY
                ));
            }
        }
        BamStage::Damage => {
            let BamEffectiveParams::Damage(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_DAMAGE));
            };
            if !effective.evidence_only {
                return Err(anyhow!(
                    "{} must remain an evidence-only damage workflow",
                    id_catalog::BAM_DAMAGE
                ));
            }
        }
        BamStage::Contamination => {
            let BamEffectiveParams::Contamination(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_CONTAMINATION));
            };
            if !effective.emit_confidence_caveats {
                return Err(anyhow!(
                    "{} must emit confidence and caveat metadata",
                    id_catalog::BAM_CONTAMINATION
                ));
            }
            let scope = effective.scope;
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
                    if effective.reference_panels.is_empty() =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires non-empty reference_panels",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if effective.chromosome_system.as_deref().is_none() =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires declared chromosome_system",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                id_catalog::TOOL_VERIFYBAMID2 | id_catalog::TOOL_CONTAMMIX
                    if effective.minimum_mean_coverage.is_none() =>
                {
                    return Err(anyhow!(
                        "{} tool {tool_id} requires declared minimum_mean_coverage",
                        id_catalog::BAM_CONTAMINATION
                    ));
                }
                _ => {}
            }
        }
        BamStage::Sex => {
            let BamEffectiveParams::Sex(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_SEX));
            };
            if effective.refuse_without_context && effective.chromosome_system.is_none() {
                return Err(anyhow!(
                    "{} requires chromosome_system when refuse_without_context is enabled",
                    id_catalog::BAM_SEX
                ));
            }
        }
        BamStage::EndogenousContent => {
            let BamEffectiveParams::EndogenousContent(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_ENDOGENOUS_CONTENT));
            };
            if effective.refuse_without_host_reference
                && effective.host_reference_scope.trim().is_empty()
            {
                return Err(anyhow!(
                    "{} requires explicit host_reference_scope",
                    id_catalog::BAM_ENDOGENOUS_CONTENT
                ));
            }
        }
        BamStage::Haplogroups => {
            let BamEffectiveParams::Haplogroups(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_HAPLOGROUPS));
            };
            if effective.reference_panel.trim().is_empty()
                || effective.reference_build.trim().is_empty()
                || effective.population_scope.as_deref().is_none()
            {
                return Err(anyhow!(
                    "{} requires explicit reference_panel, reference_build, and population_scope",
                    id_catalog::BAM_HAPLOGROUPS
                ));
            }
        }
        BamStage::Kinship => {
            let BamEffectiveParams::Kinship(effective) =
                crate::params::effective_params_for_stage(stage, params)?
            else {
                return Err(anyhow!("{} params mismatch", id_catalog::BAM_KINSHIP));
            };
            if effective.reference_panel.trim().is_empty()
                || effective.reference_build.trim().is_empty()
                || effective.population_scope.trim().is_empty()
            {
                return Err(anyhow!(
                    "{} requires explicit reference_panel, reference_build, and population_scope",
                    id_catalog::BAM_KINSHIP
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn enforce_registered_tool(stage: BamStage, tool_id: &str) -> Result<()> {
    let allowed = crate::selection::allowed_tools_for_stage(stage);
    if allowed.iter().any(|tool| tool.as_str() == tool_id) {
        return Ok(());
    }
    let allowed = allowed.iter().map(|tool| tool.as_str()).collect::<Vec<_>>().join(", ");
    Err(anyhow!(
        "tool {tool_id} is not registered for {}; allowed tools: {allowed}",
        stage.as_str()
    ))
}
