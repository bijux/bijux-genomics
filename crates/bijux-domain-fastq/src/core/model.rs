use std::path::PathBuf;

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
