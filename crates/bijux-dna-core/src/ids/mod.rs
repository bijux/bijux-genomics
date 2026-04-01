#![allow(missing_docs)]

mod parsing;
mod typed_ids;

use crate::foundation::{BijuxError, Result};
pub use crate::id_catalog;
pub use parsing::{
    parse_pipeline_id, parse_stage_id, parse_tool_id, validate_pipeline_id, validate_pipeline_id_str,
    validate_stage_id, validate_stage_id_str, validate_tool_id, validate_tool_id_str,
};
pub use typed_ids::{
    ArtifactId, ImageDigest, PipelineId, ProfileId, RunId, StageId, StageVersion, StepId, ToolId,
    ToolVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainKind {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibraryLayout {
    SingleEnd,
    PairedEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UdgTreatment {
    None,
    Partial,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformHint {
    Illumina,
    Bgi,
    IonTorrent,
    Nanopore,
    Pacbio,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssayKind {
    Shotgun,
    Capture,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LibraryModel {
    pub layout: LibraryLayout,
    pub udg_treatment: UdgTreatment,
    pub platform_hint: PlatformHint,
    pub assay_kind: AssayKind,
}

impl DomainKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::Vcf => "vcf",
            Self::Cross => "cross",
        }
    }
}

impl TryFrom<&str> for DomainKind {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "fastq" => Ok(Self::Fastq),
            "bam" => Ok(Self::Bam),
            "vcf" => Ok(Self::Vcf),
            "cross" => Ok(Self::Cross),
            _ => Err(BijuxError::validation("unknown domain kind")),
        }
    }
}
