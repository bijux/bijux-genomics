use anyhow::{anyhow, Result};
use bijux_core::StageId;

use crate::params::{
    AuthenticityEffectiveParams, BamEffectiveParams, BiasMitigationEffectiveParams,
    BqsrEffectiveParams, ComplexityEffectiveParams, ContaminationEffectiveParams,
    CoverageEffectiveParams, DamageEffectiveParams, FilterEffectiveParams,
    GenotypingEffectiveParams, HaplogroupEffectiveParams, KinshipEffectiveParams,
    MarkDupEffectiveParams, QcPreEffectiveParams, SexEffectiveParams, ValidateEffectiveParams,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BamStage {
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

impl BamStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
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
        StageId(self.as_str().to_string())
    }

    #[must_use]
    pub const fn all() -> &'static [BamStage] {
        &[
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
