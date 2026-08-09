//! Owner: bijux-dna-bench
//! Deterministic paper-facing index over permanent benchmark run protocols.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_benchmark_run_index, validate_benchmark_run_registry,
};
use bijux_dna_bench_model::{
    benchmark_run_index_schema, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexSummary, BenchmarkRunRegistry,
    BenchmarkRunRegistryState,
};

const RUN_REGISTRY: &str = "benchmarks/contracts/run-registry.json";
const RUN_INDEX_JSON: &str = "benchmarks/runs/RUN_INDEX.json";
const RUN_INDEX_MARKDOWN: &str = "benchmarks/runs/RUN_INDEX.md";
const RUN_INDEX_SCHEMA: &str = "benchmarks/contracts/schemas/run-index.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkRunIndexWriteMode {
    Write,
    Check,
}

/// Derive a complete protocol index without presenting unpublished runs as executed evidence.
///
/// # Errors
/// Returns an error when the permanent registry is invalid or the derived index fails closure.
pub fn derive_benchmark_run_index(
    registry: &BenchmarkRunRegistry,
    registry_sha256: &str,
    registry_size_bytes: u64,
) -> Result<BenchmarkRunIndex> {
    validate_benchmark_run_registry(registry).context("validate benchmark run registry")?;
    let mut runs = Vec::with_capacity(registry.runs.len());
    for row in &registry.runs {
        let mut corpus_roles = row.required_corpus_roles.clone();
        corpus_roles.sort();
        corpus_roles.dedup();
        let protocol_bytes = serde_json::to_vec(row).context("serialize benchmark protocol")?;
        runs.push(BenchmarkRunIndexEntry {
            run_id: row.run_id.clone(),
            citation_key: format!("bijux-benchmark-{}", row.run_id),
            protocol_title: row.title.clone(),
            slug: row.slug.clone(),
            stage_id: row.stage_id.clone(),
            owner: row.owner.clone(),
            protocol_state: row.state,
            protocol_sha256: bijux_dna_infra::hash_bytes_sha256(&protocol_bytes),
            registry_uri: format!("repo://benchmarks/contracts/run-registry.json#{}", row.run_id),
            tool_ids: row.tool_ids.clone(),
            corpus_roles,
            acceptance_policy_id: row.acceptance_policy_id.clone(),
            selected_publication: None,
            limitations: unpublished_limitations(row.state, row.state_reason.as_deref()),
        });
    }
    let citable_publication_count =
        runs.iter().filter(|run| run.selected_publication.is_some()).count();
    let index = BenchmarkRunIndex::v1(
        BenchmarkRunIndexFileReference {
            uri: "repo://benchmarks/contracts/run-registry.json".to_string(),
            sha256: registry_sha256.to_string(),
            size_bytes: registry_size_bytes,
        },
        BenchmarkRunIndexSummary {
            registered_run_count: runs.len(),
            active_protocol_count: registry.summary.active_count,
            reserved_protocol_count: registry.summary.reserved_count,
            retired_protocol_count: registry.summary.retired_count,
            citable_publication_count,
            unpublished_run_count: runs.len() - citable_publication_count,
        },
        runs,
    );
    validate_benchmark_run_index(&index).context("validate benchmark run index")?;
    Ok(index)
}

/// Write or check `RUN_INDEX` JSON, Markdown, and its generated schema.
///
/// # Errors
/// Returns an error when source bytes, selected-publication layout, or generated outputs drift.
pub fn publish_checked_in_benchmark_run_index(
    repository_root: &Path,
    mode: BenchmarkRunIndexWriteMode,
) -> Result<BenchmarkRunIndex> {
    ensure_no_unindexed_publication_directories(repository_root)?;
    let registry_path = repository_root.join(RUN_REGISTRY);
    let registry_bytes = fs::read(&registry_path).context("read benchmark run registry")?;
    let registry: BenchmarkRunRegistry =
        serde_json::from_slice(&registry_bytes).context("parse benchmark run registry")?;
    let registry_sha256 = bijux_dna_infra::hash_bytes_sha256(&registry_bytes);
    let registry_size_bytes =
        u64::try_from(registry_bytes.len()).context("run registry size exceeds u64")?;
    let index = derive_benchmark_run_index(&registry, &registry_sha256, registry_size_bytes)?;
    let json = serde_json::to_string_pretty(&index)? + "\n";
    let markdown = render_benchmark_run_index_markdown(&index);
    let schema = serde_json::to_string_pretty(&benchmark_run_index_schema())? + "\n";
    for (relative, content) in [
        (RUN_INDEX_JSON, json.as_bytes()),
        (RUN_INDEX_MARKDOWN, markdown.as_bytes()),
        (RUN_INDEX_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkRunIndexWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write benchmark run index {relative}"))?;
            }
            BenchmarkRunIndexWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read benchmark run index {relative}"))?;
                if current != content {
                    bail!("generated benchmark run index drifted: {relative}");
                }
            }
        }
    }
    Ok(index)
}

#[must_use]
pub fn render_benchmark_run_index_markdown(index: &BenchmarkRunIndex) -> String {
    let summary = &index.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::run_index -->".to_string(),
        "# Benchmark run index".to_string(),
        String::new(),
        "Run IDs identify permanent protocols. A citation key is stable, but a run is citable as executed evidence only when this index names its selected publication and replay receipt."
            .to_string(),
        String::new(),
        format!("- Registered protocols: **{}**", summary.registered_run_count),
        format!("- Active protocols: **{}**", summary.active_protocol_count),
        format!("- Reserved protocols: **{}**", summary.reserved_protocol_count),
        format!("- Citable publications: **{}**", summary.citable_publication_count),
        format!("- Unpublished protocols: **{}**", summary.unpublished_run_count),
        String::new(),
        "| Run | Citation key | Stage | Tools | Protocol | Publication |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for run in &index.runs {
        let state = match run.protocol_state {
            BenchmarkRunRegistryState::Active => "active",
            BenchmarkRunRegistryState::Reserved => "reserved",
            BenchmarkRunRegistryState::Retired => "retired",
        };
        let publication = run
            .selected_publication
            .as_ref()
            .map_or("not selected", |selected| selected.publication_id.as_str());
        lines.push(format!(
            "| `{}` | `{}` | `{}` | {} | `{state}` | `{publication}` |",
            run.run_id,
            run.citation_key,
            run.stage_id,
            run.tool_ids.iter().map(|tool| format!("`{tool}`")).collect::<Vec<_>>().join(", "),
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn unpublished_limitations(
    state: BenchmarkRunRegistryState,
    state_reason: Option<&str>,
) -> Vec<String> {
    let mut limitations = Vec::new();
    if let Some(reason) = state_reason {
        limitations.push(reason.to_string());
    }
    limitations.push(match state {
        BenchmarkRunRegistryState::Active => {
            "No current admitted publication is selected; this protocol is not citable as executed evidence."
        }
        BenchmarkRunRegistryState::Reserved => {
            "The protocol is reserved and has no selected publication."
        }
        BenchmarkRunRegistryState::Retired => {
            "The protocol is retired and has no current selected publication."
        }
    }
    .to_string());
    limitations
}

fn ensure_no_unindexed_publication_directories(repository_root: &Path) -> Result<()> {
    let runs_root = repository_root.join("benchmarks/runs");
    for entry in fs::read_dir(&runs_root).context("read benchmark runs root")? {
        let path = entry?.path();
        if path.is_dir() {
            bail!(
                "benchmark selected-publication directory is not yet represented in RUN_INDEX: {}",
                path.display()
            );
        }
    }
    Ok(())
}
