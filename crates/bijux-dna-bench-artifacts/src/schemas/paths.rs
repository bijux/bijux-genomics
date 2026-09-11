//! Stable benchmark schema destinations.
use std::path::{Path, PathBuf};
pub const DEFAULT_BENCHMARK_SCHEMA_ROOT: &str = "benchmarks/schemas";
pub const FASTQ_NORMALIZED_METRICS_SCHEMA_FILE: &str = "fastq-normalized-metrics.v1.json";
pub const BAM_NORMALIZED_METRICS_SCHEMA_FILE: &str = "bam-normalized-metrics.v1.json";
pub const VCF_NORMALIZED_METRICS_SCHEMA_FILE: &str = "vcf-normalized-metrics.v1.json";
pub const VCF_NORMALIZED_METRICS_STAGE_DIR_NAME: &str = "vcf-normalized-metrics";

pub const DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/fastq-normalized-metrics.v1.json";
pub const DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/bam-normalized-metrics.v1.json";
pub const DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/vcf-normalized-metrics.v1.json";
pub const DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR: &str =
    "benchmarks/schemas/vcf-normalized-metrics";

#[must_use]
pub fn fastq_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(FASTQ_NORMALIZED_METRICS_SCHEMA_FILE)
}
#[must_use]
pub fn bam_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(BAM_NORMALIZED_METRICS_SCHEMA_FILE)
}
#[must_use]
pub fn vcf_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(VCF_NORMALIZED_METRICS_SCHEMA_FILE)
}
#[must_use]
pub fn vcf_stage_schema_dir(schema_root: &Path) -> PathBuf {
    schema_root.join(VCF_NORMALIZED_METRICS_STAGE_DIR_NAME)
}
