use std::path::{Path, PathBuf};

use super::path_resolution::BenchmarkPathResolver;

pub(crate) const DEFAULT_BENCHMARK_SCHEMA_ROOT: &str = "benchmarks/schemas";
pub(crate) const FASTQ_NORMALIZED_METRICS_SCHEMA_FILE: &str = "fastq-normalized-metrics.v1.json";
pub(crate) const BAM_NORMALIZED_METRICS_SCHEMA_FILE: &str = "bam-normalized-metrics.v1.json";
pub(crate) const VCF_NORMALIZED_METRICS_SCHEMA_FILE: &str = "vcf-normalized-metrics.v1.json";
pub(crate) const VCF_NORMALIZED_METRICS_STAGE_DIR_NAME: &str = "vcf-normalized-metrics";

pub(crate) const DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/fastq-normalized-metrics.v1.json";
pub(crate) const DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/bam-normalized-metrics.v1.json";
pub(crate) const DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH: &str =
    "benchmarks/schemas/vcf-normalized-metrics.v1.json";
pub(crate) const DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR: &str =
    "benchmarks/schemas/vcf-normalized-metrics";

pub(crate) fn benchmark_schema_root_path(cwd: &Path, explicit_root: Option<&Path>) -> PathBuf {
    match explicit_root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => BenchmarkPathResolver::new(cwd, None).benchmark_schema_root(),
    }
}

pub(crate) fn fastq_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(FASTQ_NORMALIZED_METRICS_SCHEMA_FILE)
}

pub(crate) fn bam_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(BAM_NORMALIZED_METRICS_SCHEMA_FILE)
}

pub(crate) fn vcf_schema_path(schema_root: &Path) -> PathBuf {
    schema_root.join(VCF_NORMALIZED_METRICS_SCHEMA_FILE)
}

pub(crate) fn vcf_stage_schema_dir(schema_root: &Path) -> PathBuf {
    schema_root.join(VCF_NORMALIZED_METRICS_STAGE_DIR_NAME)
}
