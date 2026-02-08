#![allow(missing_docs)]

pub const FASTQ_PREFIX: &str = "fastq.";
pub const BAM_PREFIX: &str = "bam.";
pub const CORE_PREFIX: &str = "core.";

pub const CORE_PREPARE_REFERENCE: &str = "core.prepare_reference";
pub const REPORT_AGGREGATE_STAGE: &str = "report.aggregate";
pub const REPORT_AGGREGATE_STEP: &str = "report.aggregate";

pub const FASTQ_PREPROCESS: &str = "fastq.preprocess";
pub const FASTQ_VALIDATE_PRE: &str = "fastq.validate_pre";
pub const FASTQ_TRIM: &str = "fastq.trim";
pub const FASTQ_FILTER: &str = "fastq.filter";
pub const FASTQ_MERGE: &str = "fastq.merge";
pub const FASTQ_CORRECT: &str = "fastq.correct";
pub const FASTQ_QC_POST: &str = "fastq.qc_post";
pub const FASTQ_UMI: &str = "fastq.umi";
pub const FASTQ_SCREEN: &str = "fastq.screen";
pub const FASTQ_STATS_NEUTRAL: &str = "fastq.stats_neutral";

pub const BAM_ALIGN: &str = "bam.align";
pub const BAM_VALIDATE: &str = "bam.validate";
pub const BAM_MARKDUP: &str = "bam.markdup";
pub const BAM_RECALIBRATION: &str = "bam.recalibration";
pub const BAM_COVERAGE: &str = "bam.coverage";
pub const BAM_DAMAGE: &str = "bam.damage";
pub const BAM_COMPLEXITY: &str = "bam.complexity";
pub const BAM_AUTHENTICITY: &str = "bam.authenticity";
pub const BAM_HAPLOGROUPS: &str = "bam.haplogroups";
pub const BAM_KINSHIP: &str = "bam.kinship";
pub const BAM_CONTAMINATION: &str = "bam.contamination";
pub const BAM_SEX: &str = "bam.sex";

pub const PIPELINE_FASTQ_DEFAULT: &str = "fastq-to-fastq__default__v1";
pub const PIPELINE_FASTQ_MINIMAL: &str = "fastq-to-fastq__minimal__v1";
pub const PIPELINE_FASTQ_ADNA: &str = "fastq-to-fastq__adna__v1";
pub const PIPELINE_BAM_DEFAULT: &str = "bam-to-bam__default__v1";
pub const PIPELINE_BAM_ADNA_SHOTGUN: &str = "bam-to-bam__adna_shotgun__v1";
pub const PIPELINE_BAM_ADNA_CAPTURE: &str = "bam-to-bam__adna_capture__v1";
pub const PIPELINE_FASTQ_TO_BAM_DEFAULT: &str = "fastq-to-bam__default__v1";
pub const PIPELINE_FASTQ_TO_BAM_ADNA_SHOTGUN: &str = "fastq-to-bam__adna_shotgun__v1";
