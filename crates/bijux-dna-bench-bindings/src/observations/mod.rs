//! Nonserialized observations preserve their canonical source authority.
mod expected;
mod serving;
pub use expected::{AssignmentKind, CorpusAssignment, OutputContract};
pub use serving::{
    BamStageAssessment, BamToolAssessment, CorpusDeclaration, FastqAssessment, StageInventory,
};
/// Supported evidence domains, without native execution dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Fastq,
    Bam,
}
impl Domain {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
        }
    }
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Fastq => "FASTQ",
            Self::Bam => "BAM",
        }
    }
}
