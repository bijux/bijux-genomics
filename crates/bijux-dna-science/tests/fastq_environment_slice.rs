use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use bijux_dna_science::compile::compile_workspace;
use bijux_dna_science::domain::CompiledScience;
use bijux_dna_science::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv, fastq_closure_gate_tsv,
    fastq_container_reference_tsv, fastq_default_binding_risk_tsv, fastq_download_backlog_tsv,
    fastq_environment_tsv, fastq_missing_closure_prerequisites_tsv, fastq_paper_archive_tsv,
    fastq_truth_delta_tsv, index_json, source_archive_gaps_tsv, source_inventory_tsv,
    to_pretty_json,
};

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("resolve repository root from crate manifest")
}

const TEST_LOCK_ROOT: &str = "artifacts/test-locks";
const TEST_LOCK_WAIT_TIMEOUT: Duration = Duration::from_mins(5);
const TEST_LOCK_POLL_INTERVAL: Duration = Duration::from_millis(50);
const TEST_LOCK_OWNER_FILE: &str = "owner.pid";
const TEST_LOCK_MISSING_OWNER_GRACE: Duration = Duration::from_secs(1);

static CWD_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    cwd: PathBuf,
    env: std::collections::BTreeMap<OsString, OsString>,
}

impl EnvGuard {
    fn new() -> Result<Self> {
        Ok(Self { cwd: std::env::current_dir()?, env: std::env::vars_os().collect() })
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        let current: std::collections::BTreeMap<OsString, OsString> = std::env::vars_os().collect();
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

struct RepoProcessLock {
    path: PathBuf,
}

impl RepoProcessLock {
    fn acquire(name: &str) -> Result<Self> {
        let repo_root = repo_root()?;
        let lock_root = repo_root.join(TEST_LOCK_ROOT);
        fs::create_dir_all(&lock_root)?;
        let path = lock_root.join(name);
        let deadline = Instant::now() + TEST_LOCK_WAIT_TIMEOUT;

        loop {
            match fs::create_dir(&path) {
                Ok(()) => {
                    write_lock_owner(&path)?;
                    return Ok(Self { path });
                }
                Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                    if stale_repo_test_lock(&path)? {
                        match fs::remove_dir_all(&path) {
                            Ok(()) => continue,
                            Err(remove_error) if remove_error.kind() == ErrorKind::NotFound => {
                                continue;
                            }
                            Err(remove_error) => {
                                return Err(anyhow::anyhow!(
                                    "remove stale repo test lock `{}`: {remove_error}",
                                    path.display()
                                ));
                            }
                        }
                    }
                    if Instant::now() >= deadline {
                        return Err(anyhow::anyhow!(
                            "timed out waiting for repo test lock `{}`",
                            path.display()
                        ));
                    }
                    std::thread::sleep(TEST_LOCK_POLL_INTERVAL);
                }
                Err(error) => {
                    return Err(anyhow::anyhow!(
                        "create repo test lock `{}`: {error}",
                        path.display()
                    ));
                }
            }
        }
    }
}

impl Drop for RepoProcessLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_lock_owner(path: &Path) -> Result<()> {
    fs::write(path.join(TEST_LOCK_OWNER_FILE), std::process::id().to_string()).map_err(|error| {
        anyhow::anyhow!("write repo test lock owner `{}`: {error}", path.display())
    })
}

fn stale_repo_test_lock(path: &Path) -> Result<bool> {
    let owner_path = path.join(TEST_LOCK_OWNER_FILE);
    match fs::read_to_string(&owner_path) {
        Ok(raw_pid) => {
            let pid = raw_pid.trim().parse::<u32>().map_err(|error| {
                anyhow::anyhow!("parse repo test lock owner `{}`: {error}", owner_path.display())
            })?;
            Ok(!process_is_alive(pid))
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Ok(lock_is_older_than(path, TEST_LOCK_MISSING_OWNER_GRACE)?)
        }
        Err(error) => {
            Err(anyhow::anyhow!("read repo test lock owner `{}`: {error}", owner_path.display()))
        }
    }
}

fn lock_is_older_than(path: &Path, threshold: Duration) -> Result<bool> {
    let modified = match fs::metadata(path) {
        Ok(metadata) => metadata.modified().map_err(|error| {
            anyhow::anyhow!("read repo test lock metadata `{}`: {error}", path.display())
        })?,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(true),
        Err(error) => {
            return Err(anyhow::anyhow!(
                "read repo test lock metadata `{}`: {error}",
                path.display()
            ));
        }
    };
    let age = modified.elapsed().map_err(|error| {
        anyhow::anyhow!("measure repo test lock age `{}`: {error}", path.display())
    })?;
    Ok(age >= threshold)
}

#[cfg(unix)]
fn process_is_alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(not(unix))]
fn process_is_alive(_pid: u32) -> bool {
    true
}

#[test]
fn fastq_environment_slice_matches_committed_outputs() -> Result<()> {
    let _cwd_guard = CWD_LOCK.lock().map_err(|err| anyhow::anyhow!("cwd lock: {err}"))?;
    let _repo_lock = RepoProcessLock::acquire("benchmark-readiness-mutators")?;
    let _env_guard = EnvGuard::new()?;
    let root = repo_root()?;
    let compiled = compile_workspace(&root)?;

    assert_fastq_slice_rows(&compiled);
    assert_generated_output_inventory(&root)?;
    assert_committed_outputs_match(&root, &compiled)?;
    Ok(())
}

fn assert_fastq_slice_rows(compiled: &CompiledScience) {
    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads" && row.tool_id == "fastp" && row.is_default
    }));
    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads"
            && row.tool_id == "seqpurge"
            && row.tool_status == "disallowed"
    }));
    assert!(compiled
        .source_inventory
        .iter()
        .any(|row| row.source_id == "source.fastq.tool-registry"));
    assert!(compiled
        .fastq_container_reference_rows
        .iter()
        .any(|row| row.tool_id == "fastp" && row.version == "0.23.4"));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "fastp" && row.source_id == "source.fastq.tool.fastp.upstream"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "diamond"
            && row.stage_ids == "fastq.screen_taxonomy"
            && row.source_id == "source.fastq.tool.diamond.upstream"
            && row.backlog_status == "ready"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "dustmasker"
            && row.backlog_status == "ready"
            && row.locator
                == "https://www.ncbi.nlm.nih.gov/IEB/ToolBox/CPP_DOC/lxr/source/src/app/dustmasker/"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "fastp"
            && row.paper_root == "science/docs/upstream/papers/paper.fastq.fastp.chen-2018"
    }));
    assert!(compiled.fastq_paper_archive_rows.iter().any(|row| {
        row.tool_id == "atropos"
            && row.paper_id == "paper.fastq.atropos.didion-2017"
            && row.paper_status == "mapped"
    }));
    assert_eq!(compiled.index.source_inventory_rows, compiled.source_inventory.len());
    assert_eq!(compiled.index.source_archive_gap_rows, compiled.source_archive_gaps.len());
    assert_eq!(
        compiled
            .index
            .source_archive_summary
            .archive_status_counts
            .get("present")
            .copied()
            .unwrap_or_default(),
        compiled.source_inventory.iter().filter(|row| row.archive_status == "present").count()
    );
    assert_eq!(
        compiled
            .index
            .source_archive_summary
            .archive_status_counts
            .get("missing")
            .copied()
            .unwrap_or_default(),
        compiled.source_inventory.iter().filter(|row| row.archive_status == "missing").count()
    );
    assert_eq!(compiled.index.source_archive_summary.missing_tool_counts.len(), 0);
    assert_eq!(
        compiled.index.fastq_closure_summary.total_rows,
        compiled.fastq_closure_gate_rows.len()
    );
    assert_eq!(
        compiled.index.fastq_closure_summary.default_rows,
        compiled.fastq_closure_gate_rows.iter().filter(|row| row.is_default).count()
    );
    assert!(compiled
        .index
        .fastq_closure_summary
        .blocking_reason_counts
        .contains_key("missing_environment_qa_stage"));
    assert!(compiled
        .index
        .fastq_evidence_summary
        .prerequisite_counts
        .contains_key("missing_environment_qa_stage"));
    assert!(compiled
        .index
        .fastq_evidence_summary
        .default_risk_counts
        .contains_key("closure_prerequisite_blocked"));
}

fn assert_committed_outputs_match(root: &Path, compiled: &CompiledScience) -> Result<()> {
    assert_rendered(
        root,
        "science/generated/current/evidence/source_inventory.tsv",
        &source_inventory_tsv(&compiled.source_inventory),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/source_archive_gaps.tsv",
        &source_archive_gaps_tsv(&compiled.source_archive_gaps),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_container_reference_matrix.tsv",
        &fastq_container_reference_tsv(&compiled.fastq_container_reference_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_download_backlog.tsv",
        &fastq_download_backlog_tsv(&compiled.fastq_download_backlog_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_paper_archive_matrix.tsv",
        &fastq_paper_archive_tsv(&compiled.fastq_paper_archive_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/claim_evidence_map.tsv",
        &claim_evidence_tsv(&compiled.claim_evidence_map),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/decision_reasoning_map.tsv",
        &decision_reasoning_tsv(&compiled.decision_reasoning_map),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/binding_resolution.tsv",
        &binding_resolution_tsv(&compiled.binding_resolution),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv",
        &fastq_environment_tsv(&compiled.fastq_environment_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_closure_gate.tsv",
        &fastq_closure_gate_tsv(&compiled.fastq_closure_gate_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_truth_delta.tsv",
        &fastq_truth_delta_tsv(&compiled.fastq_truth_delta_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv",
        &fastq_missing_closure_prerequisites_tsv(&compiled.fastq_missing_closure_prerequisite_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv",
        &fastq_default_binding_risk_tsv(&compiled.fastq_default_binding_risk_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/unresolved_refs.json",
        &to_pretty_json(&compiled.unresolved_refs)?,
    )?;
    assert_rendered(
        root,
        "science/generated/indexes/science_index.json",
        &index_json(&compiled.index)?,
    )?;
    Ok(())
}

fn assert_generated_output_inventory(root: &Path) -> Result<()> {
    let evidence_root = root.join("science/generated/current/evidence");
    let mut actual = BTreeSet::new();
    for entry in
        fs::read_dir(&evidence_root).with_context(|| format!("read {}", evidence_root.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.file_name() != "README.md" {
            actual.insert(format!(
                "science/generated/current/evidence/{}",
                entry.file_name().to_string_lossy()
            ));
        }
    }
    actual.insert("science/generated/indexes/science_index.json".to_string());

    let expected = [
        "science/generated/current/evidence/binding_resolution.tsv",
        "science/generated/current/evidence/claim_evidence_map.tsv",
        "science/generated/current/evidence/decision_reasoning_map.tsv",
        "science/generated/current/evidence/fastq_closure_gate.tsv",
        "science/generated/current/evidence/fastq_container_reference_matrix.tsv",
        "science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv",
        "science/generated/current/evidence/fastq_download_backlog.tsv",
        "science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv",
        "science/generated/current/evidence/fastq_paper_archive_matrix.tsv",
        "science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv",
        "science/generated/current/evidence/fastq_truth_delta.tsv",
        "science/generated/current/evidence/source_archive_gaps.tsv",
        "science/generated/current/evidence/source_inventory.tsv",
        "science/generated/current/evidence/unresolved_refs.json",
        "science/generated/indexes/science_index.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected, "generated science output inventory changed");
    Ok(())
}

fn assert_rendered(root: &Path, rel_path: &str, expected: &str) -> Result<()> {
    let path = root.join(rel_path);
    let actual = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    assert_eq!(actual, expected, "generated output drifted at {rel_path}");
    Ok(())
}
