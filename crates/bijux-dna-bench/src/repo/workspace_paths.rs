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
        .map(|root| root.join("configs").join("bench").join("local"))
        .unwrap_or_else(|_| PathBuf::from("configs/bench/local"))
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
    use std::fs;
    use std::path::PathBuf;

    use super::{
        bench_bam_local_stage_matrix_path, bench_fastq_local_stage_matrix_path,
        bench_local_config_dir,
    };

    struct RepoRootOverrideGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl RepoRootOverrideGuard {
        fn install(root: &std::path::Path) -> Self {
            let previous = std::env::var_os("BIJUX_REPO_ROOT");
            std::env::set_var("BIJUX_REPO_ROOT", root);
            Self { previous }
        }
    }

    impl Drop for RepoRootOverrideGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.take() {
                std::env::set_var("BIJUX_REPO_ROOT", previous);
            } else {
                std::env::remove_var("BIJUX_REPO_ROOT");
            }
        }
    }

    #[test]
    fn bench_local_config_paths_follow_repo_layout() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        fs::write(temp.path().join("Cargo.lock"), "")?;
        fs::create_dir_all(temp.path().join("crates"))?;
        fs::create_dir_all(temp.path().join("configs").join("bench").join("local"))?;
        let _guard = RepoRootOverrideGuard::install(temp.path());

        assert_eq!(
            bench_local_config_dir(),
            temp.path().join("configs").join("bench").join("local"),
        );
        assert_eq!(
            bench_fastq_local_stage_matrix_path(),
            PathBuf::from(temp.path())
                .join("configs")
                .join("bench")
                .join("local")
                .join("fastq-stage-matrix.toml"),
        );
        assert_eq!(
            bench_bam_local_stage_matrix_path(),
            PathBuf::from(temp.path())
                .join("configs")
                .join("bench")
                .join("local")
                .join("bam-stage-matrix.toml"),
        );
        Ok(())
    }
}
