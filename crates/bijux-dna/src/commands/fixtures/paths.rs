use std::path::{Path, PathBuf};

use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;

pub(crate) const DEFAULT_BENCHMARK_FIXTURE_ROOT: &str = "benchmarks/tests/fixtures";
pub(crate) const DEFAULT_BENCHMARK_PARSER_FIXTURE_ROOT: &str = "benchmarks/tests/fixtures/bench";
pub(crate) const DEFAULT_BENCHMARK_CORPORA_ROOT: &str = "benchmarks/tests/fixtures/corpora";
pub(crate) const DEFAULT_BENCHMARK_DATABASES_ROOT: &str = "benchmarks/tests/fixtures/databases";

pub(crate) fn benchmark_fixture_root_path(cwd: &Path, explicit_root: Option<&Path>) -> PathBuf {
    match explicit_root {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => BenchmarkPathResolver::new(cwd, None).benchmark_fixture_root(),
    }
}

pub(crate) fn benchmark_corpus_manifest_path(fixture_root: &Path, corpus_id: &str) -> PathBuf {
    fixture_root.join("corpora").join(corpus_id).join("manifest.toml")
}

pub(crate) fn benchmark_database_manifest_path(fixture_root: &Path, database_id: &str) -> PathBuf {
    fixture_root.join("databases").join(database_id).join("manifest.toml")
}
