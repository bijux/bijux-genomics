//! Owner: bijux-dna-bench
//! Benchmark workspace path policy for the repository tree.

use std::path::PathBuf;

#[must_use]
pub fn bench_data_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_data_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data"))
}

#[must_use]
pub fn bench_suites_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_suites_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data/suites"))
}

#[must_use]
pub fn bench_corpora_dir() -> PathBuf {
    bench_data_dir().join("corpora")
}

#[must_use]
pub fn bench_bundles_dir() -> PathBuf {
    bench_data_dir().join("bundles")
}

#[must_use]
pub fn bench_local_config_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| bijux_dna_infra::configs_dir(&root).join("bench").join("local"))
        .unwrap_or_else(|_| PathBuf::from("configs/bench/local"))
}

#[must_use]
pub fn bench_contracts_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| root.join("benchmarks").join("contracts"))
        .unwrap_or_else(|_| PathBuf::from("benchmarks/contracts"))
}

#[must_use]
pub fn bench_stage_campaigns_path() -> PathBuf {
    bench_contracts_dir().join("stage-campaigns.toml")
}

#[must_use]
pub fn bench_portfolio_path() -> PathBuf {
    bench_contracts_dir().join("portfolio.toml")
}

#[must_use]
pub fn bench_data_fitness_path() -> PathBuf {
    bench_contracts_dir().join("data-fitness.toml")
}

#[must_use]
pub fn bench_stage_interfaces_path() -> PathBuf {
    bench_contracts_dir().join("stage-interfaces.toml")
}

#[must_use]
pub fn bench_reports_dir() -> PathBuf {
    super::resolve_repo_root()
        .map(|root| root.join("benchmarks").join("reports"))
        .unwrap_or_else(|_| PathBuf::from("benchmarks/reports"))
}

#[must_use]
pub fn bench_fastq_local_stage_matrix_path() -> PathBuf {
    bench_local_config_dir().join("fastq-stage-matrix.toml")
}

#[must_use]
pub fn bench_bam_local_stage_matrix_path() -> PathBuf {
    bench_local_config_dir().join("bam-stage-matrix.toml")
}

#[cfg(test)]
mod tests {
    use super::{
        bench_bam_local_stage_matrix_path, bench_contracts_dir, bench_data_fitness_path,
        bench_fastq_local_stage_matrix_path, bench_local_config_dir, bench_portfolio_path,
        bench_reports_dir, bench_stage_campaigns_path, bench_stage_interfaces_path,
    };

    #[test]
    fn bench_local_config_paths_follow_repo_layout() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;

        assert_eq!(
            bench_local_config_dir(),
            bijux_dna_infra::configs_dir(&repository).join("bench").join("local"),
        );
        assert_eq!(bench_contracts_dir(), repository.join("benchmarks").join("contracts"),);
        assert_eq!(
            bench_stage_campaigns_path(),
            repository.join("benchmarks").join("contracts").join("stage-campaigns.toml"),
        );
        assert_eq!(
            bench_portfolio_path(),
            repository.join("benchmarks").join("contracts").join("portfolio.toml"),
        );
        assert_eq!(
            bench_data_fitness_path(),
            repository.join("benchmarks").join("contracts").join("data-fitness.toml"),
        );
        assert_eq!(
            bench_stage_interfaces_path(),
            repository.join("benchmarks").join("contracts").join("stage-interfaces.toml"),
        );
        assert_eq!(bench_reports_dir(), repository.join("benchmarks").join("reports"));
        assert_eq!(
            bench_fastq_local_stage_matrix_path(),
            bijux_dna_infra::configs_dir(&repository)
                .join("bench")
                .join("local")
                .join("fastq-stage-matrix.toml"),
        );
        assert_eq!(
            bench_bam_local_stage_matrix_path(),
            bijux_dna_infra::configs_dir(&repository)
                .join("bench")
                .join("local")
                .join("bam-stage-matrix.toml"),
        );
        Ok(())
    }
}
