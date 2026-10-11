//! Readiness dispositions derived from one governed BAM tool observation.
use bijux_dna_core::ids::StageId;

use super::domain_tool_specs::{BamDomainToolDefinition, BamDomainToolSupportLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamToolContractStatus {
    Missing,
    Mismatched,
    Admitted(BamDomainToolSupportLevel),
}

impl BamToolContractStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing_contract",
            Self::Mismatched => "mismatched_contract",
            Self::Admitted(level) => level.as_str(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamToolAdapterStatus {
    DeclaredOnly,
    Plannable,
    Runnable,
}

impl BamToolAdapterStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DeclaredOnly => "declared_only",
            Self::Plannable => "plannable",
            Self::Runnable => "runnable",
        }
    }
}

/// Contract admission and spec viability; this does not probe an installed tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BamToolDefinitionReadiness {
    pub support: BamToolContractStatus,
    pub adapter: BamToolAdapterStatus,
}

/// Assess metadata admission and stage-spec viability independently.
#[must_use]
pub fn assess_bam_tool_definition(
    definition: Option<&BamDomainToolDefinition>,
    stage_id: &StageId,
) -> BamToolDefinitionReadiness {
    let Some(definition) = definition else {
        return BamToolDefinitionReadiness {
            support: BamToolContractStatus::Missing,
            adapter: BamToolAdapterStatus::DeclaredOnly,
        };
    };
    let support = match definition.contract_metadata() {
        Err(_) => BamToolContractStatus::Missing,
        Ok(metadata) => {
            let admitted = metadata.stage_ids.iter().any(|candidate| candidate == stage_id)
                || metadata.planned_stage_ids.iter().any(|candidate| candidate == stage_id);
            if admitted {
                BamToolContractStatus::Admitted(metadata.pair_support_level(stage_id))
            } else {
                BamToolContractStatus::Mismatched
            }
        }
    };
    let adapter = if definition.execution_spec(stage_id).is_ok() {
        BamToolAdapterStatus::Runnable
    } else if definition.planning_spec(stage_id).is_ok() {
        BamToolAdapterStatus::Plannable
    } else {
        BamToolAdapterStatus::DeclaredOnly
    };
    BamToolDefinitionReadiness { support, adapter }
}

#[cfg(test)]
#[path = "../../tests/contracts/selection/tool_readiness.rs"]
mod tests;
