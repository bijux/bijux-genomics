//! Owner: bijux-dna-bench
//! Reversible migration of benchmark attempts retained below container-owned evidence roots.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use walkdir::WalkDir;

use bijux_dna_bench_model::{
    BenchmarkArchiveFileRecord, BenchmarkAttemptDisposition, BenchmarkAttemptMigrationEntry,
    BenchmarkAttemptMigrationExclusion, BenchmarkAttemptMigrationLedger,
    BenchmarkAttemptMigrationLedgerEntry, BenchmarkAttemptMigrationPlan,
    BenchmarkAttemptMigrationState, BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1,
};
pub use bijux_dna_bench_model::{
    BenchmarkAttemptMigrationVerification, BenchmarkAttemptMigrationVerificationEntry,
};

#[derive(Debug, Deserialize)]
struct PipelineContract {
    run: PipelineRun,
}

#[derive(Debug, Deserialize)]
struct PipelineRun {
    id: String,
    #[serde(default = "qualification_kind")]
    kind: String,
    tool: String,
}

#[derive(Debug, Deserialize)]
struct HistoricalArchive {
    schema_version: String,
    run_id: String,
    tool: String,
    slurm_job_id: String,
    #[serde(default)]
    disposition: Option<String>,
    evidence_sha256: String,
    accounting_sha256: String,
    files: Vec<BenchmarkArchiveFileRecord>,
}

fn qualification_kind() -> String {
    "qualification".to_string()
}

/// Build an immutable migration plan from historical container evidence.
///
/// # Errors
/// Returns an error when STUDY cannot be read or benchmark evidence is internally inconsistent.
pub fn plan_attempt_migration(
    study_root: &Path,
    created_at_utc: String,
) -> Result<BenchmarkAttemptMigrationPlan> {
    let study_root = study_root
        .canonicalize()
        .with_context(|| format!("canonicalize STUDY root {}", study_root.display()))?;
    let mut entries = Vec::new();
    let mut exclusions = Vec::new();
    for root in [
        study_root.join("containers/evidence/failures"),
        study_root.join("containers/evidence/superseded"),
        study_root.join("results/benchmark-attempts/failures"),
    ] {
        if !root.is_dir() {
            continue;
        }
        for source in sorted_child_directories(&root)? {
            classify_candidate(&study_root, &source, &mut entries, &mut exclusions)?;
        }
    }
    entries.sort_by(|left, right| left.source.cmp(&right.source));
    exclusions.sort_by(|left, right| left.source.cmp(&right.source));
    let identity_bytes = serde_json::to_vec(&(&entries, &exclusions))?;
    let plan_id = bijux_dna_infra::hash_bytes_sha256(&identity_bytes);
    Ok(BenchmarkAttemptMigrationPlan {
        schema_version: BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1.to_string(),
        plan_id,
        created_at_utc,
        entries,
        exclusions,
    })
}

/// Atomically write a migration plan.
///
/// # Errors
/// Returns an error when the plan is invalid or cannot be written.
pub fn write_attempt_migration_plan(
    path: &Path,
    plan: &BenchmarkAttemptMigrationPlan,
) -> Result<()> {
    validate_plan(plan)?;
    bijux_dna_infra::atomic_write_json(path, plan)
        .with_context(|| format!("write migration plan {}", path.display()))
}

/// Apply a migration plan and persist an interruption-safe ledger.
///
/// # Errors
/// Returns an error when evidence changed, paths conflict, or an atomic rename is unavailable.
pub fn apply_attempt_migration(
    study_root: &Path,
    plan_path: &Path,
    ledger_path: &Path,
    updated_at_utc: String,
) -> Result<BenchmarkAttemptMigrationLedger> {
    let study_root = canonical_study_root(study_root)?;
    let plan = load_plan(plan_path)?;
    let plan_sha256 = bijux_dna_infra::hash_file_sha256(plan_path)
        .with_context(|| format!("hash migration plan {}", plan_path.display()))?;
    let mut ledger = load_or_initialize_ledger(ledger_path, &plan, &plan_sha256, &updated_at_utc)?;
    for (index, entry) in plan.entries.iter().enumerate() {
        match ledger.entries[index].state {
            BenchmarkAttemptMigrationState::Applied => {
                verify_applied_entry(&study_root, entry)?;
            }
            BenchmarkAttemptMigrationState::RolledBack => {
                bail!("migration entry was rolled back; create a new plan before applying again")
            }
            BenchmarkAttemptMigrationState::Planned => {
                let source = path_from_study_uri(&study_root, &entry.source)?;
                let target = path_from_study_uri(&study_root, &entry.target)?;
                if target.exists() && !source.exists() {
                    verify_inventory(&target, &entry.files)?;
                } else {
                    if !source.is_dir() || target.exists() {
                        bail!(
                            "migration paths are not in the planned state: {} -> {}",
                            source.display(),
                            target.display()
                        );
                    }
                    verify_inventory(&source, &entry.files)?;
                    let target_parent =
                        target.parent().ok_or_else(|| anyhow!("migration target has no parent"))?;
                    bijux_dna_infra::ensure_dir(target_parent)?;
                    if !bijux_dna_infra::same_filesystem(&source, target_parent)? {
                        bail!("migration requires a same-filesystem rename: {}", entry.source);
                    }
                    bijux_dna_infra::rename(&source, &target)?;
                    verify_inventory(&target, &entry.files)?;
                }
                ledger.entries[index].state = BenchmarkAttemptMigrationState::Applied;
                ledger.updated_at_utc.clone_from(&updated_at_utc);
                write_ledger(ledger_path, &ledger)?;
            }
        }
    }
    Ok(ledger)
}

/// Verify every migration entry against its ledger state and return the after inventory.
///
/// # Errors
/// Returns an error when the plan, ledger, state, size, or digest differs.
pub fn verify_attempt_migration(
    study_root: &Path,
    plan_path: &Path,
    ledger_path: &Path,
) -> Result<BenchmarkAttemptMigrationVerification> {
    let study_root = canonical_study_root(study_root)?;
    let plan = load_plan(plan_path)?;
    let plan_sha256 = bijux_dna_infra::hash_file_sha256(plan_path)?;
    let ledger = load_ledger(ledger_path)?;
    validate_ledger(&ledger, &plan, &plan_sha256)?;
    let mut entries = Vec::with_capacity(plan.entries.len());
    for (entry, ledger_entry) in plan.entries.iter().zip(&ledger.entries) {
        let source = path_from_study_uri(&study_root, &entry.source)?;
        let target = path_from_study_uri(&study_root, &entry.target)?;
        let current_location = match ledger_entry.state {
            BenchmarkAttemptMigrationState::Applied => {
                verify_applied_entry(&study_root, entry)?;
                entry.target.clone()
            }
            BenchmarkAttemptMigrationState::Planned
            | BenchmarkAttemptMigrationState::RolledBack => {
                if target.exists() || !source.is_dir() {
                    bail!("migration source layout is not restored: {}", entry.source);
                }
                verify_inventory(&source, &entry.files)?;
                entry.source.clone()
            }
        };
        entries.push(BenchmarkAttemptMigrationVerificationEntry {
            source: entry.source.clone(),
            target: entry.target.clone(),
            state: ledger_entry.state,
            current_location,
            files: entry.files.clone(),
        });
    }
    Ok(BenchmarkAttemptMigrationVerification {
        schema_version: BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1.to_string(),
        plan_id: plan.plan_id,
        entries,
    })
}

/// Roll an applied migration back to the exact source layout.
///
/// # Errors
/// Returns an error when target evidence changed or the source cannot be restored atomically.
pub fn rollback_attempt_migration(
    study_root: &Path,
    plan_path: &Path,
    ledger_path: &Path,
    updated_at_utc: String,
) -> Result<BenchmarkAttemptMigrationLedger> {
    let study_root = canonical_study_root(study_root)?;
    let plan = load_plan(plan_path)?;
    let plan_sha256 = bijux_dna_infra::hash_file_sha256(plan_path)?;
    let mut ledger = load_ledger(ledger_path)?;
    validate_ledger(&ledger, &plan, &plan_sha256)?;
    for index in (0..plan.entries.len()).rev() {
        let entry = &plan.entries[index];
        let source = path_from_study_uri(&study_root, &entry.source)?;
        let target = path_from_study_uri(&study_root, &entry.target)?;
        match ledger.entries[index].state {
            BenchmarkAttemptMigrationState::Applied => {
                if source.exists() || !target.is_dir() {
                    bail!("rollback paths are not in the applied state: {}", entry.target);
                }
                verify_inventory(&target, &entry.files)?;
                let source_parent =
                    source.parent().ok_or_else(|| anyhow!("migration source has no parent"))?;
                bijux_dna_infra::ensure_dir(source_parent)?;
                if !bijux_dna_infra::same_filesystem(&target, source_parent)? {
                    bail!("rollback requires a same-filesystem rename: {}", entry.target);
                }
                bijux_dna_infra::rename(&target, &source)?;
                verify_inventory(&source, &entry.files)?;
            }
            BenchmarkAttemptMigrationState::Planned => {
                if target.exists() || !source.is_dir() {
                    bail!("planned migration entry cannot be rolled back safely: {}", entry.source);
                }
                verify_inventory(&source, &entry.files)?;
            }
            BenchmarkAttemptMigrationState::RolledBack => {
                if target.exists() || !source.is_dir() {
                    bail!("rolled-back migration entry drifted: {}", entry.source);
                }
                verify_inventory(&source, &entry.files)?;
            }
        }
        ledger.entries[index].state = BenchmarkAttemptMigrationState::RolledBack;
        ledger.updated_at_utc.clone_from(&updated_at_utc);
        write_ledger(ledger_path, &ledger)?;
    }
    Ok(ledger)
}

fn classify_candidate(
    study_root: &Path,
    source: &Path,
    entries: &mut Vec<BenchmarkAttemptMigrationEntry>,
    exclusions: &mut Vec<BenchmarkAttemptMigrationExclusion>,
) -> Result<()> {
    let source_uri = study_uri(study_root, source)?;
    let Some(pipeline_path) =
        [source.join("submission/config-pipeline.yml"), source.join("result/config-pipeline.yml")]
            .into_iter()
            .find(|path| path.is_file())
    else {
        exclusions.push(BenchmarkAttemptMigrationExclusion {
            source: source_uri,
            reason: "missing retained pipeline contract".to_string(),
        });
        return Ok(());
    };
    let pipeline: PipelineContract = serde_yaml::from_str(&fs::read_to_string(&pipeline_path)?)
        .with_context(|| format!("parse retained pipeline {}", pipeline_path.display()))?;
    if pipeline.run.kind != "benchmark" {
        exclusions.push(BenchmarkAttemptMigrationExclusion {
            source: source_uri,
            reason: format!("run.kind={} remains container-owned", pipeline.run.kind),
        });
        return Ok(());
    }
    let archive_path = source.join("archive.json");
    if !archive_path.is_file() {
        exclusions.push(BenchmarkAttemptMigrationExclusion {
            source: source_uri,
            reason: "benchmark material lacks archive.json and requires quarantine review"
                .to_string(),
        });
        return Ok(());
    }
    let archive: HistoricalArchive = read_json(&archive_path)?;
    let disposition = historical_disposition(&archive)?;
    validate_historical_archive(source, &pipeline, &archive)?;
    let bundle_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("historical attempt bundle name is not UTF-8"))?;
    let expected_name =
        format!("{}-{}-slurm-{}", pipeline.run.id, pipeline.run.tool, archive.slurm_job_id);
    if bundle_name != expected_name {
        bail!("historical attempt bundle identity drift: {bundle_name} != {expected_name}");
    }
    let scope = match disposition {
        BenchmarkAttemptDisposition::Failed => "failed",
        BenchmarkAttemptDisposition::Superseded => "superseded",
    };
    let target = study_root.join("results/benchmark-attempts").join(scope).join(bundle_name);
    if target.exists() {
        bail!("historical migration target already exists: {}", target.display());
    }
    entries.push(BenchmarkAttemptMigrationEntry {
        run_id: pipeline.run.id,
        case_id: None,
        attempt_id: format!("slurm-{}", archive.slurm_job_id),
        tool_id: pipeline.run.tool,
        slurm_job_id: archive.slurm_job_id,
        disposition,
        source: source_uri,
        target: study_uri(study_root, &target)?,
        files: inventory(source)?,
    });
    Ok(())
}

fn historical_disposition(archive: &HistoricalArchive) -> Result<BenchmarkAttemptDisposition> {
    if archive.schema_version == "bijux.container.failed_run_archive.v1" {
        return Ok(BenchmarkAttemptDisposition::Failed);
    }
    if archive.schema_version == "bijux.container.superseded_run_archive.v1"
        && archive.disposition.as_deref() == Some("superseded")
    {
        return Ok(BenchmarkAttemptDisposition::Superseded);
    }
    bail!("historical benchmark archive has unsupported disposition contract")
}

fn validate_historical_archive(
    root: &Path,
    pipeline: &PipelineContract,
    archive: &HistoricalArchive,
) -> Result<()> {
    if archive.run_id != pipeline.run.id || archive.tool != pipeline.run.tool {
        bail!("historical archive identity differs from retained pipeline contract");
    }
    if !archive.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("historical archive has invalid Slurm identity");
    }
    verify_declared_files(root, &archive.files)?;
    for (relative, expected) in [
        ("result/evidence.json", archive.evidence_sha256.as_str()),
        ("result/slurm-accounting.json", archive.accounting_sha256.as_str()),
    ] {
        let actual = bijux_dna_infra::hash_file_sha256(&root.join(relative))?;
        if actual != expected {
            bail!("historical archive digest mismatch: {relative}");
        }
    }
    Ok(())
}

fn verify_declared_files(root: &Path, files: &[BenchmarkArchiveFileRecord]) -> Result<()> {
    let mut paths = BTreeSet::new();
    for record in files {
        if !safe_relative_path(&record.path) || !paths.insert(record.path.as_str()) {
            bail!("historical archive declares an unsafe or duplicate file path");
        }
        let path = root.join(&record.path);
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("stat historical archive file {}", path.display()))?;
        if !metadata.is_file()
            || metadata.len() != record.size_bytes
            || bijux_dna_infra::hash_file_sha256(&path)? != record.sha256
        {
            bail!("historical archive file inventory mismatch: {}", record.path);
        }
    }
    Ok(())
}

fn sorted_child_directories(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.is_dir());
    paths.sort();
    Ok(paths)
}

fn inventory(root: &Path) -> Result<Vec<BenchmarkArchiveFileRecord>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).sort_by_file_name() {
        let entry = entry?;
        if entry.path() == root {
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            bail!("attempt archive contains a symlink: {}", entry.path().display());
        }
        if !metadata.is_file() {
            continue;
        }
        let relative = entry.path().strip_prefix(root)?.to_string_lossy().replace('\\', "/");
        files.push(BenchmarkArchiveFileRecord {
            path: relative,
            size_bytes: metadata.len(),
            sha256: bijux_dna_infra::hash_file_sha256(entry.path())?,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn verify_inventory(root: &Path, expected: &[BenchmarkArchiveFileRecord]) -> Result<()> {
    let actual = inventory(root)?;
    if actual != expected {
        bail!("attempt archive size/SHA-256 inventory changed: {}", root.display());
    }
    Ok(())
}

fn verify_applied_entry(study_root: &Path, entry: &BenchmarkAttemptMigrationEntry) -> Result<()> {
    let source = path_from_study_uri(study_root, &entry.source)?;
    let target = path_from_study_uri(study_root, &entry.target)?;
    if source.exists() || !target.is_dir() {
        bail!("migration entry is not in the applied layout: {}", entry.target);
    }
    verify_inventory(&target, &entry.files)
}

fn load_plan(path: &Path) -> Result<BenchmarkAttemptMigrationPlan> {
    let plan = read_json(path)?;
    validate_plan(&plan)?;
    Ok(plan)
}

fn validate_plan(plan: &BenchmarkAttemptMigrationPlan) -> Result<()> {
    if plan.schema_version != BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1 {
        bail!("unsupported benchmark attempt migration plan schema");
    }
    let identity_bytes = serde_json::to_vec(&(&plan.entries, &plan.exclusions))?;
    if plan.plan_id != bijux_dna_infra::hash_bytes_sha256(&identity_bytes) {
        bail!("benchmark attempt migration plan identity mismatch");
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for entry in &plan.entries {
        if !sources.insert(entry.source.as_str()) || !targets.insert(entry.target.as_str()) {
            bail!("benchmark attempt migration plan contains duplicate paths");
        }
        if entry.files.is_empty() || entry.attempt_id != format!("slurm-{}", entry.slurm_job_id) {
            bail!("benchmark attempt migration plan contains incomplete identity");
        }
        validate_study_uri(&entry.source)?;
        validate_study_uri(&entry.target)?;
    }
    Ok(())
}

fn load_or_initialize_ledger(
    ledger_path: &Path,
    plan: &BenchmarkAttemptMigrationPlan,
    plan_sha256: &str,
    updated_at_utc: &str,
) -> Result<BenchmarkAttemptMigrationLedger> {
    if ledger_path.exists() {
        let ledger = load_ledger(ledger_path)?;
        validate_ledger(&ledger, plan, plan_sha256)?;
        return Ok(ledger);
    }
    let ledger = BenchmarkAttemptMigrationLedger {
        schema_version: BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1.to_string(),
        plan_sha256: plan_sha256.to_string(),
        updated_at_utc: updated_at_utc.to_string(),
        entries: plan
            .entries
            .iter()
            .map(|entry| BenchmarkAttemptMigrationLedgerEntry {
                source: entry.source.clone(),
                target: entry.target.clone(),
                state: BenchmarkAttemptMigrationState::Planned,
            })
            .collect(),
    };
    write_ledger(ledger_path, &ledger)?;
    Ok(ledger)
}

fn load_ledger(path: &Path) -> Result<BenchmarkAttemptMigrationLedger> {
    read_json(path)
}

fn validate_ledger(
    ledger: &BenchmarkAttemptMigrationLedger,
    plan: &BenchmarkAttemptMigrationPlan,
    plan_sha256: &str,
) -> Result<()> {
    if ledger.schema_version != BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1
        || ledger.plan_sha256 != plan_sha256
        || ledger.entries.len() != plan.entries.len()
    {
        bail!("benchmark attempt migration ledger does not identify the exact plan");
    }
    for (ledger_entry, plan_entry) in ledger.entries.iter().zip(&plan.entries) {
        if ledger_entry.source != plan_entry.source || ledger_entry.target != plan_entry.target {
            bail!("benchmark attempt migration ledger path identity drift");
        }
    }
    Ok(())
}

fn write_ledger(path: &Path, ledger: &BenchmarkAttemptMigrationLedger) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, ledger)
        .with_context(|| format!("write migration ledger {}", path.display()))
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).with_context(|| format!("read JSON {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse JSON {}", path.display()))
}

fn canonical_study_root(study_root: &Path) -> Result<PathBuf> {
    study_root
        .canonicalize()
        .with_context(|| format!("canonicalize STUDY root {}", study_root.display()))
}

fn study_uri(study_root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(study_root)
        .with_context(|| format!("path is outside STUDY: {}", path.display()))?;
    Ok(format!("study://{}", relative.to_string_lossy().replace('\\', "/")))
}

fn path_from_study_uri(study_root: &Path, uri: &str) -> Result<PathBuf> {
    validate_study_uri(uri)?;
    let relative = uri.strip_prefix("study://").ok_or_else(|| anyhow!("invalid STUDY URI"))?;
    Ok(study_root.join(relative))
}

fn validate_study_uri(uri: &str) -> Result<()> {
    let relative =
        uri.strip_prefix("study://").ok_or_else(|| anyhow!("migration path is not a STUDY URI"))?;
    if !safe_relative_path(relative) {
        bail!("migration STUDY URI is unsafe: {uri}");
    }
    Ok(())
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && Path::new(value).components().all(|component| matches!(component, Component::Normal(_)))
}
