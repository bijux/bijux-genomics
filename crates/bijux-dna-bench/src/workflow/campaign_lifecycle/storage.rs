//! Owner: bijux-dna-bench
//! Crash-recoverable publication of a complete planned campaign cycle.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_stage_campaign_registry;
use bijux_dna_bench_model::{
    validate_campaign_plan, validate_execution_spec, validate_stage_plan_manifest,
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkCampaignPlan,
    BenchmarkExecutionSpec, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BenchmarkStageCampaignRegistry, BenchmarkStagePlanManifest, BenchmarkStagePlanReference,
    BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1,
};
use bijux_dna_runtime::{
    benchmark_attempt_directory, ensure_attempt_lifecycle, load_attempt_lifecycle,
};
use bijux_dna_stage_contract::StagePlanV1;

const MAX_STAGE_PLAN_BYTES: usize = 16 * 1024 * 1024;

/// Recover the complete executed retry lineage ending at an explicit attempt tip for each tool.
///
/// Untouched planned siblings are ignored because they contain no execution evidence and cannot
/// compete with the selected lineage. Every other detached record fails closed.
///
/// # Errors
/// Returns an error when the campaign is unknown, any tool tip or predecessor is missing, a chain
/// cycles or is nonterminal, persisted identity differs from its path, or detached execution
/// evidence would make lineage selection ambiguous.
pub fn recover_stage_campaign_retry_lineage(
    benchmark_root: &Path,
    registry: &BenchmarkStageCampaignRegistry,
    run_id: &str,
    campaign_id: &str,
    tip_attempt_id: &str,
) -> Result<Vec<BenchmarkAttemptRecordV2>> {
    validate_stage_campaign_registry(registry).context("validate benchmark campaign registry")?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == run_id)
        .with_context(|| format!("benchmark campaign registry does not define {run_id}"))?;
    let run = BenchmarkRunIdentity {
        run_id: campaign.run_id.clone(),
        domain: campaign.domain.clone(),
        stage_id: campaign.stage_id.clone(),
    };
    let mut tools = campaign.tool_ids.clone();
    tools.sort();
    let mut lineage = Vec::new();
    for tool_id in tools {
        let tip = BenchmarkAttemptIdentity {
            campaign_id: campaign_id.to_string(),
            run: run.clone(),
            attempt_id: tip_attempt_id.to_string(),
            tool_id,
        };
        let records = load_campaign_tool_attempts(benchmark_root, &tip)?;
        let mut selected = BTreeSet::new();
        let mut chain = Vec::new();
        let mut current = tip.attempt_id.clone();
        loop {
            if !selected.insert(current.clone()) {
                bail!("benchmark retry lineage contains a cycle");
            }
            let (record, _) = records
                .get(&current)
                .with_context(|| format!("benchmark retry predecessor is absent: {current}"))?;
            if !is_terminal_history(record) {
                bail!("benchmark retry lineage contains a nonterminal attempt");
            }
            chain.push(record.clone());
            let Some(predecessor) = &record.supersedes_attempt_id else {
                break;
            };
            current.clone_from(predecessor);
        }
        for (attempt_id, (record, directory)) in &records {
            if !selected.contains(attempt_id) && !is_untouched_planned_sibling(record, directory)? {
                bail!("benchmark retry lineage has detached execution evidence: {attempt_id}");
            }
        }
        chain.reverse();
        lineage.extend(chain);
    }
    Ok(lineage)
}

fn load_campaign_tool_attempts(
    benchmark_root: &Path,
    tip: &BenchmarkAttemptIdentity,
) -> Result<BTreeMap<String, (BenchmarkAttemptRecordV2, PathBuf)>> {
    let tip_directory = benchmark_attempt_directory(benchmark_root, tip)?;
    let campaign_directory =
        tip_directory.parent().context("benchmark retry tip has no campaign directory")?;
    let metadata = fs::symlink_metadata(campaign_directory).with_context(|| {
        format!("inspect benchmark retry campaign {}", campaign_directory.display())
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("benchmark retry campaign path is not an owned directory");
    }
    let mut records = BTreeMap::new();
    for entry in fs::read_dir(campaign_directory).with_context(|| {
        format!("read benchmark retry campaign {}", campaign_directory.display())
    })? {
        let entry = entry.context("read benchmark retry attempt directory entry")?;
        let file_type = entry.file_type().context("inspect benchmark retry attempt entry")?;
        if file_type.is_symlink() || !file_type.is_dir() {
            bail!("benchmark retry campaign contains a non-directory attempt entry");
        }
        let attempt_id = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("benchmark retry attempt identity is not UTF-8"))?;
        let mut identity = tip.clone();
        identity.attempt_id.clone_from(&attempt_id);
        let record = load_attempt_lifecycle(benchmark_root, &identity)
            .with_context(|| format!("load benchmark retry attempt {attempt_id}"))?;
        if records.insert(attempt_id.clone(), (record, entry.path())).is_some() {
            bail!("benchmark retry campaign duplicates attempt identity: {attempt_id}");
        }
    }
    Ok(records)
}

fn is_untouched_planned_sibling(
    record: &BenchmarkAttemptRecordV2,
    directory: &Path,
) -> Result<bool> {
    if record.supersedes_attempt_id.is_some()
        || record.transitions.len() != 1
        || record.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        return Ok(false);
    }
    let mut entries = BTreeMap::new();
    for entry in fs::read_dir(directory)
        .with_context(|| format!("read detached benchmark attempt {}", directory.display()))?
    {
        let entry = entry.context("read detached benchmark attempt entry")?;
        let file_type = entry.file_type().context("inspect detached benchmark attempt entry")?;
        if file_type.is_symlink() || entries.insert(entry.file_name(), file_type).is_some() {
            return Ok(false);
        }
    }
    let expected = [
        "accounting",
        "attempt.json",
        "logs",
        "manifests",
        "outputs",
        "receipts",
        "science",
        "validation",
    ]
    .into_iter()
    .map(std::ffi::OsString::from)
    .collect::<BTreeSet<_>>();
    if entries.keys().cloned().collect::<BTreeSet<_>>() != expected
        || !entries
            .get(&std::ffi::OsString::from("attempt.json"))
            .is_some_and(std::fs::FileType::is_file)
    {
        return Ok(false);
    }
    for name in ["accounting", "logs", "outputs", "receipts", "science", "validation"] {
        if !entries.get(&std::ffi::OsString::from(name)).is_some_and(std::fs::FileType::is_dir)
            || fs::read_dir(directory.join(name))?.next().is_some()
        {
            return Ok(false);
        }
    }
    let manifests = directory.join("manifests");
    if !entries.get(&std::ffi::OsString::from("manifests")).is_some_and(std::fs::FileType::is_dir) {
        return Ok(false);
    }
    let manifest_entries = fs::read_dir(manifests)?.collect::<std::io::Result<Vec<_>>>()?;
    Ok(manifest_entries.len() == 1
        && manifest_entries[0].file_name() == "attempt-layout.json"
        && manifest_entries[0].file_type()?.is_file())
}

/// Persist every planned campaign attempt under its immutable run/tool/campaign/attempt identity.
///
/// Existing attempts with the same immutable planned origin are accepted after later lifecycle
/// transitions. A complete preflight rejects known conflicts before any missing attempt is created.
///
/// # Errors
/// Returns an error when the cycle is invalid or not wholly planned, an existing identity differs,
/// or runtime cannot atomically initialize an attempt.
pub fn ensure_stage_campaign_attempts(
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
) -> Result<Vec<PathBuf>> {
    validate_campaign_plan(plan).context("validate benchmark campaign plan")?;
    let mut attempts = plan.cycle.attempts.iter().collect::<Vec<_>>();
    attempts.sort_by(|left, right| attempt_sort_key(left).cmp(&attempt_sort_key(right)));
    for attempt in &attempts {
        let directory = benchmark_attempt_directory(benchmark_root, &attempt.identity)?;
        if is_planned_origin(attempt) {
            if directory.exists() {
                validate_attempt_origin(benchmark_root, attempt, &directory)?;
            }
        } else if is_terminal_history(attempt) {
            if !directory.exists() {
                bail!(
                    "benchmark retry history is absent from persistent storage: {}",
                    directory.display()
                );
            }
            let existing = load_attempt_lifecycle(benchmark_root, &attempt.identity)?;
            if existing != **attempt {
                bail!(
                    "benchmark retry history differs from persistent evidence: {}",
                    directory.display()
                );
            }
        } else {
            bail!("benchmark campaign plan contains unsupported active history");
        }
    }

    attempts
        .into_iter()
        .filter(|attempt| is_planned_origin(attempt))
        .map(|attempt| {
            let directory = benchmark_attempt_directory(benchmark_root, &attempt.identity)?;
            if directory.exists() {
                validate_attempt_origin(benchmark_root, attempt, &directory)?;
                Ok(directory)
            } else {
                ensure_attempt_lifecycle(benchmark_root, attempt)
                    .context("ensure benchmark campaign attempt")
            }
        })
        .collect()
}

fn is_planned_origin(attempt: &BenchmarkAttemptRecordV2) -> bool {
    attempt.transitions.len() == 1
        && attempt.current_status() == Some(BenchmarkLifecycleStatus::Planned)
}

fn is_terminal_history(attempt: &BenchmarkAttemptRecordV2) -> bool {
    matches!(
        attempt.current_status(),
        Some(
            BenchmarkLifecycleStatus::Passed
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::Cancelled
                | BenchmarkLifecycleStatus::Superseded
                | BenchmarkLifecycleStatus::BlockedExternal
        )
    )
}

fn validate_attempt_origin(
    benchmark_root: &Path,
    planned: &BenchmarkAttemptRecordV2,
    directory: &Path,
) -> Result<()> {
    let existing = load_attempt_lifecycle(benchmark_root, &planned.identity)?;
    if existing.identity != planned.identity
        || existing.supersedes_attempt_id != planned.supersedes_attempt_id
        || existing.transitions.first() != planned.transitions.first()
    {
        bail!(
            "benchmark campaign attempt conflicts with immutable origin evidence: {}",
            directory.display()
        );
    }
    Ok(())
}

/// Load and validate one persisted execution specification for an initialized attempt.
///
/// # Errors
/// Returns an error when the attempt or specification is missing, malformed, symlinked, or bound
/// to a different immutable result identity.
pub fn load_execution_spec(
    benchmark_root: &Path,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
) -> Result<BenchmarkExecutionSpec> {
    let directory = benchmark_attempt_directory(benchmark_root, identity)?;
    load_attempt_lifecycle(benchmark_root, identity).context("load execution attempt lifecycle")?;
    let path = directory.join("execution-spec.json");
    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("inspect benchmark execution spec {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark execution specification is not a regular file");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
        .with_context(|| format!("read benchmark execution spec {}", path.display()))?;
    let spec: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark execution spec {}", path.display()))?;
    validate_execution_spec(&spec).context("validate persisted execution specification")?;
    if spec.identity != *identity || Path::new(&spec.outputs.result_root) != directory {
        bail!("benchmark execution specification differs from immutable attempt identity");
    }
    Ok(spec)
}

/// Persist one execution specification beside its initialized attempt lifecycle.
///
/// Existing structurally identical content is accepted for crash recovery. The result root and
/// planned attempt evidence must already match the same immutable identity.
///
/// # Errors
/// Returns an error when the specification is invalid, its attempt is absent or no longer wholly
/// planned, its result root differs from runtime path policy, or existing content conflicts.
pub fn ensure_execution_spec(
    benchmark_root: &Path,
    spec: &BenchmarkExecutionSpec,
) -> Result<PathBuf> {
    validate_execution_spec(spec).context("validate benchmark execution specification")?;
    let directory = benchmark_attempt_directory(benchmark_root, &spec.identity)?;
    if directory != Path::new(&spec.outputs.result_root) {
        bail!("benchmark execution result root differs from immutable attempt path");
    }
    let attempt = load_attempt_lifecycle(benchmark_root, &spec.identity)
        .context("load initialized benchmark attempt")?;
    if attempt.transitions.len() != 1
        || attempt.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("benchmark execution specification requires a wholly planned attempt");
    }
    let path = directory.join("execution-spec.json");
    match fs::symlink_metadata(&path) {
        Ok(metadata) => {
            if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                bail!("benchmark execution specification is not a regular file");
            }
            let bytes = bijux_dna_infra::read_to_end_bounded(&path, 16 * 1024 * 1024)
                .with_context(|| format!("read benchmark execution spec {}", path.display()))?;
            let existing: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse benchmark execution spec {}", path.display()))?;
            validate_execution_spec(&existing)
                .context("validate existing execution specification")?;
            if existing != *spec {
                bail!(
                    "benchmark execution specification conflicts with immutable attempt evidence"
                );
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(&path, spec)
                .with_context(|| format!("write benchmark execution spec {}", path.display()))?;
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect benchmark execution spec {}", path.display()));
        }
    }
    Ok(path)
}

/// Persist immutable measured-repetition execution-unit plans and their checksum-bound manifest.
///
/// Existing byte-identical evidence is accepted for crash recovery. Any identity, count, path, or
/// byte conflict fails closed.
///
/// # Errors
/// Returns an error when execution authority is absent or any plan differs from the attempt,
/// measured repetition policy, canonical path layout, or previously persisted evidence.
pub fn ensure_stage_plan_manifest(
    benchmark_root: &Path,
    spec: &BenchmarkExecutionSpec,
    plans: &[StagePlanV1],
) -> Result<BenchmarkStagePlanManifest> {
    validate_execution_spec(spec).context("validate stage-plan execution specification")?;
    let directory = benchmark_attempt_directory(benchmark_root, &spec.identity)?;
    let execution_spec_path = directory.join("execution-spec.json");
    let persisted_spec = load_execution_spec(benchmark_root, &spec.identity)?;
    if persisted_spec != *spec {
        bail!("stage-plan execution specification differs from persisted authority");
    }
    let repetition_count = usize::try_from(spec.measurement.measured_repetitions)?;
    if plans.len() < repetition_count || !plans.len().is_multiple_of(repetition_count) {
        bail!("stage-plan count does not close the measured repetition policy");
    }
    let stage_plan_root = directory.join("stage-plans");
    ensure_owned_directory(&stage_plan_root, "benchmark stage-plan root")?;
    let mut references = Vec::with_capacity(plans.len());
    let mut ordered_plans = plans.iter().collect::<Vec<_>>();
    ordered_plans.sort_by(|left, right| {
        left.stage_instance_id
            .as_ref()
            .map(ToString::to_string)
            .cmp(&right.stage_instance_id.as_ref().map(ToString::to_string))
    });
    for plan in ordered_plans {
        let (repetition, execution_unit_id, instance) = stage_plan_identity(plan)?;
        validate_stage_plan_identity(spec, plan, &instance, &directory)?;
        let relative_path = format!("stage-plans/repetition-{repetition}/{execution_unit_id}.json");
        let path = directory.join(&relative_path);
        let bytes = serde_json::to_vec_pretty(plan).context("serialize benchmark stage plan")?;
        ensure_immutable_bytes(&path, &bytes, "benchmark stage plan")?;
        references.push(BenchmarkStagePlanReference {
            repetition,
            execution_unit_id,
            stage_instance_id: instance,
            path: relative_path,
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
        });
    }
    let manifest = BenchmarkStagePlanManifest {
        schema_version: BENCHMARK_STAGE_PLAN_MANIFEST_SCHEMA_V1.to_owned(),
        identity: spec.identity.clone(),
        execution_spec_sha256: bijux_dna_infra::hash_file_sha256(&execution_spec_path)?,
        plans: references,
    };
    validate_stage_plan_manifest(&manifest)?;
    ensure_immutable_json(
        &directory.join("stage-plan-manifest.json"),
        &manifest,
        "benchmark stage-plan manifest",
    )?;
    Ok(manifest)
}

/// Load and validate every immutable measured-repetition stage plan for an attempt.
///
/// # Errors
/// Returns an error when the manifest, execution specification, any plan bytes, or plan identity
/// differs from persisted authority.
pub fn load_stage_plan_manifest(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<(BenchmarkStagePlanManifest, Vec<StagePlanV1>)> {
    let directory = benchmark_attempt_directory(benchmark_root, identity)?;
    let spec = load_execution_spec(benchmark_root, identity)?;
    let manifest_path = directory.join("stage-plan-manifest.json");
    let manifest: BenchmarkStagePlanManifest = read_bounded_json(&manifest_path)?;
    validate_stage_plan_manifest(&manifest)?;
    if manifest.identity != *identity
        || manifest.execution_spec_sha256
            != bijux_dna_infra::hash_file_sha256(&directory.join("execution-spec.json"))?
        || manifest.plans.last().map(|reference| reference.repetition)
            != Some(spec.measurement.measured_repetitions)
    {
        bail!("benchmark stage-plan manifest differs from execution authority");
    }
    let mut plans = Vec::with_capacity(manifest.plans.len());
    for reference in &manifest.plans {
        let path = directory.join(&reference.path);
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("inspect benchmark stage plan {}", path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("benchmark stage plan is not a regular file: {}", path.display());
        }
        let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_STAGE_PLAN_BYTES)?;
        if bijux_dna_infra::hash_bytes_sha256(&bytes) != reference.sha256 {
            bail!("benchmark stage-plan bytes differ from manifest authority");
        }
        let plan: StagePlanV1 = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse benchmark stage plan {}", path.display()))?;
        validate_stage_plan_identity(&spec, &plan, &reference.stage_instance_id, &directory)?;
        plans.push(plan);
    }
    Ok((manifest, plans))
}

fn validate_stage_plan_identity(
    spec: &BenchmarkExecutionSpec,
    plan: &StagePlanV1,
    stage_instance_id: &str,
    attempt_directory: &Path,
) -> Result<()> {
    let expected_image_digest = format!("sha256:{}", spec.image.sha256);
    if plan.stage_id.as_str() != spec.identity.run.stage_id
        || plan.tool_id.as_str() != spec.identity.tool_id
        || plan.stage_instance_id.as_ref().map(ToString::to_string).as_deref()
            != Some(stage_instance_id)
        || plan.stage_version.0 != 1
        || plan.tool_version.trim().is_empty()
        || plan.image.image != spec.image.path
        || plan.image.digest.as_deref() != Some(expected_image_digest.as_str())
        || plan.command.template.is_empty()
        || plan.command.template.iter().any(String::is_empty)
        || plan.resources.threads != spec.resources.cpus
        || plan.io.inputs.is_empty()
        || plan.io.outputs.is_empty()
        || plan.out_dir != attempt_directory.join("repetitions").join(stage_instance_id)
    {
        bail!("benchmark stage plan differs from immutable attempt identity");
    }
    Ok(())
}

fn stage_plan_identity(plan: &StagePlanV1) -> Result<(u32, String, String)> {
    let instance = plan
        .stage_instance_id
        .as_ref()
        .map(ToString::to_string)
        .context("benchmark stage plan omits execution-unit identity")?;
    let suffix = instance
        .strip_prefix("measured-repetition-")
        .context("benchmark stage plan has a noncanonical instance identity")?;
    let (repetition, execution_unit_id) = suffix
        .split_once("--")
        .context("benchmark stage plan omits repetition or execution-unit identity")?;
    let repetition = repetition.parse::<u32>().context("parse stage-plan repetition identity")?;
    if repetition == 0 || execution_unit_id.is_empty() {
        bail!("benchmark stage plan has an invalid repetition or execution-unit identity");
    }
    Ok((repetition, execution_unit_id.to_owned(), instance))
}

fn read_bounded_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect immutable JSON {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("immutable JSON is not a regular file: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_STAGE_PLAN_BYTES)?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse immutable JSON {}", path.display()))
}

fn ensure_immutable_json<T: serde::Serialize>(path: &Path, value: &T, label: &str) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure_immutable_bytes(path, &bytes, label)
}

fn ensure_immutable_bytes(path: &Path, bytes: &[u8], label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() || fs::read(path)? != bytes
            {
                bail!("{label} conflicts with immutable evidence: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, bytes)?;
            Ok(())
        }
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn ensure_owned_directory(path: &Path, label: &str) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                bail!("{label} is not an owned directory: {}", path.display());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::ensure_dir(path)?;
        }
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {label} {}", path.display()))
        }
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not an owned directory: {}", path.display());
    }
    Ok(())
}

fn attempt_sort_key(attempt: &BenchmarkAttemptRecordV2) -> (&str, &str, &str, &str, &str) {
    (
        &attempt.identity.run.run_id,
        &attempt.identity.run.stage_id,
        &attempt.identity.tool_id,
        &attempt.identity.campaign_id,
        &attempt.identity.attempt_id,
    )
}
