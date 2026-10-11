//! Import, selection, ledger, and publication workflows for container run evidence.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::validation::{run_directory_identity, run_number, validate_run_attempt_ledger_value};
use super::{
    render_run_evidence_markdown, validate_run_attempt_ledger, validate_run_evidence_index,
    validate_run_evidence_root, AttemptExecutionStatus, ContainerAttemptIdentity,
    FinalRunEvidencePlan, RunAttemptLedger, RunAttemptLedgerRow, RunAttemptLedgerSummary,
    RunEvidenceFileIdentity, RunEvidenceImportRequest, RunEvidenceIndex, RunEvidenceLock,
    RunEvidenceRecord, RunEvidenceSummary, ScientificAcceptanceStatus,
    CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1, CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2,
    CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1,
};

const COPIED_EVIDENCE: [&str; 5] = [
    "config-governance.yml",
    "config-pipeline.yml",
    "config-credential.yml",
    "evidence.json",
    "slurm-accounting.json",
];

/// Create or recover one immutable container-owned attempt layout.
///
/// Container and benchmark roots are validated together before any path is created, preventing
/// the two evidence owners from sharing or nesting their permanent attempt histories.
///
/// # Errors
/// Returns an error when owner roots overlap, identity segments are unsafe, or layout publication
/// or recovery fails.
pub fn ensure_container_attempt_layout(
    container_root: &Path,
    benchmark_root: &Path,
    identity: &ContainerAttemptIdentity,
) -> Result<bijux_dna_infra::AttemptLayoutPaths> {
    bijux_dna_infra::validate_attempt_owner_roots(container_root, benchmark_root)
        .context("validate container and benchmark attempt roots")?;
    let shared_identity = bijux_dna_infra::AttemptLayoutIdentity {
        run_directory: identity.run_directory.clone(),
        tool_id: identity.tool_id.clone(),
        campaign_id: identity.campaign_id.clone(),
        attempt_id: identity.attempt_id.clone(),
    };
    bijux_dna_infra::ensure_attempt_layout(
        container_root,
        bijux_dna_infra::AttemptEvidenceOwner::Containers,
        &shared_identity,
    )
    .context("ensure governed container attempt layout")
}

/// Import immutable result evidence without replacing an existing selected-run directory.
///
/// # Errors
/// Returns an error when source contracts are malformed, evidence cannot be copied, or a target
/// identity already exists.
pub fn import_run_evidence(request: &RunEvidenceImportRequest) -> Result<usize> {
    let result_root = request
        .result_root
        .canonicalize()
        .with_context(|| format!("resolve result root {}", request.result_root.display()))?;
    bijux_dna_infra::ensure_dir(&request.output_root)?;
    let mut admitted = 0;
    for (run_id, tool, result) in result_directories(&result_root)? {
        let number = run_number(&run_id)?;
        if request.minimum_run.is_some_and(|minimum| number < minimum)
            || request.maximum_run.is_some_and(|maximum| number > maximum)
        {
            continue;
        }
        let mut record = evidence_record(&run_id, &tool, &result, &result_root)?;
        let target = request.output_root.join(format!("{run_id}-{tool}"));
        if target.exists() {
            bail!("refusing to replace tracked run evidence: {}", target.display());
        }
        fs::create_dir(&target)
            .with_context(|| format!("create tracked run directory {}", target.display()))?;
        let import_result = (|| {
            for name in COPIED_EVIDENCE {
                let source = result.join(name);
                if !source.is_file() {
                    continue;
                }
                let destination = target.join(name);
                bijux_dna_infra::copy_file(&source, &destination)?;
                record.tracked_files.insert(
                    name.to_string(),
                    RunEvidenceFileIdentity {
                        path: None,
                        size_bytes: destination.metadata()?.len(),
                        sha256: bijux_dna_infra::hash_file_sha256(&destination)?,
                    },
                );
            }
            bijux_dna_infra::atomic_create_json(&target.join("evidence.lock.json"), &record)?;
            Ok::<(), anyhow::Error>(())
        })();
        if let Err(error) = import_result {
            let _ = bijux_dna_infra::remove_dir_all(&target);
            return Err(error);
        }
        admitted += 1;
    }
    Ok(admitted)
}

/// Derive final selected runs and a checksummed superseded-attempt ledger without mutation.
///
/// # Errors
/// Returns an error when no passing run exists for a tool or existing ledger history is invalid.
pub fn plan_final_run_evidence(root: &Path, ledger_path: &Path) -> Result<FinalRunEvidencePlan> {
    let records = validate_run_evidence_root(root)?;
    let (selected, superseded) = select_final_evidence(&records)?;
    let final_by_tool = selected
        .iter()
        .map(|record| (record.tool.as_str(), record.run_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut attempts =
        if ledger_path.is_file() { load_attempt_ledger(ledger_path)?.attempts } else { Vec::new() }
            .into_iter()
            .map(|attempt| (attempt.run_id.clone(), attempt))
            .collect::<BTreeMap<_, _>>();
    for record in &superseded {
        let lock_path =
            root.join(format!("{}-{}", record.run_id, record.tool)).join("evidence.lock.json");
        attempts.insert(
            record.run_id.clone(),
            RunAttemptLedgerRow {
                run_id: record.run_id.clone(),
                tool: record.tool.clone(),
                execution_status: record.execution_status,
                repository_commit: record.repository_commit.clone(),
                slurm_job_id: record.slurm_job_id.clone(),
                source_result: record.source_result.clone(),
                source_file_count: record.source_files.len(),
                evidence_lock_sha256: bijux_dna_infra::hash_file_sha256(&lock_path)?,
                failures: record.failures.clone(),
                superseded_by: final_by_tool[record.tool.as_str()].to_string(),
            },
        );
    }
    for attempt in attempts.values_mut() {
        let Some(final_run_id) = final_by_tool.get(attempt.tool.as_str()) else {
            bail!("{}: attempt ledger has no final published execution", attempt.tool);
        };
        attempt.superseded_by = (*final_run_id).to_string();
    }
    let attempts = attempts.into_values().collect::<Vec<_>>();
    let ledger = RunAttemptLedger {
        schema_version: CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1.to_string(),
        summary: RunAttemptLedgerSummary {
            published_run_count: selected.len(),
            superseded_attempt_count: attempts.len(),
            execution_status: status_counts(
                attempts.iter().map(|attempt| attempt.execution_status.as_str()),
            ),
        },
        attempts,
    };
    validate_run_attempt_ledger_value(&ledger, &selected, "derived attempt ledger")?;
    Ok(FinalRunEvidencePlan {
        ledger,
        selected_run_count: selected.len(),
        superseded_run_directories: superseded
            .iter()
            .map(|record| root.join(format!("{}-{}", record.run_id, record.tool)))
            .collect(),
    })
}

/// Apply a freshly derived final-evidence plan and remove only its superseded tracked copies.
///
/// # Errors
/// Returns an error when the plan is stale, the ledger cannot be written, or removal fails.
pub fn apply_final_run_evidence_plan(
    root: &Path,
    ledger_path: &Path,
    plan: &FinalRunEvidencePlan,
) -> Result<()> {
    let current = plan_final_run_evidence(root, ledger_path)?;
    if current != *plan {
        bail!("final run-evidence plan is stale");
    }
    bijux_dna_infra::atomic_write_json(ledger_path, &plan.ledger)?;
    for path in &plan.superseded_run_directories {
        bijux_dna_infra::remove_dir_all(path)?;
    }
    let selected = validate_run_evidence_root(root)?;
    validate_final_evidence(&selected)?;
    validate_run_attempt_ledger(ledger_path, &selected)?;
    Ok(())
}

/// Derive the selected-run JSON publication from locks, pipeline contracts, and the ledger.
///
/// # Errors
/// Returns an error when selected evidence, ledger history, or pipeline identity is invalid.
pub fn derive_run_evidence_index(root: &Path) -> Result<RunEvidenceIndex> {
    let records = validate_run_evidence_root(root)?;
    validate_final_evidence(&records)?;
    validate_run_attempt_ledger(&root.join("attempt-ledger.json"), &records)?;
    let mut runs = Vec::with_capacity(records.len());
    for record in &records {
        let run_path = root.join(format!("{}-{}", record.run_id, record.tool));
        let identity = pipeline_identity(&run_path.join("config-pipeline.yml"))?;
        runs.push(RunEvidenceRecord {
            campaign: identity.campaign,
            corpus: identity.corpus,
            domain: identity.domain,
            evidence_lock_sha256: bijux_dna_infra::hash_file_sha256(
                &run_path.join("evidence.lock.json"),
            )?,
            execution_status: record.execution_status,
            pipeline_schema_version: identity.pipeline_schema_version,
            repository_commit: record.repository_commit.clone(),
            run_id: record.run_id.clone(),
            sample: identity.sample,
            scientific_status: record.scientific_status,
            slurm_job_id: record.slurm_job_id.clone(),
            source_file_count: u64::try_from(record.source_files.len())?,
            source_result: record.source_result.clone(),
            stage: identity.stage,
            tool: record.tool.clone(),
        });
    }
    let tools = runs.iter().map(|record| record.tool.as_str()).collect::<BTreeSet<_>>();
    let index = RunEvidenceIndex {
        schema_version: CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2.to_string(),
        summary: RunEvidenceSummary {
            run_count: runs.len(),
            tool_count: tools.len(),
            execution_status: status_counts(
                runs.iter().map(|record| record.execution_status.as_str()),
            ),
            scientific_status: status_counts(
                runs.iter().map(|record| record.scientific_status.as_str()),
            ),
        },
        runs,
    };
    validate_run_evidence_index(&index)?;
    Ok(index)
}

/// Verify checked JSON and Markdown publications against selected run evidence.
///
/// # Errors
/// Returns an error when derivation fails or either publication drifted.
pub fn verify_run_evidence_publication(
    root: &Path,
    json_path: &Path,
    markdown_path: &Path,
) -> Result<()> {
    let derived = derive_run_evidence_index(root)?;
    let checked = super::load_run_evidence_index(json_path)?;
    if checked != derived {
        bail!("selected run-evidence JSON drifted from governed locks");
    }
    let markdown = fs::read_to_string(markdown_path)
        .with_context(|| format!("read run-evidence Markdown {}", markdown_path.display()))?;
    if markdown != render_run_evidence_markdown(&derived) {
        bail!("selected run-evidence Markdown drifted from governed locks");
    }
    Ok(())
}

fn result_directories(result_root: &Path) -> Result<Vec<(String, String, PathBuf)>> {
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for tool_root in child_directories(result_root)? {
        if tool_root.file_name().and_then(|name| name.to_str()) == Some("python-cache") {
            continue;
        }
        let tool = tool_root
            .file_name()
            .and_then(|name| name.to_str())
            .context("result tool directory is not UTF-8")?;
        for candidate in child_directories(&tool_root)? {
            let Some(run_id) = candidate.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if run_number(run_id).is_err() {
                continue;
            }
            let resolved = candidate.canonicalize()?;
            if seen.insert(resolved.clone()) {
                rows.push((run_id.to_string(), tool.to_string(), resolved));
            }
        }
    }
    for candidate in child_directories(result_root)? {
        let Ok((run_id, tool)) = run_directory_identity(&candidate) else {
            continue;
        };
        let resolved = candidate.canonicalize()?;
        if seen.insert(resolved.clone()) {
            rows.push((run_id, tool, resolved));
        }
    }
    rows.sort_by_key(|(run_id, _, _)| run_number(run_id).unwrap_or(u64::MAX));
    Ok(rows)
}

fn evidence_record(
    run_id: &str,
    tool: &str,
    result: &Path,
    result_root: &Path,
) -> Result<RunEvidenceLock> {
    let pipeline_path = result.join("config-pipeline.yml");
    let credential_path = result.join("config-credential.yml");
    if !pipeline_path.is_file() || !credential_path.is_file() {
        bail!("run lacks archived pipeline contracts: {}", result.display());
    }
    crate::execution_contract::load_container_credential_reference(&credential_path)?;
    let identity = pipeline_identity(&pipeline_path)?;
    if identity.run_id != run_id || identity.tool != tool {
        bail!("pipeline identity disagrees with result path: {}", pipeline_path.display());
    }
    let evidence = optional_json_object(&result.join("evidence.json"))?;
    if evidence
        .get("run_id")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|evidence_run_id| evidence_run_id != run_id)
    {
        bail!("evidence run identity disagrees with its directory: {}", result.display());
    }
    let accounting = optional_json_object(&result.join("slurm-accounting.json"))?;
    let execution_status =
        evidence.get("status").and_then(serde_json::Value::as_str).unwrap_or("incomplete");
    let repository_commit = evidence
        .get("repository_commit")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&identity.repository_commit)
        .to_string();
    let slurm_job_id = evidence
        .get("runtime")
        .and_then(serde_json::Value::as_object)
        .and_then(|runtime| runtime.get("slurm_job_id"))
        .or_else(|| accounting.get("slurm_job_id"))
        .and_then(value_as_string)
        .unwrap_or_default();
    let failures = evidence
        .get("failures")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter(|values| !values.is_empty())
        .unwrap_or_else(|| {
            if evidence.is_empty() {
                vec!["execution evidence is absent".to_string()]
            } else {
                Vec::new()
            }
        });
    Ok(RunEvidenceLock {
        schema_version: CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1.to_string(),
        run_id: run_id.to_string(),
        tool: tool.to_string(),
        execution_status: parse_execution_status(execution_status)?,
        scientific_status: ScientificAcceptanceStatus::NotAssessed,
        source_result: format!(
            "results/container-validation/{}",
            result.strip_prefix(result_root)?.to_string_lossy()
        ),
        repository_commit,
        slurm_job_id,
        image: evidence.get("image").cloned().unwrap_or(serde_json::Value::Null),
        failures,
        source_files: file_inventory(result)?,
        tracked_files: BTreeMap::new(),
    })
}

fn file_inventory(root: &Path) -> Result<Vec<RunEvidenceFileIdentity>> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<RunEvidenceFileIdentity>,
) -> Result<()> {
    let mut entries = fs::read_dir(current)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else if path.is_file() {
            files.push(RunEvidenceFileIdentity {
                path: Some(path.strip_prefix(root)?.to_string_lossy().replace('\\', "/")),
                size_bytes: path.metadata()?.len(),
                sha256: bijux_dna_infra::hash_file_sha256(&path)?,
            });
        }
    }
    Ok(())
}

fn select_final_evidence(
    records: &[RunEvidenceLock],
) -> Result<(Vec<RunEvidenceLock>, Vec<RunEvidenceLock>)> {
    let mut by_tool = BTreeMap::<&str, Vec<&RunEvidenceLock>>::new();
    for record in records {
        by_tool.entry(&record.tool).or_default().push(record);
    }
    let mut selected = Vec::new();
    let mut superseded = Vec::new();
    for (tool, tool_records) in by_tool {
        let final_record = tool_records
            .iter()
            .filter(|record| record.execution_status == AttemptExecutionStatus::Passed)
            .max_by_key(|record| run_number(&record.run_id).unwrap_or_default())
            .with_context(|| {
                format!("{tool}: no successful execution can be published as final evidence")
            })?;
        let final_record = (*final_record).clone();
        selected.push(final_record.clone());
        superseded.extend(
            tool_records.into_iter().filter(|record| record.run_id != final_record.run_id).cloned(),
        );
    }
    selected.sort_by(|left, right| left.tool.cmp(&right.tool));
    superseded.sort_by_key(|record| run_number(&record.run_id).unwrap_or(u64::MAX));
    Ok((selected, superseded))
}

fn validate_final_evidence(records: &[RunEvidenceLock]) -> Result<()> {
    let (selected, superseded) = select_final_evidence(records)?;
    if !superseded.is_empty() {
        bail!(
            "tracked run surface contains superseded attempts; publish final evidence first: {}",
            superseded.iter().map(|record| record.run_id.as_str()).collect::<Vec<_>>().join(", ")
        );
    }
    if selected.iter().any(|record| record.execution_status != AttemptExecutionStatus::Passed) {
        bail!("final evidence contains a non-passing execution");
    }
    Ok(())
}

#[derive(Debug)]
struct PipelineIdentity {
    pipeline_schema_version: String,
    run_id: String,
    campaign: String,
    corpus: String,
    sample: String,
    domain: String,
    stage: String,
    tool: String,
    repository_commit: String,
}

fn pipeline_identity(path: &Path) -> Result<PipelineIdentity> {
    let contract = crate::execution_contract::load_container_pipeline(path)?;
    let identity = match contract {
        crate::execution_contract::ContainerPipelineContract::Legacy(pipeline) => {
            PipelineIdentity {
                pipeline_schema_version: pipeline.schema_version,
                run_id: pipeline.run.id,
                campaign: pipeline.run.campaign,
                corpus: pipeline.run.corpus,
                sample: pipeline.run.sample,
                domain: pipeline.run.domain,
                stage: pipeline.run.stage,
                tool: pipeline.run.tool,
                repository_commit: pipeline.repository.commit,
            }
        }
        crate::execution_contract::ContainerPipelineContract::Current(pipeline) => {
            PipelineIdentity {
                pipeline_schema_version: pipeline.schema_version,
                run_id: pipeline.run.id,
                campaign: pipeline.run.campaign,
                corpus: pipeline.run.corpus,
                sample: pipeline.run.sample,
                domain: pipeline.run.domain,
                stage: pipeline.run.stage,
                tool: pipeline.run.tool,
                repository_commit: pipeline.repository.commit,
            }
        }
    };
    Ok(identity)
}

fn optional_json_object(path: &Path) -> Result<serde_json::Map<String, serde_json::Value>> {
    if !path.is_file() {
        return Ok(serde_json::Map::new());
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read JSON evidence {}", path.display()))?;
    serde_json::from_str::<serde_json::Value>(&raw)?
        .as_object()
        .cloned()
        .with_context(|| format!("JSON evidence must be an object: {}", path.display()))
}

fn load_attempt_ledger(path: &Path) -> Result<RunAttemptLedger> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read existing attempt ledger {}", path.display()))?;
    let ledger: RunAttemptLedger = serde_json::from_str(&raw)
        .with_context(|| format!("parse existing attempt ledger {}", path.display()))?;
    if ledger.schema_version != CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1 {
        bail!("existing attempt ledger is invalid: {}", path.display());
    }
    Ok(ledger)
}

fn parse_execution_status(value: &str) -> Result<AttemptExecutionStatus> {
    match value {
        "passed" => Ok(AttemptExecutionStatus::Passed),
        "failed" => Ok(AttemptExecutionStatus::Failed),
        "blocked" => Ok(AttemptExecutionStatus::Blocked),
        "incomplete" => Ok(AttemptExecutionStatus::Incomplete),
        _ => bail!("unsupported run execution status: {value}"),
    }
}

fn value_as_string(value: &serde_json::Value) -> Option<String> {
    value.as_str().map(str::to_string).or_else(|| value.as_u64().map(|number| number.to_string()))
}

fn status_counts<'a>(values: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value.to_string()).or_insert(0) += 1;
    }
    counts
}

fn child_directories(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)
        .with_context(|| format!("read directory {}", root.display()))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.is_dir());
    paths.sort();
    Ok(paths)
}
