//! Domain selection and governed matrix identities for stage inventory admission.

use super::super::DEFAULT_VCF_STAGE_MATRIX_PATH;

/// Domain whose local benchmark stage declaration is admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkLocalStageInventoryDomain {
    Fastq,
    Bam,
    Vcf,
}

impl BenchmarkLocalStageInventoryDomain {
    /// Stable domain label used in inventory reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::Vcf => "vcf",
        }
    }

    pub(super) const fn stage_prefix(self) -> &'static str {
        match self {
            Self::Fastq => "fastq.",
            Self::Bam => "bam.",
            Self::Vcf => "vcf.",
        }
    }

    pub(super) const fn matrix_relative_path(self) -> &'static str {
        match self {
            Self::Fastq => "configs/bench/local/fastq-stage-matrix.toml",
            Self::Bam => "configs/bench/local/bam-stage-matrix.toml",
            Self::Vcf => DEFAULT_VCF_STAGE_MATRIX_PATH,
        }
    }

    pub(super) const fn expected_matrix_schema_version(self) -> &'static str {
        match self {
            Self::Fastq => "bijux.bench.fastq.local_stage_matrix.v1",
            Self::Bam => "bijux.bench.bam.local_stage_matrix.v1",
            Self::Vcf => "bijux.bench.vcf.local_stage_matrix.v1",
        }
    }
}
