use super::path_resolution::BenchmarkPathResolver;
pub(crate) use bijux_dna_bench_artifacts::schemas::paths::{
    bam_schema_path, fastq_schema_path, vcf_schema_path, vcf_stage_schema_dir,
    DEFAULT_BAM_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH,
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use std::path::{Path, PathBuf};
pub(crate) fn benchmark_schema_root_path(cwd: &Path, explicit_root: Option<&Path>) -> PathBuf {
    match explicit_root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => BenchmarkPathResolver::new(cwd, None).benchmark_schema_root(),
    }
}
