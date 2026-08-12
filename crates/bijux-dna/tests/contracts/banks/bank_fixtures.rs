use std::cell::Cell;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};

#[path = "../../support/workspace_paths.rs"]
mod test_support;

pub static CWD_LOCK: Mutex<()> = Mutex::new(());
#[allow(dead_code)]
const TEST_LOCK_ROOT: &str = "artifacts/test-locks";

thread_local! {
    static REPO_MUTATOR_LOCK_DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub struct EnvGuard {
    cwd: PathBuf,
    env: BTreeMap<OsString, OsString>,
    _sandbox: Option<RepoSandbox>,
}

impl EnvGuard {
    pub fn new() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let env = std::env::vars_os().collect();
        let sandbox = if repo_mutator_lock_held() {
            Some(RepoSandbox::new("locked-repository-mutator-")?)
        } else {
            None
        };
        if let Some(sandbox) = &sandbox {
            std::env::set_current_dir(sandbox.path())?;
        }
        Ok(Self { cwd, env, _sandbox: sandbox })
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let current: BTreeMap<OsString, OsString> = std::env::vars_os().collect();
        for key in current.keys() {
            if !self.env.contains_key(key) {
                std::env::remove_var(key);
            }
        }
        for (key, value) in &self.env {
            std::env::set_var(key, value);
        }
        let _ = std::env::set_current_dir(&self.cwd);
    }
}

#[allow(dead_code)]
pub fn crate_root(name: &str) -> Result<PathBuf> {
    test_support::crate_root(name)
}

#[allow(dead_code)]
pub fn repo_root() -> Result<PathBuf> {
    test_support::repo_root()
}

#[allow(dead_code)]
pub fn hpc_dry_run_test_directory(repo_root: &Path, label: &str) -> Result<tempfile::TempDir> {
    let parent = repo_root.join("runs/bench/hpc-dry-run");
    fs::create_dir_all(&parent)?;
    Ok(tempfile::Builder::new().prefix(label).tempdir_in(parent)?)
}

#[allow(dead_code)]
pub struct RepoSandbox {
    root: tempfile::TempDir,
    git_dir: PathBuf,
}

#[allow(dead_code)]
impl RepoSandbox {
    pub fn new(label: &str) -> Result<Self> {
        let source_root = test_support::repo_root()?;
        let git_dir = resolve_git_dir(&source_root)?;
        let sandbox_parent = source_root.join("artifacts/readiness-sandboxes");
        fs::create_dir_all(&sandbox_parent)?;
        let root = tempfile::Builder::new().prefix(label).tempdir_in(sandbox_parent)?;
        let checkout = Command::new("git")
            .current_dir(&source_root)
            .args(["checkout-index", "--all", "--force"])
            .arg(format!("--prefix={}/", root.path().display()))
            .output()
            .map_err(|error| anyhow!("materialize repository sandbox: {error}"))?;
        if !checkout.status.success() {
            return Err(anyhow!(
                "materialize repository sandbox: {}",
                String::from_utf8_lossy(&checkout.stderr).trim()
            ));
        }
        Ok(Self { root, git_dir })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.root.path()
    }

    #[must_use]
    pub fn bijux_dna_command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-dna"));
        command
            .current_dir(self.path())
            .env("BIJUX_REPO_ROOT", self.path())
            .env("BIJUX_BENCHMARK_ROOT", self.path().join("benchmarks"))
            .env("GIT_DIR", &self.git_dir)
            .env("GIT_WORK_TREE", self.path());
        command
    }

    pub fn materialize_vcf_score_evidence(&self, home: &Path) -> Result<()> {
        const EVIDENCE_COMMANDS: &[&[&str]] = &[
            &["bench", "local", "run-vcf-admixture-smoke"],
            &["bench", "local", "run-vcf-call-smoke"],
            &["bench", "local", "run-vcf-call-diploid-smoke"],
            &["bench", "local", "run-vcf-call-gl-smoke"],
            &["bench", "local", "run-vcf-call-pseudohaploid-smoke"],
            &["bench", "local", "run-vcf-damage-filter-smoke"],
            &["bench", "local", "run-vcf-demography-smoke"],
            &["bench", "local", "run-vcf-filter-smoke"],
            &["bench", "local", "run-vcf-gl-propagation-smoke"],
            &["bench", "local", "run-vcf-ibd-smoke"],
            &["bench", "local", "run-vcf-ibd-smoke", "--tool-id", "ibdseq"],
            &["bench", "local", "run-vcf-impute-smoke"],
            &["bench", "local", "run-vcf-imputation-metrics-smoke"],
            &["bench", "local", "run-vcf-pca-smoke"],
            &["bench", "local", "run-vcf-pca-smoke", "--tool-id", "eigensoft"],
            &["bench", "local", "run-vcf-phasing-smoke"],
            &["bench", "local", "run-vcf-population-structure-smoke"],
            &["bench", "local", "run-vcf-postprocess-smoke"],
            &["bench", "local", "run-vcf-prepare-reference-panel-smoke"],
            &["bench", "local", "run-vcf-qc-smoke", "--tool-id", "bcftools"],
            &["bench", "local", "run-vcf-qc-smoke", "--tool-id", "plink"],
            &["bench", "local", "run-vcf-qc-smoke"],
            &["bench", "local", "run-vcf-roh-smoke"],
            &["bench", "local", "run-vcf-stats-smoke"],
            &["bench", "readiness", "render-full-benchmark-report"],
            &["bench", "run-micro"],
        ];

        for args in EVIDENCE_COMMANDS {
            let output = self
                .bijux_dna_command()
                .env("HOME", home)
                .env("BIJUX_SKIP_QA", "1")
                .env("BIJUX_ALLOW_SILVER", "1")
                .env("BIJUX_SKIP_IMAGE_CHECK", "1")
                .args(*args)
                .output()
                .map_err(|error| anyhow!("run VCF score evidence command: {error}"))?;
            if !output.status.success() {
                return Err(anyhow!(
                    "VCF score evidence command failed: {}\ncommand: {}\nstdout:\n{}\nstderr:\n{}",
                    output.status,
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Ok(())
    }

    pub fn materialize_bam_score_evidence(&self, home: &Path) -> Result<()> {
        const EVIDENCE_COMMANDS: &[&[&str]] = &[
            &["bench", "local", "run-bam-stage-smoke", "--stage-id", "bam.coverage"],
            &["bench", "readiness", "render-bam-contamination-complete"],
            &["bench", "local", "run-bam-stage-smoke", "--stage-id", "bam.gc_bias"],
        ];

        for args in EVIDENCE_COMMANDS {
            let output = self
                .bijux_dna_command()
                .env("HOME", home)
                .env("BIJUX_SKIP_QA", "1")
                .env("BIJUX_ALLOW_SILVER", "1")
                .env("BIJUX_SKIP_IMAGE_CHECK", "1")
                .args(*args)
                .output()?;
            if !output.status.success() {
                return Err(anyhow!(
                    "BAM score evidence command `{}` failed: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
        }
        Ok(())
    }
}

fn resolve_git_dir(source_root: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .current_dir(source_root)
        .args(["rev-parse", "--absolute-git-dir"])
        .output()
        .map_err(|error| anyhow!("resolve repository git directory: {error}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "resolve repository git directory: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let git_dir = String::from_utf8(output.stdout)
        .map_err(|error| anyhow!("decode repository git directory: {error}"))?;
    Ok(PathBuf::from(git_dir.trim()))
}

#[allow(dead_code)]
#[must_use]
pub fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string())
        .replace('\\', "/")
}

#[allow(dead_code)]
pub struct RepoProcessLock {
    _file_lock: bijux_dna_infra::FileLock,
    _sandbox_request: RepoSandboxRequest,
}

#[allow(dead_code)]
impl RepoProcessLock {
    pub fn acquire(name: &str) -> Result<Self> {
        let repo_root = test_support::repo_root()?;
        let path = repo_root.join(TEST_LOCK_ROOT).join(format!("{name}.lock"));
        let file_lock = bijux_dna_infra::FileLock::try_acquire(&path)
            .map_err(|error| anyhow!("acquire repo test lock `{}`: {error}", path.display()))?;
        Ok(Self { _file_lock: file_lock, _sandbox_request: RepoSandboxRequest::new() })
    }
}

struct RepoSandboxRequest;

impl RepoSandboxRequest {
    fn new() -> Self {
        REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
        Self
    }
}

impl Drop for RepoSandboxRequest {
    fn drop(&mut self) {
        REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

fn repo_mutator_lock_held() -> bool {
    REPO_MUTATOR_LOCK_DEPTH.with(|depth| depth.get() > 0)
}

#[allow(dead_code)]
pub fn with_repo_root<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let _guard = CWD_LOCK.lock().map_err(|err| anyhow!("cwd lock: {err}"))?;
    let _env_guard = EnvGuard::new()?;
    let repo_root = test_support::repo_root()?;
    std::env::set_current_dir(&repo_root)?;
    f()
}

#[allow(dead_code)]
pub fn json_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

#[allow(dead_code)]
pub fn json_bool(value: &Value, key: &str) -> Option<bool> {
    value.get(key).and_then(Value::as_bool)
}

#[allow(dead_code)]
pub fn json_object<'a>(value: &'a Value, key: &str) -> Result<&'a Map<String, Value>> {
    value
        .get(key)
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow!("JSON field `{key}` must be an object"))
}

#[allow(dead_code)]
pub fn json_array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("JSON field `{key}` must be an array"))
}

#[allow(dead_code)]
pub fn object_u64(map: &Map<String, Value>, key: &str) -> Option<u64> {
    map.get(key).and_then(Value::as_u64)
}

#[allow(dead_code)]
pub fn object_u64_sum(map: &Map<String, Value>) -> u64 {
    map.values().filter_map(Value::as_u64).sum()
}
