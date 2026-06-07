use std::path::{Path, PathBuf};

pub(crate) const BENCHMARK_ROOT_ENV: &str = "BIJUX_BENCHMARK_ROOT";
pub(crate) const DEFAULT_BENCHMARK_ROOT_RELATIVE: &str = "benchmarks";
pub(crate) const DEFAULT_BENCHMARK_CONFIG_ROOT_SUFFIX: &str = "configs";
pub(crate) const DEFAULT_BENCHMARK_LOCAL_CONFIG_ROOT_SUFFIX: &str = "configs/local";
pub(crate) const DEFAULT_BENCHMARK_PIPELINE_CONFIG_ROOT_SUFFIX: &str = "configs/pipelines/local";
pub(crate) const DEFAULT_BENCHMARK_HPC_CAMPAIGN_ROOT_SUFFIX: &str = "configs/hpc/campaign";
pub(crate) const DEFAULT_BENCHMARK_SCHEMA_ROOT_SUFFIX: &str = "schemas";
pub(crate) const DEFAULT_BENCHMARK_FIXTURE_ROOT_SUFFIX: &str = "tests/fixtures";
pub(crate) const DEFAULT_BENCHMARK_PARSER_FIXTURE_ROOT_SUFFIX: &str = "tests/fixtures/bench";
pub(crate) const DEFAULT_BENCHMARK_CORPORA_ROOT_SUFFIX: &str = "tests/fixtures/corpora";
pub(crate) const DEFAULT_BENCHMARK_DATABASES_ROOT_SUFFIX: &str = "tests/fixtures/databases";
pub(crate) const DEFAULT_BENCHMARK_READINESS_ROOT_RELATIVE: &str = "target/bench-readiness";
pub(crate) const DEFAULT_BENCHMARK_LOCAL_READY_ROOT_RELATIVE: &str = "target/local-ready";
pub(crate) const DEFAULT_BENCHMARK_LOCAL_FAKE_RUN_ROOT_RELATIVE: &str = "target/local-fake-runs";
pub(crate) const DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE: &str = "target/slurm-dry-run";

#[derive(Debug, Clone)]
pub(crate) struct BenchmarkPathResolver {
    repo_root: PathBuf,
    benchmark_root: PathBuf,
}

impl BenchmarkPathResolver {
    pub(crate) fn new(repo_root: &Path, explicit_benchmark_root: Option<&Path>) -> Self {
        Self {
            repo_root: repo_root.to_path_buf(),
            benchmark_root: resolve_benchmark_root(repo_root, explicit_benchmark_root),
        }
    }

    pub(crate) fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    pub(crate) fn benchmark_root(&self) -> &Path {
        &self.benchmark_root
    }

    pub(crate) fn benchmark_config_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_CONFIG_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_local_config_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_LOCAL_CONFIG_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_pipeline_config_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_PIPELINE_CONFIG_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_hpc_campaign_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_HPC_CAMPAIGN_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_schema_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_SCHEMA_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_fixture_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_FIXTURE_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_parser_fixture_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_PARSER_FIXTURE_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_corpora_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_CORPORA_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_databases_root(&self) -> PathBuf {
        self.benchmark_root.join(DEFAULT_BENCHMARK_DATABASES_ROOT_SUFFIX)
    }

    pub(crate) fn benchmark_readiness_root(&self) -> PathBuf {
        self.repo_root.join(DEFAULT_BENCHMARK_READINESS_ROOT_RELATIVE)
    }

    pub(crate) fn benchmark_local_ready_root(&self) -> PathBuf {
        self.repo_root.join(DEFAULT_BENCHMARK_LOCAL_READY_ROOT_RELATIVE)
    }

    pub(crate) fn benchmark_local_fake_run_root(&self) -> PathBuf {
        self.repo_root.join(DEFAULT_BENCHMARK_LOCAL_FAKE_RUN_ROOT_RELATIVE)
    }

    pub(crate) fn benchmark_slurm_dry_run_root(&self) -> PathBuf {
        self.repo_root.join(DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE)
    }

    pub(crate) fn resolve_repo_relative(&self, candidate: &Path) -> PathBuf {
        if candidate.is_absolute() {
            candidate.to_path_buf()
        } else {
            self.repo_root.join(candidate)
        }
    }
}

pub(crate) fn benchmark_root_env_binding() -> Option<PathBuf> {
    std::env::var_os(BENCHMARK_ROOT_ENV).filter(|value| !value.is_empty()).map(PathBuf::from)
}

pub(crate) fn resolve_benchmark_root(
    repo_root: &Path,
    explicit_benchmark_root: Option<&Path>,
) -> PathBuf {
    if let Some(path) = explicit_benchmark_root {
        return absolutize(repo_root, path);
    }
    if let Some(path) = benchmark_root_env_binding() {
        return absolutize(repo_root, &path);
    }
    repo_root.join(DEFAULT_BENCHMARK_ROOT_RELATIVE)
}

pub(crate) fn absolutize(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        BenchmarkPathResolver, BENCHMARK_ROOT_ENV, DEFAULT_BENCHMARK_LOCAL_FAKE_RUN_ROOT_RELATIVE,
        DEFAULT_BENCHMARK_LOCAL_READY_ROOT_RELATIVE, DEFAULT_BENCHMARK_READINESS_ROOT_RELATIVE,
        DEFAULT_BENCHMARK_ROOT_RELATIVE, DEFAULT_BENCHMARK_SCHEMA_ROOT_SUFFIX,
        DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE,
    };
    use std::ffi::{OsStr, OsString};
    use std::path::Path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        value: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl AsRef<OsStr>) -> Self {
            let guard = Self { key, value: std::env::var_os(key) };
            std::env::set_var(key, value.as_ref());
            guard
        }

        fn remove(key: &'static str) -> Self {
            let guard = Self { key, value: std::env::var_os(key) };
            std::env::remove_var(key);
            guard
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(value) = &self.value {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn benchmark_path_resolver_uses_default_root_when_unset() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _root_env = EnvVarGuard::remove(BENCHMARK_ROOT_ENV);
        let repo_root = Path::new("/workspace/repo");

        let resolver = BenchmarkPathResolver::new(repo_root, None);

        assert_eq!(resolver.benchmark_root(), repo_root.join(DEFAULT_BENCHMARK_ROOT_RELATIVE));
        assert_eq!(
            resolver.benchmark_schema_root(),
            repo_root
                .join(DEFAULT_BENCHMARK_ROOT_RELATIVE)
                .join(DEFAULT_BENCHMARK_SCHEMA_ROOT_SUFFIX)
        );
        assert_eq!(
            resolver.benchmark_readiness_root(),
            repo_root.join(DEFAULT_BENCHMARK_READINESS_ROOT_RELATIVE)
        );
        assert_eq!(
            resolver.benchmark_local_ready_root(),
            repo_root.join(DEFAULT_BENCHMARK_LOCAL_READY_ROOT_RELATIVE)
        );
        assert_eq!(
            resolver.benchmark_local_fake_run_root(),
            repo_root.join(DEFAULT_BENCHMARK_LOCAL_FAKE_RUN_ROOT_RELATIVE)
        );
        assert_eq!(
            resolver.benchmark_slurm_dry_run_root(),
            repo_root.join(DEFAULT_BENCHMARK_SLURM_DRY_RUN_ROOT_RELATIVE)
        );
    }

    #[test]
    fn benchmark_path_resolver_honors_explicit_benchmark_root() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _root_env = EnvVarGuard::set(BENCHMARK_ROOT_ENV, "benchmarks-from-env");
        let repo_root = Path::new("/workspace/repo");

        let resolver = BenchmarkPathResolver::new(repo_root, Some(Path::new("benchmarks-custom")));

        assert_eq!(resolver.benchmark_root(), repo_root.join("benchmarks-custom"));
        assert_eq!(
            resolver.benchmark_pipeline_config_root(),
            repo_root.join("benchmarks-custom/configs/pipelines/local")
        );
    }

    #[test]
    fn benchmark_path_resolver_honors_environment_override() {
        let _env_lock = ENV_LOCK.lock().expect("env lock");
        let _root_env = EnvVarGuard::set(BENCHMARK_ROOT_ENV, "benchmarks-from-env");
        let repo_root = Path::new("/workspace/repo");

        let resolver = BenchmarkPathResolver::new(repo_root, None);

        assert_eq!(resolver.benchmark_root(), repo_root.join("benchmarks-from-env"));
        assert_eq!(
            resolver.benchmark_fixture_root(),
            repo_root.join("benchmarks-from-env/tests/fixtures")
        );
        assert_eq!(
            resolver.benchmark_hpc_campaign_root(),
            repo_root.join("benchmarks-from-env/configs/hpc/campaign")
        );
    }
}
