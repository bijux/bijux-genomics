#![allow(missing_docs)]

mod typed_ids;

use crate::foundation::{BijuxError, Result};
pub use crate::id_catalog;
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

/// Canonical stage identifiers owned by bijux-dna-core.
/// # Errors
/// Returns an error if the stage id is invalid.
pub fn parse_stage_id(value: &str) -> Result<StageId> {
    StageId::try_from(value)
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn parse_tool_id(value: &str) -> Result<ToolId> {
    ToolId::try_from(value)
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn parse_pipeline_id(value: &str) -> Result<PipelineId> {
    PipelineId::try_from(value)
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn validate_stage_id(id: &StageId) -> Result<()> {
    validate_stage_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn validate_tool_id(id: &ToolId) -> Result<()> {
    validate_tool_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn validate_pipeline_id(id: &PipelineId) -> Result<()> {
    validate_pipeline_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn validate_stage_id_str(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation("stage id cannot be empty"));
    }
    if !id.contains('.') {
        return Err(BijuxError::validation("stage id must contain '.'"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation(
            "stage id contains invalid characters",
        ));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn validate_tool_id_str(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation("tool id cannot be empty"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation(
            "tool id contains invalid characters",
        ));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn validate_pipeline_id_str(id: &str) -> Result<()> {
    let parts: Vec<&str> = id.split("__").collect();
    if parts.len() != 3 {
        return Err(BijuxError::validation(
            "pipeline id must be <graph>__<flavor>__vN",
        ));
    }
    let graph = parts[0];
    let flavor = parts[1];
    let version = parts[2];
    if !graph.contains("-to-") {
        return Err(BijuxError::validation(
            "pipeline id graph must contain '-to-'",
        ));
    }
    if !version.starts_with('v') || version.len() < 2 || !version[1..].chars().all(char::is_numeric)
    {
        return Err(BijuxError::validation(
            "pipeline id version must be v<digits>",
        ));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if !graph.chars().all(allowed) || !flavor.chars().all(allowed) {
        return Err(BijuxError::validation(
            "pipeline id contains invalid characters",
        ));
    }
    Ok(())
}
