use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_runtime::run_layout::{
    ArtifactIdentityV1, ArtifactInventoryV1, ArtifactScientificContextV1, CacheDecisionV1,
    HashLedgerEntryV1, HashLedgerV1, ReplayManifestV1, RunExecutionModeV1, RunLayout,
    RunLifecycleStateV1,
};
use sha2::Digest;

use super::summary_artifact::relative_path_string;

#[derive(Debug, Clone)]
pub(super) struct GovernedEvidenceArtifacts {
    pub artifact_inventory_path: PathBuf,
    pub artifact_inventory_text_path: PathBuf,
    pub replay_manifest_path: PathBuf,
    pub hash_ledger_path: PathBuf,
    pub run_summary_text_path: PathBuf,
}

pub(super) fn materialize_governed_evidence(
    layout: &RunLayout,
    graph: &ExecutionGraph,
    run_id: &str,
    mode: RunExecutionModeV1,
    state: RunLifecycleStateV1,
    original_run_id: &str,
    reused_artifact_ids: Vec<String>,
    cache_decisions: Vec<CacheDecisionV1>,
    environment_differences: Vec<String>,
) -> Result<GovernedEvidenceArtifacts> {
    let inventory = build_artifact_inventory(layout, graph, run_id, original_run_id)?;
    bijux_dna_runtime::run_layout::write_artifact_inventory(layout, &inventory)?;
    write_artifact_inventory_text(layout, &inventory)?;

    let replay_manifest = build_replay_manifest(
        graph,
        run_id,
        original_run_id,
        mode,
        reused_artifact_ids,
        cache_decisions,
        environment_differences,
    );
    bijux_dna_runtime::run_layout::write_replay_manifest(layout, &replay_manifest)?;

    write_scientific_run_summary_text(layout, graph, &inventory, mode, state)?;

    let ledger = build_hash_ledger(layout, run_id)?;
    bijux_dna_runtime::run_layout::write_hash_ledger(layout, &ledger)?;

    Ok(GovernedEvidenceArtifacts {
        artifact_inventory_path: layout.artifact_inventory_path.clone(),
        artifact_inventory_text_path: layout.artifact_inventory_text_path.clone(),
        replay_manifest_path: layout.replay_manifest_path.clone(),
        hash_ledger_path: layout.hash_ledger_path.clone(),
        run_summary_text_path: layout.run_summary_text_path.clone(),
    })
}

fn build_artifact_inventory(
    layout: &RunLayout,
    graph: &ExecutionGraph,
    run_id: &str,
    original_run_id: &str,
) -> Result<ArtifactInventoryV1> {
    let manifest_raw = std::fs::read_to_string(&layout.manifest_path)
        .with_context(|| format!("read {}", layout.manifest_path.display()))?;
    let manifest: serde_json::Value =
        serde_json::from_str(&manifest_raw).context("parse run manifest for artifact inventory")?;
    let dataset_fingerprints = manifest
        .get("dataset_fingerprints")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let stage_outputs = stage_output_index(layout.run_dir.as_path(), graph);
    let artifacts = manifest
        .get("output_artifacts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .map(|entry| {
            artifact_identity_from_entry(layout, entry, &stage_outputs, &dataset_fingerprints)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ArtifactInventoryV1 {
        schema_version: "bijux.artifact_inventory.v1".to_string(),
        run_id: run_id.to_string(),
        replay_source_run_id: (original_run_id != run_id).then(|| original_run_id.to_string()),
        artifacts,
    })
}

fn artifact_identity_from_entry(
    layout: &RunLayout,
    entry: &serde_json::Value,
    stage_outputs: &BTreeMap<String, StageOutputContract>,
    dataset_fingerprints: &[String],
) -> Result<ArtifactIdentityV1> {
    let name =
        entry.get("name").and_then(serde_json::Value::as_str).unwrap_or("artifact").to_string();
    let path = entry
        .get("path")
        .and_then(serde_json::Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(name.clone()));
    let key = relative_path_string(&layout.run_dir, &layout.run_dir.join(&path));
    let stage_output = stage_outputs.get(&key);
    let role = stage_output
        .map(|output| output.role.clone())
        .or_else(|| entry.get("kind").and_then(serde_json::Value::as_str).map(str::to_string))
        .unwrap_or_else(|| "artifact".to_string());
    let domain = stage_output
        .and_then(|output| output.stage_id.split('.').next().map(str::to_string))
        .unwrap_or_else(|| "runtime".to_string());
    let meaning = if let Some(output) = stage_output {
        format!("declared output for {}", output.stage_id)
    } else {
        format!("governed runtime artifact {name}")
    };
    Ok(ArtifactIdentityV1 {
        artifact_id: name.clone(),
        name,
        role,
        path,
        sha256: entry.get("sha256").and_then(serde_json::Value::as_str).map(str::to_string),
        producing_stage_id: stage_output.map(|output| output.stage_id.clone()),
        producing_command: stage_output.map(|output| output.command.clone()).unwrap_or_default(),
        input_lineage: dataset_fingerprints.to_vec(),
        schema_version: entry.get("schema").and_then(serde_json::Value::as_str).map(str::to_string),
        replay_source_run_id: None,
        scientific_context: Some(ArtifactScientificContextV1 {
            domain,
            meaning,
            safe_to_use: !matches!(
                entry.get("kind").and_then(serde_json::Value::as_str),
                Some("run_failure")
            ),
            advisory_only: matches!(
                entry.get("kind").and_then(serde_json::Value::as_str),
                Some("runtime_policy" | "checkpoint")
            ),
        }),
    })
}

fn write_artifact_inventory_text(
    layout: &RunLayout,
    inventory: &ArtifactInventoryV1,
) -> Result<()> {
    let mut lines = vec!["artifact_id\trole\tpath\tstage_id\tsha256".to_string()];
    for artifact in &inventory.artifacts {
        lines.push(format!(
            "{}\t{}\t{}\t{}\t{}",
            artifact.artifact_id,
            artifact.role,
            artifact.path.display(),
            artifact.producing_stage_id.as_deref().unwrap_or("runtime"),
            artifact.sha256.as_deref().unwrap_or("unhashed"),
        ));
    }
    lines.push(String::new());
    bijux_dna_infra::atomic_write_bytes(
        &layout.artifact_inventory_text_path,
        lines.join("\n").as_bytes(),
    )?;
    Ok(())
}

fn build_replay_manifest(
    graph: &ExecutionGraph,
    run_id: &str,
    original_run_id: &str,
    mode: RunExecutionModeV1,
    reused_artifact_ids: Vec<String>,
    cache_decisions: Vec<CacheDecisionV1>,
    environment_differences: Vec<String>,
) -> ReplayManifestV1 {
    let selected_stage_ids =
        graph.steps().iter().map(|step| step.stage_id.to_string()).collect::<Vec<_>>();
    let expected_outputs = graph
        .steps()
        .iter()
        .flat_map(|step| step.io.outputs.iter().map(|artifact| artifact.name.to_string()))
        .collect::<Vec<_>>();
    let rerun_stage_ids = match mode {
        RunExecutionModeV1::DryRun => Vec::new(),
        RunExecutionModeV1::Simulation
        | RunExecutionModeV1::Advisory
        | RunExecutionModeV1::Enforced => selected_stage_ids.clone(),
    };
    ReplayManifestV1 {
        schema_version: "bijux.replay_manifest.v1".to_string(),
        replay_run_id: run_id.to_string(),
        original_run_id: original_run_id.to_string(),
        selected_stage_ids,
        reused_artifact_ids,
        rerun_stage_ids,
        expected_outputs,
        cache_decisions,
        environment_differences,
    }
}

fn write_scientific_run_summary_text(
    layout: &RunLayout,
    graph: &ExecutionGraph,
    inventory: &ArtifactInventoryV1,
    mode: RunExecutionModeV1,
    state: RunLifecycleStateV1,
) -> Result<()> {
    let safe_outputs = inventory
        .artifacts
        .iter()
        .filter(|artifact| {
            artifact.scientific_context.as_ref().is_some_and(|context| context.safe_to_use)
        })
        .map(|artifact| artifact.path.display().to_string())
        .collect::<Vec<_>>();
    let checked = graph
        .steps()
        .iter()
        .map(|step| format!("{} via {}", step.stage_id, step.image.image))
        .collect::<Vec<_>>();
    let what_changed = inventory
        .artifacts
        .iter()
        .map(|artifact| {
            format!(
                "{} -> {}",
                artifact.artifact_id,
                artifact.sha256.as_deref().unwrap_or("unhashed")
            )
        })
        .collect::<Vec<_>>();
    let failed = if matches!(state, RunLifecycleStateV1::Failed | RunLifecycleStateV1::Cancelled) {
        vec![format!("run state is {state}")]
    } else {
        Vec::new()
    };
    let advisory = if matches!(mode, RunExecutionModeV1::Simulation | RunExecutionModeV1::Advisory)
    {
        vec![format!("mode {mode} did not enforce process execution")]
    } else {
        Vec::new()
    };
    let enforced = if matches!(mode, RunExecutionModeV1::Enforced) {
        vec!["runner execution and runtime policy were enforced".to_string()]
    } else {
        Vec::new()
    };

    let body = [
        format!(
            "run_id: {}",
            layout.run_dir.file_name().and_then(|value| value.to_str()).unwrap_or("unknown-run")
        ),
        format!("mode: {mode}"),
        format!("state: {state}"),
        "what_was_checked:".to_string(),
        checked.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n"),
        "what_changed:".to_string(),
        what_changed.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n"),
        "what_failed:".to_string(),
        if failed.is_empty() {
            "- none".to_string()
        } else {
            failed.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n")
        },
        "advisory_findings:".to_string(),
        if advisory.is_empty() {
            "- none".to_string()
        } else {
            advisory.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n")
        },
        "enforced_findings:".to_string(),
        if enforced.is_empty() {
            "- none".to_string()
        } else {
            enforced.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n")
        },
        "safe_outputs:".to_string(),
        if safe_outputs.is_empty() {
            "- none".to_string()
        } else {
            safe_outputs.iter().map(|entry| format!("- {entry}")).collect::<Vec<_>>().join("\n")
        },
        String::new(),
    ]
    .join("\n");
    bijux_dna_infra::atomic_write_bytes(&layout.run_summary_text_path, body.as_bytes())?;
    Ok(())
}

fn build_hash_ledger(layout: &RunLayout, run_id: &str) -> Result<HashLedgerV1> {
    let report_path = layout.reports_dir.join("report.json");
    let mut files = vec![
        (layout.manifest_path.clone(), "run_manifest".to_string()),
        (layout.graph_path.clone(), "graph".to_string()),
        (layout.plan_manifest_path.clone(), "plan_manifest".to_string()),
        (layout.run_state_path.clone(), "run_state".to_string()),
        (layout.runtime_policy_path.clone(), "runtime_policy".to_string()),
        (layout.executor_descriptor_path.clone(), "executor_descriptor".to_string()),
        (layout.checkpoint_path.clone(), "checkpoint".to_string()),
        (layout.run_summary_path.clone(), "run_summary".to_string()),
        (layout.run_summary_text_path.clone(), "run_summary_text".to_string()),
        (layout.artifact_inventory_path.clone(), "artifact_inventory".to_string()),
        (layout.artifact_inventory_text_path.clone(), "artifact_inventory_text".to_string()),
        (report_path, "report".to_string()),
    ];
    if layout.failure_path.exists() {
        files.push((layout.failure_path.clone(), "run_failure".to_string()));
    }

    let mut existing = files
        .into_iter()
        .filter(|(path, _)| path.exists())
        .map(|(path, kind)| (relative_path_string(&layout.run_dir, &path), path, kind))
        .collect::<Vec<_>>();
    existing.sort_by(|left, right| left.0.cmp(&right.0));

    let mut entries = Vec::new();
    let mut previous_entry_sha256 = None;
    for (relative_path, path, kind) in existing {
        let sha256 = bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash ledger entry {}", path.display()))?;
        entries.push(HashLedgerEntryV1 {
            record_id: relative_path.replace('/', ":"),
            kind,
            path: PathBuf::from(relative_path),
            sha256: sha256.clone(),
            previous_entry_sha256: previous_entry_sha256.clone(),
        });
        previous_entry_sha256 = Some(sha256);
    }

    let root_sha256 = sha256_hex(sha2::Sha256::digest(to_canonical_json_bytes(&entries)?));
    Ok(HashLedgerV1 {
        schema_version: "bijux.hash_ledger.v1".to_string(),
        run_id: run_id.to_string(),
        root_sha256,
        entries,
    })
}

#[derive(Debug, Clone)]
struct StageOutputContract {
    stage_id: String,
    role: String,
    command: Vec<String>,
}

fn stage_output_index(
    run_dir: &Path,
    graph: &ExecutionGraph,
) -> BTreeMap<String, StageOutputContract> {
    let mut index = BTreeMap::new();
    for step in graph.steps() {
        let command = step.command.template.clone();
        for artifact in &step.io.outputs {
            let absolute_path = if artifact.path.is_absolute() {
                artifact.path.clone()
            } else {
                run_dir.join(&artifact.path)
            };
            let relative = relative_path_string(run_dir, &absolute_path);
            index.insert(
                relative,
                StageOutputContract {
                    stage_id: step.stage_id.to_string(),
                    role: format!("{:?}", artifact.role),
                    command: command.clone(),
                },
            );
        }
    }
    index
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
