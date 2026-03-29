use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const STAGE_PREFIX: &str = "vcf.";
pub const STAGE_CALL: &str = "vcf.call";
pub const STAGE_FILTER_READS: &str = "vcf.filter";
pub const STAGE_STATS: &str = "vcf.stats";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfInvariantsPreset {
    Minimal,
}

impl VcfInvariantsPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "vcf_minimal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfStage {
    Call,
    Filter,
    Stats,
}

impl VcfStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Call => STAGE_CALL,
            Self::Filter => STAGE_FILTER_READS,
            Self::Stats => STAGE_STATS,
        }
    }

    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Call, Self::Filter, Self::Stats]
    }
}

impl TryFrom<&str> for VcfStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            STAGE_CALL => Ok(Self::Call),
            STAGE_FILTER_READS => Ok(Self::Filter),
            STAGE_STATS => Ok(Self::Stats),
            _ => Err(anyhow!("unknown VCF stage: {value}")),
        }
    }
}
