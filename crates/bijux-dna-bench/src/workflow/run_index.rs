//! Owner: bijux-dna-bench
//! Deterministic paper-facing index over permanent benchmark run protocols.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    benchmark_run_definition_sha256, benchmark_run_directory_name,
    validate_benchmark_protocol_citation, validate_benchmark_run_case_index,
    validate_benchmark_run_definition, validate_benchmark_run_index,
    validate_benchmark_run_publication_pointer, validate_benchmark_run_registry,
    BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1, BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1,
};
use bijux_dna_bench_model::{
    benchmark_protocol_citation_schema, benchmark_run_case_index_schema,
    benchmark_run_definition_schema, benchmark_run_index_schema,
    benchmark_run_publication_pointer_schema, BenchmarkContractLock, BenchmarkCorpusRole,
    BenchmarkPortfolio, BenchmarkProtocolCitation, BenchmarkProtocolCitationStatus,
    BenchmarkRunCaseIndex, BenchmarkRunDefinition, BenchmarkRunIdentity, BenchmarkRunIndex,
    BenchmarkRunIndexEntry, BenchmarkRunIndexFileReference, BenchmarkRunIndexPublication,
    BenchmarkRunIndexSummary, BenchmarkRunPublicationPointer, BenchmarkRunRegistry,
    BenchmarkRunRegistryRow, BenchmarkRunRegistryState, BenchmarkRunState,
};

use super::load_benchmark_portfolio;

const RUN_REGISTRY: &str = "benchmarks/contracts/run-registry.json";
const RUN_INDEX_JSON: &str = "benchmarks/runs/RUN_INDEX.json";
const RUN_INDEX_MARKDOWN: &str = "benchmarks/runs/RUN_INDEX.md";
const RUN_INDEX_SCHEMA: &str = "benchmarks/contracts/schemas/run-index.schema.json";
const RUN_DEFINITION_SCHEMA: &str = "benchmarks/contracts/schemas/run-definition.schema.json";
const PROTOCOL_CITATION_SCHEMA: &str = "benchmarks/contracts/schemas/protocol-citation.schema.json";
const RUN_CASE_INDEX_SCHEMA: &str = "benchmarks/contracts/schemas/run-case-index.schema.json";
const RUN_PUBLICATION_POINTER_SCHEMA: &str =
    "benchmarks/contracts/schemas/run-publication-pointer.schema.json";
const PORTFOLIO: &str = "benchmarks/contracts/portfolio.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkRunIndexWriteMode {
    Write,
    Check,
}

struct BenchmarkRunPublicationSurface {
    directory_name: String,
    definition: BenchmarkRunDefinition,
    definition_markdown: String,
    citation: BenchmarkProtocolCitation,
    case_index: BenchmarkRunCaseIndex,
    publication: BenchmarkRunPublicationPointer,
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
    let protocol_hashes = registry
        .runs
        .iter()
        .map(|row| {
            let protocol_bytes = serde_json::to_vec(row).context("serialize benchmark protocol")?;
            Ok((row.run_id.clone(), bijux_dna_infra::hash_bytes_sha256(&protocol_bytes)))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let publications =
        registry.runs.iter().map(|row| (row.run_id.clone(), None)).collect::<BTreeMap<_, _>>();
    derive_benchmark_run_index_with_publications(
        registry,
        registry_sha256,
        registry_size_bytes,
        &protocol_hashes,
        &publications,
    )
}

fn derive_benchmark_run_index_with_publications(
    registry: &BenchmarkRunRegistry,
    registry_sha256: &str,
    registry_size_bytes: u64,
    protocol_hashes: &BTreeMap<String, String>,
    publications: &BTreeMap<String, Option<BenchmarkRunIndexPublication>>,
) -> Result<BenchmarkRunIndex> {
    validate_benchmark_run_registry(registry).context("validate benchmark run registry")?;
    let mut runs = Vec::with_capacity(registry.runs.len());
    for row in &registry.runs {
        let mut corpus_roles = row.required_corpus_roles.clone();
        corpus_roles.sort();
        corpus_roles.dedup();
        let selected_publication = publications
            .get(&row.run_id)
            .with_context(|| format!("publication selection is absent for {}", row.run_id))?
            .clone();
        runs.push(BenchmarkRunIndexEntry {
            run_id: row.run_id.clone(),
            citation_key: format!("bijux-benchmark-{}", row.run_id),
            protocol_title: row.title.clone(),
            slug: row.slug.clone(),
            stage_id: row.stage_id.clone(),
            owner: row.owner.clone(),
            protocol_state: row.state,
            protocol_sha256: protocol_hashes
                .get(&row.run_id)
                .with_context(|| format!("protocol identity is absent for {}", row.run_id))?
                .clone(),
            registry_uri: format!("repo://benchmarks/contracts/run-registry.json#{}", row.run_id),
            tool_ids: row.tool_ids.clone(),
            corpus_roles,
            acceptance_policy_id: row.acceptance_policy_id.clone(),
            limitations: selected_publication.as_ref().map_or_else(
                || unpublished_limitations(row.state, row.state_reason.as_deref()),
                |_| Vec::new(),
            ),
            selected_publication,
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
    let registry_path = repository_root.join(RUN_REGISTRY);
    let registry_bytes = fs::read(&registry_path).context("read benchmark run registry")?;
    let registry: BenchmarkRunRegistry =
        serde_json::from_slice(&registry_bytes).context("parse benchmark run registry")?;
    let registry_sha256 = bijux_dna_infra::hash_bytes_sha256(&registry_bytes);
    let registry_size_bytes =
        u64::try_from(registry_bytes.len()).context("run registry size exceeds u64")?;
    let portfolio = load_benchmark_portfolio(&repository_root.join(PORTFOLIO))?;
    let surfaces = derive_run_publication_surfaces(repository_root, &registry, &portfolio)?;
    ensure_no_unindexed_publication_directories(repository_root, &surfaces)?;
    let protocol_hashes = surfaces
        .iter()
        .map(|surface| {
            Ok((
                surface.definition.identity.run_id.clone(),
                benchmark_run_definition_sha256(&surface.definition)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let publications = surfaces
        .iter()
        .map(|surface| {
            (
                surface.definition.identity.run_id.clone(),
                surface.publication.selected_publication.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let index = derive_benchmark_run_index_with_publications(
        &registry,
        &registry_sha256,
        registry_size_bytes,
        &protocol_hashes,
        &publications,
    )?;
    let json = serde_json::to_string_pretty(&index)? + "\n";
    let markdown = render_benchmark_run_index_markdown(&index);
    let schema = serde_json::to_string_pretty(&benchmark_run_index_schema())? + "\n";
    let definition_schema =
        serde_json::to_string_pretty(&benchmark_run_definition_schema())? + "\n";
    let citation_schema =
        serde_json::to_string_pretty(&benchmark_protocol_citation_schema())? + "\n";
    let case_index_schema =
        serde_json::to_string_pretty(&benchmark_run_case_index_schema())? + "\n";
    let publication_pointer_schema =
        serde_json::to_string_pretty(&benchmark_run_publication_pointer_schema())? + "\n";
    for (relative, content) in [
        (RUN_INDEX_JSON, json.as_bytes()),
        (RUN_INDEX_MARKDOWN, markdown.as_bytes()),
        (RUN_INDEX_SCHEMA, schema.as_bytes()),
        (RUN_DEFINITION_SCHEMA, definition_schema.as_bytes()),
        (PROTOCOL_CITATION_SCHEMA, citation_schema.as_bytes()),
        (RUN_CASE_INDEX_SCHEMA, case_index_schema.as_bytes()),
        (RUN_PUBLICATION_POINTER_SCHEMA, publication_pointer_schema.as_bytes()),
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
    for surface in surfaces {
        publish_run_surface(repository_root, mode, &surface)?;
    }
    Ok(index)
}

fn derive_run_publication_surfaces(
    repository_root: &Path,
    registry: &BenchmarkRunRegistry,
    portfolio: &BenchmarkPortfolio,
) -> Result<Vec<BenchmarkRunPublicationSurface>> {
    registry
        .runs
        .iter()
        .map(|row| derive_run_publication_surface(repository_root, row, portfolio))
        .collect()
}

fn derive_run_publication_surface(
    repository_root: &Path,
    row: &BenchmarkRunRegistryRow,
    portfolio: &BenchmarkPortfolio,
) -> Result<BenchmarkRunPublicationSurface> {
    let definition = derive_run_definition(repository_root, row, portfolio)?;
    validate_benchmark_run_definition(&definition)
        .with_context(|| format!("validate run definition {}", row.run_id))?;
    let protocol_sha256 = benchmark_run_definition_sha256(&definition)?;
    let directory_name = benchmark_run_directory_name(&definition.identity)?;
    let directory = repository_root.join("benchmarks/runs").join(&directory_name);
    let selected_publication = load_current_publication_selection(
        &directory.join("current-publication.json"),
        &definition.identity,
        &protocol_sha256,
    )?;
    let limitations = selected_publication.as_ref().map_or_else(
        || unpublished_limitations(row.state, row.state_reason.as_deref()),
        |publication| publication.limitations.clone(),
    );
    let case_index = load_case_index(
        &directory.join("case-index.json"),
        &definition.identity,
        &protocol_sha256,
    )?;
    let citation = BenchmarkProtocolCitation {
        schema_version: BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1.to_string(),
        citation_key: definition.citation_key.clone(),
        run: definition.identity.clone(),
        protocol_title: definition.title.clone(),
        protocol_version: definition.protocol_version.clone(),
        protocol_sha256: protocol_sha256.clone(),
        status: if selected_publication.is_some() {
            BenchmarkProtocolCitationStatus::Published
        } else {
            BenchmarkProtocolCitationStatus::ProtocolOnly
        },
        selected_publication: selected_publication.clone(),
        limitations: limitations.clone(),
    };
    let publication = BenchmarkRunPublicationPointer {
        schema_version: BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1.to_string(),
        run: definition.identity.clone(),
        protocol_sha256,
        selected_publication,
        limitations,
    };
    validate_benchmark_protocol_citation(&citation)?;
    validate_benchmark_run_case_index(&case_index)?;
    validate_benchmark_run_publication_pointer(&publication)?;
    let definition_markdown = render_benchmark_run_definition_markdown(&definition);
    Ok(BenchmarkRunPublicationSurface {
        directory_name,
        definition,
        definition_markdown,
        citation,
        case_index,
        publication,
    })
}

fn derive_run_definition(
    repository_root: &Path,
    row: &BenchmarkRunRegistryRow,
    portfolio: &BenchmarkPortfolio,
) -> Result<BenchmarkRunDefinition> {
    let state = match row.state {
        BenchmarkRunRegistryState::Active => BenchmarkRunState::Active,
        BenchmarkRunRegistryState::Reserved => BenchmarkRunState::Reserved,
        BenchmarkRunRegistryState::Retired => BenchmarkRunState::Retired,
    };
    let executable = row.state != BenchmarkRunRegistryState::Reserved;
    let contracts = if executable {
        let path = row.normalized_result_contract.as_deref().with_context(|| {
            format!("active run {} lacks a normalized result contract", row.run_id)
        })?;
        let bytes = fs::read(repository_root.join(path))
            .with_context(|| format!("read normalized result contract {path}"))?;
        vec![BenchmarkContractLock {
            contract_id: "normalized-result-contract".to_string(),
            repository_path: path.to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
        }]
    } else {
        Vec::new()
    };
    let mut corpus_roles = Vec::with_capacity(row.required_corpus_roles.len());
    for role in &row.required_corpus_roles {
        let mut corpus_ids = portfolio
            .corpora
            .iter()
            .filter(|corpus| corpus.roles.contains(role))
            .map(|corpus| corpus.id.clone())
            .collect::<Vec<_>>();
        corpus_ids.sort();
        corpus_ids.dedup();
        if corpus_ids.is_empty() {
            bail!("run {} corpus role {role} has no portfolio authority", row.run_id);
        }
        corpus_roles.push(BenchmarkCorpusRole { role: role.clone(), corpus_ids });
    }
    corpus_roles.sort_by(|left, right| left.role.cmp(&right.role));
    let mut tool_ids = row.tool_ids.clone();
    tool_ids.sort();
    tool_ids.dedup();
    let mut accepted_input_contracts =
        if executable { row.accepted_input_contracts.clone() } else { Vec::new() };
    accepted_input_contracts.sort();
    accepted_input_contracts.dedup();
    Ok(BenchmarkRunDefinition {
        schema_version: BENCHMARK_RUN_DEFINITION_SCHEMA_V1.to_string(),
        identity: BenchmarkRunIdentity {
            run_id: row.run_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
        },
        protocol_version: "1.0.0".to_string(),
        title: row.title.clone(),
        purpose: row.purpose.clone(),
        owner: row.owner.clone(),
        profile_id: row.profile_id.clone(),
        tool_exclusion_policy: row.tool_exclusion_policy.clone(),
        state,
        state_reason: row.state_reason.clone(),
        tool_ids,
        accepted_input_contracts,
        output_contract: executable.then(|| row.output_contract.clone()).flatten(),
        normalized_result_contract: executable
            .then(|| row.normalized_result_contract.clone())
            .flatten(),
        measurement_policy_id: row.measurement_policy_id.clone(),
        contracts,
        corpus_roles,
        acceptance_policy_id: row.acceptance_policy_id.clone(),
        citation_key: format!("bijux-benchmark-{}", row.run_id),
        supersedes_run_ids: row.supersedes_run_ids.clone(),
    })
}

fn load_current_publication_selection(
    path: &Path,
    identity: &BenchmarkRunIdentity,
    protocol_sha256: &str,
) -> Result<Option<BenchmarkRunIndexPublication>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("read current publication pointer {}", path.display()));
        }
    };
    let pointer: BenchmarkRunPublicationPointer = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse current publication pointer {}", path.display()))?;
    if pointer.run != *identity {
        bail!("current publication pointer targets another run: {}", path.display());
    }
    if pointer.protocol_sha256 != protocol_sha256 {
        if pointer.selected_publication.is_some() {
            bail!("selected publication is stale for current protocol: {}", path.display());
        }
        return Ok(None);
    }
    Ok(pointer.selected_publication)
}

fn load_case_index(
    path: &Path,
    identity: &BenchmarkRunIdentity,
    protocol_sha256: &str,
) -> Result<BenchmarkRunCaseIndex> {
    let existing = match fs::read(path) {
        Ok(bytes) => Some(bytes),
        Err(error) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error).with_context(|| format!("read run case index {}", path.display()));
        }
    };
    if let Some(bytes) = existing {
        let index: BenchmarkRunCaseIndex = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse run case index {}", path.display()))?;
        if index.run != *identity {
            bail!("case index targets another run: {}", path.display());
        }
        if index.protocol_sha256 == protocol_sha256 {
            return Ok(index);
        }
        if !index.cases.is_empty() {
            bail!("case index is stale for current protocol: {}", path.display());
        }
    }
    Ok(BenchmarkRunCaseIndex {
        schema_version: BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1.to_string(),
        run: identity.clone(),
        protocol_sha256: protocol_sha256.to_string(),
        cases: Vec::new(),
        limitations: vec![
            "No immutable benchmark cases are indexed; this protocol has no executable publication evidence."
                .to_string(),
        ],
    })
}

fn publish_run_surface(
    repository_root: &Path,
    mode: BenchmarkRunIndexWriteMode,
    surface: &BenchmarkRunPublicationSurface,
) -> Result<()> {
    let directory = repository_root.join("benchmarks/runs").join(&surface.directory_name);
    if mode == BenchmarkRunIndexWriteMode::Write {
        fs::create_dir_all(&directory)
            .with_context(|| format!("create benchmark run directory {}", directory.display()))?;
    }
    let definition = serde_json::to_string_pretty(&surface.definition)? + "\n";
    let citation = serde_json::to_string_pretty(&surface.citation)? + "\n";
    let case_index = serde_json::to_string_pretty(&surface.case_index)? + "\n";
    let publication = serde_json::to_string_pretty(&surface.publication)? + "\n";
    for (name, content) in [
        ("definition.json", definition.as_bytes()),
        ("definition.md", surface.definition_markdown.as_bytes()),
        ("citation.json", citation.as_bytes()),
        ("case-index.json", case_index.as_bytes()),
        ("current-publication.json", publication.as_bytes()),
    ] {
        let path = directory.join(name);
        match mode {
            BenchmarkRunIndexWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write benchmark run surface {}", path.display()))?
            }
            BenchmarkRunIndexWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read benchmark run surface {}", path.display()))?;
                if current != content {
                    bail!("generated benchmark run surface drifted: {}", path.display());
                }
            }
        }
    }
    Ok(())
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

#[must_use]
pub fn render_benchmark_run_definition_markdown(definition: &BenchmarkRunDefinition) -> String {
    let state = match definition.state {
        BenchmarkRunState::Active => "active",
        BenchmarkRunState::Reserved => "reserved",
        BenchmarkRunState::Retired => "retired",
    };
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::run_index -->".to_string(),
        format!("# {}", definition.title),
        String::new(),
        format!("- Run: `{}`", definition.identity.run_id),
        format!("- Stage: `{}`", definition.identity.stage_id),
        format!("- Protocol version: `{}`", definition.protocol_version),
        format!("- State: `{state}`"),
        format!("- Citation key: `{}`", definition.citation_key),
        String::new(),
        definition.purpose.clone(),
        String::new(),
        "## Planned tool closure".to_string(),
        String::new(),
    ];
    if definition.tool_ids.is_empty() {
        lines.push("No tools are selected by this protocol.".to_string());
    } else {
        lines.extend(definition.tool_ids.iter().map(|tool| format!("- `{tool}`")));
    }
    lines.extend([
        String::new(),
        "## Evidence status".to_string(),
        String::new(),
        "This file defines a permanent protocol; it is not execution evidence. See `current-publication.json` for the admitted publication selection and `case-index.json` for immutable case pointers."
            .to_string(),
        String::new(),
    ]);
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

fn ensure_no_unindexed_publication_directories(
    repository_root: &Path,
    surfaces: &[BenchmarkRunPublicationSurface],
) -> Result<()> {
    let runs_root = repository_root.join("benchmarks/runs");
    let expected =
        surfaces.iter().map(|surface| surface.directory_name.as_str()).collect::<BTreeSet<_>>();
    let allowed_files = BTreeSet::from([
        "case-index.json",
        "citation.json",
        "current-publication.json",
        "definition.json",
        "definition.md",
    ]);
    for entry in fs::read_dir(&runs_root).context("read benchmark runs root")? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .context("benchmark run directory name is not UTF-8")?;
        if !expected.contains(name) {
            bail!(
                "benchmark run directory is not represented by the permanent registry: {}",
                path.display()
            );
        }
        for child in fs::read_dir(&path)
            .with_context(|| format!("read benchmark run directory {}", path.display()))?
        {
            let child = child?;
            let child_name = child.file_name();
            let child_name =
                child_name.to_str().context("benchmark run publication file name is not UTF-8")?;
            if !child.file_type()?.is_file() || !allowed_files.contains(child_name) {
                bail!(
                    "benchmark run directory contains a non-publication artifact: {}",
                    child.path().display()
                );
            }
        }
    }
    Ok(())
}
