//! Single source of truth for BAM stage metadata and contracts.

use anyhow::{anyhow, Result};
use bijux_core::contract::StageId;
use serde::{Deserialize, Serialize};

use crate::params::{
    AlignEffectiveParams, AuthenticityEffectiveParams, BamEffectiveParams,
    BiasMitigationEffectiveParams, BqsrEffectiveParams, ComplexityEffectiveParams,
    ContaminationEffectiveParams, CoverageEffectiveParams, DamageEffectiveParams,
    FilterEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, MarkDupEffectiveParams, QcPreEffectiveParams, SexEffectiveParams,
    ValidateEffectiveParams,
};

mod specs;
pub use specs::{required_audit_artifacts, stage_spec, stage_specs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamArtifactKind {
    Bam,
    BamIndex,
    ReferenceFasta,
    ReferenceIndex,
    ReferenceDict,
    BedRegions,
    Report,
}

#[derive(Debug, Clone, Copy)]
pub struct BamStageContract {
    pub input: BamArtifactKind,
    pub output: BamArtifactKind,
    pub emits_bam: bool,
    pub emits_report: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditArtifact {
    pub name: &'static str,
    pub filename: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BamStage {
    Align,
    Validate,
    QcPre,
    Filter,
    Markdup,
    Complexity,
    Coverage,
    Damage,
    Authenticity,
    Contamination,
    Sex,
    BiasMitigation,
    Recalibration,
    Haplogroups,
    Genotyping,
    Kinship,
}

pub const STAGE_PREFIX: &str = "bam.";

impl BamStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamStage::Align => "bam.align",
            BamStage::Validate => "bam.validate",
            BamStage::QcPre => "bam.qc_pre",
            BamStage::Filter => "bam.filter",
            BamStage::Markdup => "bam.markdup",
            BamStage::Complexity => "bam.complexity",
            BamStage::Coverage => "bam.coverage",
            BamStage::Damage => "bam.damage",
            BamStage::Authenticity => "bam.authenticity",
            BamStage::Contamination => "bam.contamination",
            BamStage::Sex => "bam.sex",
            BamStage::BiasMitigation => "bam.bias_mitigation",
            BamStage::Recalibration => "bam.recalibration",
            BamStage::Haplogroups => "bam.haplogroups",
            BamStage::Genotyping => "bam.genotyping",
            BamStage::Kinship => "bam.kinship",
        }
    }

    #[must_use]
    pub fn id(self) -> StageId {
        StageId::from_static(self.as_str())
    }

    #[must_use]
    pub const fn all() -> &'static [BamStage] {
        &[
            BamStage::Align,
            BamStage::Validate,
            BamStage::QcPre,
            BamStage::Filter,
            BamStage::Markdup,
            BamStage::Complexity,
            BamStage::Coverage,
            BamStage::Damage,
            BamStage::Authenticity,
            BamStage::Contamination,
            BamStage::Sex,
            BamStage::BiasMitigation,
            BamStage::Recalibration,
            BamStage::Haplogroups,
            BamStage::Genotyping,
            BamStage::Kinship,
        ]
    }

    /// # Errors
    /// Returns an error if the provided JSON does not match the expected
    /// effective params schema for the stage.
    pub fn parse_effective_params(self, value: &serde_json::Value) -> Result<BamEffectiveParams> {
        match self {
            BamStage::Align => serde_json::from_value::<AlignEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Align),
            BamStage::Validate => serde_json::from_value::<ValidateEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Validate),
            BamStage::QcPre => serde_json::from_value::<QcPreEffectiveParams>(value.clone())
                .map(BamEffectiveParams::QcPre),
            BamStage::Filter => serde_json::from_value::<FilterEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Filter),
            BamStage::Markdup => serde_json::from_value::<MarkDupEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Markdup),
            BamStage::Complexity => {
                serde_json::from_value::<ComplexityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Complexity)
            }
            BamStage::Coverage => serde_json::from_value::<CoverageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Coverage),
            BamStage::Damage => serde_json::from_value::<DamageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Damage),
            BamStage::Authenticity => {
                serde_json::from_value::<AuthenticityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Authenticity)
            }
            BamStage::Contamination => {
                serde_json::from_value::<ContaminationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Contamination)
            }
            BamStage::Sex => serde_json::from_value::<SexEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Sex),
            BamStage::BiasMitigation => {
                serde_json::from_value::<BiasMitigationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::BiasMitigation)
            }
            BamStage::Recalibration => serde_json::from_value::<BqsrEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Recalibration),
            BamStage::Haplogroups => {
                serde_json::from_value::<HaplogroupEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Haplogroups)
            }
            BamStage::Genotyping => {
                serde_json::from_value::<GenotypingEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Genotyping)
            }
            BamStage::Kinship => serde_json::from_value::<KinshipEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Kinship),
        }
        .map_err(|err| anyhow!("failed to parse params for {}: {err}", self.as_str()))
    }
}

impl TryFrom<&str> for BamStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "bam.align" => Ok(BamStage::Align),
            "bam.validate" => Ok(BamStage::Validate),
            "bam.qc_pre" => Ok(BamStage::QcPre),
            "bam.filter" => Ok(BamStage::Filter),
            "bam.markdup" => Ok(BamStage::Markdup),
            "bam.complexity" => Ok(BamStage::Complexity),
            "bam.coverage" => Ok(BamStage::Coverage),
            "bam.damage" => Ok(BamStage::Damage),
            "bam.authenticity" => Ok(BamStage::Authenticity),
            "bam.contamination" => Ok(BamStage::Contamination),
            "bam.sex" => Ok(BamStage::Sex),
            "bam.bias_mitigation" => Ok(BamStage::BiasMitigation),
            "bam.recalibration" => Ok(BamStage::Recalibration),
            "bam.haplogroups" => Ok(BamStage::Haplogroups),
            "bam.genotyping" => Ok(BamStage::Genotyping),
            "bam.kinship" => Ok(BamStage::Kinship),
            _ => Err(anyhow!("unknown bam stage: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactPolicy {
    pub required_outputs: &'static [&'static str],
    pub required_audit: &'static [AuditArtifact],
}

#[derive(Debug, Clone)]
pub struct BamStageSpec {
    pub stage: BamStage,
    pub required_inputs: &'static [&'static str],
    pub artifact_policy: ArtifactPolicy,
    pub allowed_tools: &'static [&'static str],
    pub default_tool: &'static str,
    pub default_params: BamEffectiveParams,
}

#[derive(Debug, Clone)]
pub struct StageSpec {
    pub id: StageId,
    pub stage: BamStage,
    pub contract: BamStageContract,
}

#[must_use]
pub fn stage_registry() -> Vec<StageSpec> {
    BamStage::all()
        .iter()
        .map(|stage| StageSpec {
            id: stage.id(),
            stage: *stage,
            contract: contract_for_stage(stage.as_str()).unwrap_or(BamStageContract {
                input: BamArtifactKind::Bam,
                output: BamArtifactKind::Report,
                emits_bam: false,
                emits_report: true,
            }),
        })
        .collect()
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<BamStageContract> {
    let stage = BamStage::try_from(stage_id).ok()?;
    match stage {
        BamStage::Filter | BamStage::Markdup | BamStage::Recalibration => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Bam,
            emits_bam: true,
            emits_report: true,
        }),
        _ => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
        }),
    }
}
