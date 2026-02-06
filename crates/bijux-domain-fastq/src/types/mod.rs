use std::path::PathBuf;

pub use bijux_core::primitives::input_assessment::{FastqLayout, FastqSampleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FastqArtifactKind {
    SingleEnd,
    PairedEnd,
    Merged,
    StatsOnly,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FastqArtifact {
    pub path: PathBuf,
    pub kind: FastqArtifactKind,
}

impl FastqArtifact {
    pub fn single_end(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: FastqArtifactKind::SingleEnd,
        }
    }

    pub fn merged(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            kind: FastqArtifactKind::Merged,
        }
    }

    pub fn paired_end(r1: impl Into<PathBuf>, r2: impl Into<PathBuf>) -> (Self, Self) {
        (
            Self {
                path: r1.into(),
                kind: FastqArtifactKind::PairedEnd,
            },
            Self {
                path: r2.into(),
                kind: FastqArtifactKind::PairedEnd,
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct FastqSE {
    pub r1: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FastqPE {
    pub r1: PathBuf,
    pub r2: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FastqStats {
    pub report: PathBuf,
}

pub type FastqSingleEnd = FastqSE;
pub type FastqPairedEnd = FastqPE;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolReferenceV1 {
    pub id: String,
    pub stage: String,
    pub version: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionReportV1 {
    pub schema_version: String,
    pub definition: String,
    pub numerator: serde_json::Value,
    pub denominator: serde_json::Value,
    pub units: String,
    pub scope: String,
    pub stage_boundary: String,
    pub tool: ToolReferenceV1,
    pub raw_reads_total: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterTrimmingReportV1 {
    pub schema_version: String,
    pub reads_with_adapter: u64,
    pub total_reads: u64,
    pub bases_trimmed_total: u64,
    pub per_adapter_counts: std::collections::BTreeMap<String, u64>,
    pub top_k_adapters: Vec<AdapterContributionV1>,
    pub tool: ToolReferenceV1,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdapterContributionV1 {
    pub id: String,
    pub count: u64,
}
