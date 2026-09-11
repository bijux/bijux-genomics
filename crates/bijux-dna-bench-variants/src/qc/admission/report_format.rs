//! Report dialect is independent of the internal QC execution engine.
use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QcReportFormat {
    Bcftools,
    Plink,
    Plink2,
}

impl QcReportFormat {
    pub(crate) fn from_tool(tool: &str) -> Result<Self> {
        match tool {
            "bcftools" => Ok(Self::Bcftools),
            "plink" => Ok(Self::Plink),
            "plink2" => Ok(Self::Plink2),
            other => bail!("unsupported VCF QC smoke tool `{other}`"),
        }
    }
    pub(crate) fn tool(self) -> &'static str {
        match self {
            Self::Bcftools => "bcftools",
            Self::Plink => "plink",
            Self::Plink2 => "plink2",
        }
    }
}
