use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use toml::Value as TomlValue;

use crate::domain::{
    BindingResolutionRow, BindingSpec, ClaimEvidenceRow, ClaimSpec, CompiledScience,
    DecisionReasoningRow, FastqEnvironmentRow, LoadedSpecs, ScienceIndex, SourceAccess,
    SourceArchiveGapRow, SourceId, SourceInventoryRow, SourceKind, SourceSpec,
};
use crate::errors::validation_error;
use crate::io::{list_yaml_files, read_utf8};
use crate::schema::{
    ASSUMPTION_SCHEMA_VERSION, BINDING_SCHEMA_VERSION, CLAIM_SCHEMA_VERSION,
    DECISION_SCHEMA_VERSION, EVIDENCE_SCHEMA_VERSION, REASONING_SCHEMA_VERSION,
    RELEASE_SCHEMA_VERSION, SOURCE_SCHEMA_VERSION,
};

#[derive(Deserialize)]
struct ExecutionSupportDoc {
    stages: Vec<ExecutionSupportStage>,
}

#[derive(Deserialize)]
struct ExecutionSupportStage {
    stage_id: String,
    execution_status: String,
    runtime_support: String,
    normalization_support: String,
    benchmark_support: String,
    default_tool: String,
    admitted_tools: Vec<String>,
}

#[derive(Deserialize)]
struct StageContractDoc {
    stage_id: Option<String>,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct ToolRegistryEntry {
    status: String,
    version: String,
    default_version: String,
    version_rule: String,
    upstream: String,
    citation: String,
    license: String,
    pinned_commit: String,
    pin_strategy: String,
    runtimes: Vec<String>,
    container_ref: String,
    dockerfile: String,
    apptainer_def: String,
}

pub fn load_specs(root: &Path) -> Result<LoadedSpecs> {
    let mut loaded = LoadedSpecs::default();
    let mut errors = Vec::new();

    load_dir::<crate::domain::SourceSpec, _, _>(
        root,
        "science/specs/evidence/sources",
        &mut loaded.sources,
        SOURCE_SCHEMA_VERSION,
        |row| row.source_id.as_str().to_string(),
        |row| validate_source(row),
        &mut errors,
    )?;
    load_dir::<crate::domain::EvidenceSpec, _, _>(
        root,
        "science/specs/evidence/evidences",
        &mut loaded.evidences,
        EVIDENCE_SCHEMA_VERSION,
        |row| row.evidence_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;
    load_dir::<crate::domain::ClaimSpec, _, _>(
        root,
        "science/specs/evidence/claims",
        &mut loaded.claims,
        CLAIM_SCHEMA_VERSION,
        |row| row.claim_id.as_str().to_string(),
        |row| validate_claim(row),
        &mut errors,
    )?;
    load_dir::<crate::domain::AssumptionSpec, _, _>(
        root,
        "science/specs/evidence/assumptions",
        &mut loaded.assumptions,
        ASSUMPTION_SCHEMA_VERSION,
        |row| row.assumption_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;
    load_dir::<crate::domain::ReasoningSpec, _, _>(
        root,
        "science/specs/evidence/reasoning",
        &mut loaded.reasonings,
        REASONING_SCHEMA_VERSION,
        |row| row.reasoning_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;
    load_dir::<crate::domain::DecisionSpec, _, _>(
        root,
        "science/specs/evidence/decisions",
        &mut loaded.decisions,
        DECISION_SCHEMA_VERSION,
        |row| row.decision_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;
    load_dir::<crate::domain::BindingSpec, _, _>(
        root,
        "science/specs/evidence/bindings",
        &mut loaded.bindings,
        BINDING_SCHEMA_VERSION,
        |row| row.binding_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;
    load_dir::<crate::domain::ReleaseManifestSpec, _, _>(
        root,
        "science/specs/releases/manifests",
        &mut loaded.releases,
        RELEASE_SCHEMA_VERSION,
        |row| row.release_id.as_str().to_string(),
        |_| Ok(()),
        &mut errors,
    )?;

    errors.extend(validate_cross_references(&loaded));
    if !errors.is_empty() {
        return Err(validation_error(errors));
    }
    Ok(loaded)
}

pub fn compile_workspace(root: &Path) -> Result<CompiledScience> {
    let loaded = load_specs(root)?;
    compile_loaded(root, loaded)
}

pub fn compile_loaded(root: &Path, loaded: LoadedSpecs) -> Result<CompiledScience> {
    let source_inventory = build_source_inventory(root, &loaded);
    let source_archive_gaps = build_source_archive_gaps(&source_inventory);
    let claim_evidence_map = build_claim_evidence_map(&loaded);
    let decision_reasoning_map = build_decision_reasoning_map(&loaded);
    let binding_resolution = build_binding_resolution(&loaded);
    let fastq_environment_rows = build_fastq_environment_rows(root, &loaded)?;
    let fastq_container_reference_rows =
        build_fastq_container_reference_rows(root, &loaded, &fastq_environment_rows)?;
    let fastq_download_backlog_rows =
        build_fastq_download_backlog_rows(root, &fastq_container_reference_rows);
    let unresolved_refs = validate_cross_references(&loaded);
    if !unresolved_refs.is_empty() {
        return Err(validation_error(unresolved_refs));
    }
    let index = ScienceIndex {
        sources: loaded.sources.len(),
        source_inventory_rows: source_inventory.len(),
        source_archive_gap_rows: source_archive_gaps.len(),
        evidences: loaded.evidences.len(),
        claims: loaded.claims.len(),
        assumptions: loaded.assumptions.len(),
        reasonings: loaded.reasonings.len(),
        decisions: loaded.decisions.len(),
        bindings: loaded.bindings.len(),
        releases: loaded.releases.len(),
        fastq_container_reference_rows: fastq_container_reference_rows.len(),
        fastq_download_backlog_rows: fastq_download_backlog_rows.len(),
        fastq_environment_rows: fastq_environment_rows.len(),
    };
    Ok(CompiledScience {
        source_inventory,
        source_archive_gaps,
        claim_evidence_map,
        decision_reasoning_map,
        binding_resolution,
        unresolved_refs: Vec::new(),
        fastq_container_reference_rows,
        fastq_download_backlog_rows,
        fastq_environment_rows,
        index,
    })
}

fn build_source_inventory(root: &Path, loaded: &LoadedSpecs) -> Vec<SourceInventoryRow> {
    let mut rows = loaded
        .sources
        .values()
        .map(|source| {
            let archive_path = source.archive_path.clone().unwrap_or_default();
            let archive_status = match source.access {
                SourceAccess::RepoPath => "not_applicable".to_string(),
                SourceAccess::ManualDownload | SourceAccess::ManualClone => {
                    if root.join(&archive_path).exists() {
                        "present".to_string()
                    } else {
                        "missing".to_string()
                    }
                }
            };
            SourceInventoryRow {
                source_id: source.source_id.to_string(),
                kind: source_kind_label(&source.kind).to_string(),
                access: source_access_label(&source.access).to_string(),
                authority: source.authority.clone(),
                locator: source.locator.clone(),
                archive_path,
                archive_status,
                citation: source.citation.clone().unwrap_or_default(),
                tool_ids: source.tool_ids.join(","),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    rows
}

fn source_kind_label(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::RepoFile => "repo_file",
        SourceKind::RepoDirectory => "repo_directory",
        SourceKind::Document => "document",
        SourceKind::ExternalDocument => "external_document",
        SourceKind::ExternalRepository => "external_repository",
        SourceKind::Paper => "paper",
    }
}

fn source_access_label(access: &SourceAccess) -> &'static str {
    match access {
        SourceAccess::RepoPath => "repo_path",
        SourceAccess::ManualDownload => "manual_download",
        SourceAccess::ManualClone => "manual_clone",
    }
}

fn build_source_archive_gaps(rows: &[SourceInventoryRow]) -> Vec<SourceArchiveGapRow> {
    let mut gaps = rows
        .iter()
        .filter(|row| row.archive_status == "missing")
        .map(|row| SourceArchiveGapRow {
            source_id: row.source_id.clone(),
            kind: row.kind.clone(),
            access: row.access.clone(),
            locator: row.locator.clone(),
            archive_path: row.archive_path.clone(),
            citation: row.citation.clone(),
            tool_ids: row.tool_ids.clone(),
            reason: "expected archive payload is not present under science-docs".to_string(),
        })
        .collect::<Vec<_>>();
    gaps.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    gaps
}

fn load_dir<T, FKey, FValidate>(
    root: &Path,
    rel: &str,
    out: &mut BTreeMap<String, T>,
    expected_schema: &str,
    key_fn: FKey,
    validate_fn: FValidate,
    errors: &mut Vec<String>,
) -> Result<()>
where
    T: serde::de::DeserializeOwned + SchemaVersion,
    FKey: Fn(&T) -> String,
    FValidate: Fn(&T) -> Result<()>,
{
    for path in list_yaml_files(&root.join(rel))? {
        let raw = read_utf8(&path)?;
        let row: T = bijux_dna_infra::formats::yaml::parse_yaml(&raw)
            .map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
        if row.schema_version() != expected_schema {
            errors.push(format!(
                "{} uses schema {} but {} is required",
                path.display(),
                row.schema_version(),
                expected_schema
            ));
            continue;
        }
        if let Err(err) = validate_fn(&row) {
            errors.push(format!("{}: {err}", path.display()));
            continue;
        }
        let key = key_fn(&row);
        if out.insert(key.clone(), row).is_some() {
            errors.push(format!("{rel} contains duplicate id {key}"));
        }
    }
    Ok(())
}

fn validate_source(row: &SourceSpec) -> Result<()> {
    if row.title.trim().is_empty() {
        return Err(anyhow!("source title must not be empty"));
    }
    if row.locator.trim().is_empty() {
        return Err(anyhow!("source locator must not be empty"));
    }
    if row.authority.trim().is_empty() {
        return Err(anyhow!("source authority must not be empty"));
    }
    match row.access {
        SourceAccess::RepoPath => {
            if row.locator.contains("://") {
                return Err(anyhow!("repo_path sources must use repository-relative locators"));
            }
            if row.archive_path.is_some() {
                return Err(anyhow!("repo_path sources must not declare archive_path"));
            }
            if matches!(
                row.kind,
                SourceKind::ExternalDocument | SourceKind::ExternalRepository | SourceKind::Paper
            ) {
                return Err(anyhow!(
                    "repo_path sources must use repo-local kinds, not external source kinds"
                ));
            }
        }
        SourceAccess::ManualDownload | SourceAccess::ManualClone => {
            if !row.locator.contains("://") {
                return Err(anyhow!(
                    "manual acquisition sources must use an external locator such as https://..."
                ));
            }
            let archive_path =
                row.archive_path.as_deref().filter(|value| !value.trim().is_empty()).ok_or_else(
                    || anyhow!("manual acquisition sources must declare archive_path"),
                )?;
            if !archive_path.starts_with("science-docs/") {
                return Err(anyhow!(
                    "archive_path must live under science-docs/ for manual acquisition sources"
                ));
            }
            if matches!(row.access, SourceAccess::ManualClone)
                && !matches!(row.kind, SourceKind::ExternalRepository)
            {
                return Err(anyhow!("manual_clone sources must use kind external_repository"));
            }
            if matches!(
                row.kind,
                SourceKind::RepoFile | SourceKind::RepoDirectory | SourceKind::Document
            ) {
                return Err(anyhow!(
                    "manual acquisition sources must use external_document, external_repository, or paper kinds"
                ));
            }
        }
    }
    Ok(())
}

fn validate_claim(row: &ClaimSpec) -> Result<()> {
    if row.supports.is_empty() {
        return Err(anyhow!("claim must reference at least one evidence record"));
    }
    for field in [
        ("statement", row.statement.as_str()),
        ("scope", row.scope.as_str()),
        ("subject", row.subject.as_str()),
        ("predicate", row.predicate.as_str()),
        ("object", row.object.as_str()),
        ("owner", row.owner.as_str()),
        ("review_due", row.review_due.as_str()),
    ] {
        if field.1.trim().is_empty() {
            return Err(anyhow!("claim {} must not be empty", field.0));
        }
    }
    Ok(())
}

fn validate_cross_references(loaded: &LoadedSpecs) -> Vec<String> {
    let mut errors = Vec::new();

    for evidence in loaded.evidences.values() {
        for source_id in &evidence.source_ids {
            if !loaded.sources.contains_key(source_id.as_str()) {
                errors.push(format!(
                    "{} references missing source {}",
                    evidence.evidence_id, source_id
                ));
            }
        }
    }
    for claim in loaded.claims.values() {
        for evidence_id in &claim.supports {
            if !loaded.evidences.contains_key(evidence_id.as_str()) {
                errors.push(format!(
                    "{} references missing evidence {}",
                    claim.claim_id, evidence_id
                ));
            }
        }
    }
    for reasoning in loaded.reasonings.values() {
        if reasoning.claim_ids.is_empty() {
            errors.push(format!("{} must reference at least one claim", reasoning.reasoning_id));
        }
        for claim_id in &reasoning.claim_ids {
            if !loaded.claims.contains_key(claim_id.as_str()) {
                errors.push(format!(
                    "{} references missing claim {}",
                    reasoning.reasoning_id, claim_id
                ));
            }
        }
        for evidence_id in &reasoning.evidence_ids {
            if !loaded.evidences.contains_key(evidence_id.as_str()) {
                errors.push(format!(
                    "{} references missing evidence {}",
                    reasoning.reasoning_id, evidence_id
                ));
            }
        }
        for assumption_id in &reasoning.assumption_ids {
            if !loaded.assumptions.contains_key(assumption_id.as_str()) {
                errors.push(format!(
                    "{} references missing assumption {}",
                    reasoning.reasoning_id, assumption_id
                ));
            }
        }
    }
    for decision in loaded.decisions.values() {
        if !loaded.reasonings.contains_key(decision.reasoning.as_str()) {
            errors.push(format!(
                "{} references missing reasoning {}",
                decision.decision_id, decision.reasoning
            ));
        }
        for claim_id in &decision.derived_from {
            if !loaded.claims.contains_key(claim_id.as_str()) {
                errors.push(format!(
                    "{} references missing claim {}",
                    decision.decision_id, claim_id
                ));
            }
        }
    }
    for binding in loaded.bindings.values() {
        if !loaded.decisions.contains_key(binding.decision_id.as_str()) {
            errors.push(format!(
                "{} references missing decision {}",
                binding.binding_id, binding.decision_id
            ));
        }
        for claim_id in &binding.claim_ids {
            if !loaded.claims.contains_key(claim_id.as_str()) {
                errors
                    .push(format!("{} references missing claim {}", binding.binding_id, claim_id));
            }
        }
        for source_id in &binding.source_ids {
            if !loaded.sources.contains_key(source_id.as_str()) {
                errors.push(format!(
                    "{} references missing source {}",
                    binding.binding_id, source_id
                ));
            }
        }
    }
    for release in loaded.releases.values() {
        for binding_id in &release.binding_ids {
            if !loaded.bindings.contains_key(binding_id.as_str()) {
                errors.push(format!(
                    "{} references missing binding {}",
                    release.release_id, binding_id
                ));
            }
        }
        for claim_id in &release.claim_ids {
            if !loaded.claims.contains_key(claim_id.as_str()) {
                errors
                    .push(format!("{} references missing claim {}", release.release_id, claim_id));
            }
        }
    }

    errors.sort();
    errors.dedup();
    errors
}

fn build_claim_evidence_map(loaded: &LoadedSpecs) -> Vec<ClaimEvidenceRow> {
    let mut rows = loaded
        .claims
        .values()
        .flat_map(|claim| {
            claim.supports.iter().map(|evidence_id| ClaimEvidenceRow {
                claim_id: claim.claim_id.to_string(),
                evidence_id: evidence_id.to_string(),
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (&left.claim_id, &left.evidence_id).cmp(&(&right.claim_id, &right.evidence_id))
    });
    rows
}

fn build_decision_reasoning_map(loaded: &LoadedSpecs) -> Vec<DecisionReasoningRow> {
    let mut rows = loaded
        .decisions
        .values()
        .map(|decision| DecisionReasoningRow {
            decision_id: decision.decision_id.to_string(),
            reasoning_id: decision.reasoning.to_string(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.decision_id.cmp(&right.decision_id));
    rows
}

fn build_binding_resolution(loaded: &LoadedSpecs) -> Vec<BindingResolutionRow> {
    let mut rows = loaded
        .bindings
        .values()
        .map(|binding| BindingResolutionRow {
            binding_id: binding.binding_id.to_string(),
            decision_id: binding.decision_id.to_string(),
            target_type: binding.target_type.clone(),
            target_ref: binding.target_ref.clone(),
            enforcement_level: format!("{:?}", binding.enforcement_level).to_ascii_lowercase(),
            status: "resolved".to_string(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.binding_id.cmp(&right.binding_id));
    rows
}

fn build_fastq_container_reference_rows(
    root: &Path,
    loaded: &LoadedSpecs,
    fastq_environment_rows: &[FastqEnvironmentRow],
) -> Result<Vec<crate::domain::FastqContainerReferenceRow>> {
    let registry_source = loaded
        .bindings
        .values()
        .filter(|binding| binding.target_type == "fastq_stage_tool_environment_matrix")
        .find_map(|binding| {
            binding
                .source_ids
                .iter()
                .find(|source_id| source_id.as_str() == "source.fastq.tool-registry")
        })
        .ok_or_else(|| {
            anyhow!("missing source.fastq.tool-registry binding for fastq container matrix")
        })?;
    let registry_path = loaded
        .sources
        .get(registry_source.as_str())
        .ok_or_else(|| anyhow!("missing source record {}", registry_source))?;
    let registry = load_tool_registry(&validate_source_path(root, registry_path)?)?;

    let mut stage_map = BTreeMap::<String, BTreeSet<String>>::new();
    for row in fastq_environment_rows {
        stage_map.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }

    let mut rows = stage_map
        .into_iter()
        .map(|(tool_id, stage_ids)| {
            let entry = registry.get(&tool_id).cloned().unwrap_or_default();
            let reference_status = if entry.container_ref.is_empty()
                && entry.dockerfile.is_empty()
                && entry.apptainer_def.is_empty()
            {
                "missing_registry_metadata".to_string()
            } else {
                "governed".to_string()
            };
            crate::domain::FastqContainerReferenceRow {
                tool_id,
                stage_ids: stage_ids.into_iter().collect::<Vec<_>>().join(","),
                reference_status,
                registry_status: entry.status,
                version: entry.version,
                default_version: entry.default_version,
                version_rule: entry.version_rule,
                upstream: entry.upstream,
                citation: entry.citation,
                license: entry.license,
                pinned_commit: entry.pinned_commit,
                pin_strategy: entry.pin_strategy,
                runtimes: entry.runtimes.join(","),
                container_ref: entry.container_ref,
                dockerfile: entry.dockerfile,
                apptainer_def: entry.apptainer_def,
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(rows)
}

fn build_fastq_download_backlog_rows(
    root: &Path,
    rows: &[crate::domain::FastqContainerReferenceRow],
) -> Vec<crate::domain::FastqDownloadBacklogRow> {
    let mut backlog = rows
        .iter()
        .map(|row| {
            let source_id = format!("source.fastq.tool.{}.upstream", row.tool_id);
            let acquisition_mode = infer_acquisition_mode(&row.upstream);
            let archive_path = if acquisition_mode.is_empty() {
                String::new()
            } else if acquisition_mode == "manual_clone" {
                format!("science-docs/upstream/fastq/tools/{}/repo", row.tool_id)
            } else {
                format!("science-docs/upstream/fastq/tools/{}/download", row.tool_id)
            };
            let archive_status = if archive_path.is_empty() {
                "not_applicable".to_string()
            } else if root.join(&archive_path).exists() {
                "present".to_string()
            } else {
                "missing".to_string()
            };
            let backlog_status = if row.reference_status != "governed" {
                "missing_registry_source".to_string()
            } else if row.upstream.trim().is_empty() {
                "missing_upstream_locator".to_string()
            } else if row.upstream.contains("${") {
                "templated_locator".to_string()
            } else {
                "ready".to_string()
            };
            let notes = match backlog_status.as_str() {
                "missing_registry_source" => {
                    "tool is on the FASTQ surface but lacks governed registry source metadata"
                }
                "missing_upstream_locator" => {
                    "tool has registry metadata but no upstream locator to archive yet"
                }
                "templated_locator" => {
                    "registry locator contains unresolved template variables; archive the resolved release page or source bundle"
                }
                _ => "archive upstream evidence for later review and claim confirmation",
            }
            .to_string();
            crate::domain::FastqDownloadBacklogRow {
                source_id,
                tool_id: row.tool_id.clone(),
                stage_ids: row.stage_ids.clone(),
                acquisition_mode,
                backlog_status,
                locator: row.upstream.clone(),
                citation: row.citation.clone(),
                archive_path,
                archive_status,
                notes,
            }
        })
        .collect::<Vec<_>>();
    backlog.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    backlog
}

fn infer_acquisition_mode(locator: &str) -> String {
    if locator.trim().is_empty() {
        return String::new();
    }
    if locator.contains("github.com/") || locator.contains("gitlab.") {
        if locator.ends_with(".zip")
            || locator.ends_with(".tar.gz")
            || locator.contains("/archive/")
            || locator.contains("/releases/download/")
        {
            "manual_download".to_string()
        } else {
            "manual_clone".to_string()
        }
    } else {
        "manual_download".to_string()
    }
}

fn build_fastq_environment_rows(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<FastqEnvironmentRow>> {
    let mut rows = Vec::new();
    for binding in loaded
        .bindings
        .values()
        .filter(|binding| binding.target_type == "fastq_stage_tool_environment_matrix")
    {
        rows.extend(build_fastq_binding_rows(root, loaded, binding)?);
    }
    rows.sort_by(|left, right| {
        (&left.stage_id, !left.is_default, &left.tool_id).cmp(&(
            &right.stage_id,
            !right.is_default,
            &right.tool_id,
        ))
    });
    Ok(rows)
}

fn build_fastq_binding_rows(
    root: &Path,
    loaded: &LoadedSpecs,
    binding: &BindingSpec,
) -> Result<Vec<FastqEnvironmentRow>> {
    let execution_support = resolve_binding_source(
        root,
        loaded,
        &binding.source_ids,
        "source.fastq.execution-support",
    )?;
    let stage_contracts =
        resolve_binding_source(root, loaded, &binding.source_ids, "source.fastq.stage-contracts")?;
    let tool_registry =
        resolve_binding_source(root, loaded, &binding.source_ids, "source.fastq.tool-registry")?;

    let execution_support_doc: ExecutionSupportDoc =
        bijux_dna_infra::formats::yaml::parse_yaml(&read_utf8(&execution_support)?)
            .map_err(|err| anyhow!("parse {}: {err}", execution_support.display()))?;
    let planned_tools = load_planned_out_of_scope(&stage_contracts)?;
    let registry = load_tool_registry(&tool_registry)?;
    let evidence_count = binding_claim_evidence_count(loaded, binding);
    let claim_ids = binding.claim_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");

    let mut rows = Vec::new();
    for stage in execution_support_doc.stages {
        let mut tool_ids = BTreeSet::new();
        for tool_id in &stage.admitted_tools {
            tool_ids.insert(tool_id.clone());
        }
        if let Some(extra) = planned_tools.get(&stage.stage_id) {
            for tool_id in extra {
                tool_ids.insert(tool_id.clone());
            }
        }
        for tool_id in tool_ids {
            let registry_entry = registry.get(&tool_id).cloned().unwrap_or_default();
            let is_default = tool_id == stage.default_tool;
            let tool_status = if is_default {
                "default".to_string()
            } else if stage.admitted_tools.iter().any(|candidate| candidate == &tool_id) {
                "allowed".to_string()
            } else {
                "disallowed".to_string()
            };
            rows.push(FastqEnvironmentRow {
                stage_id: stage.stage_id.clone(),
                tool_id,
                stage_status: "governed".to_string(),
                tool_status,
                is_default,
                execution_status: stage.execution_status.clone(),
                runtime_support: stage.runtime_support.clone(),
                normalization_support: stage.normalization_support.clone(),
                benchmark_support: stage.benchmark_support.clone(),
                registry_status: registry_entry.status,
                runtimes: registry_entry.runtimes.join(","),
                container_ref: registry_entry.container_ref,
                dockerfile: registry_entry.dockerfile,
                apptainer_def: registry_entry.apptainer_def,
                evidence_count,
                claim_ids: claim_ids.clone(),
                decision_id: binding.decision_id.to_string(),
                binding_id: binding.binding_id.to_string(),
            });
        }
    }
    Ok(rows)
}

fn resolve_binding_source(
    root: &Path,
    loaded: &LoadedSpecs,
    binding_sources: &[SourceId],
    required_id: &str,
) -> Result<PathBuf> {
    let source = binding_sources
        .iter()
        .find(|source_id| source_id.as_str() == required_id)
        .and_then(|source_id| loaded.sources.get(source_id.as_str()))
        .ok_or_else(|| anyhow!("binding is missing required source {required_id}"))?;
    validate_source_path(root, source)
}

fn validate_source_path(root: &Path, source: &SourceSpec) -> Result<PathBuf> {
    let path = root.join(&source.locator);
    match source.kind {
        SourceKind::RepoFile | SourceKind::Document => {
            if !path.is_file() {
                return Err(anyhow!("required source path is not a file: {}", path.display()));
            }
        }
        SourceKind::RepoDirectory => {
            if !path.is_dir() {
                return Err(anyhow!("required source path is not a directory: {}", path.display()));
            }
        }
        SourceKind::ExternalDocument | SourceKind::ExternalRepository | SourceKind::Paper => {
            return Err(anyhow!(
                "external source {} cannot be resolved as a repository path",
                source.source_id
            ));
        }
    }
    Ok(path)
}

fn load_planned_out_of_scope(stage_dir: &Path) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut map = BTreeMap::<String, BTreeSet<String>>::new();
    for path in list_yaml_files(stage_dir)? {
        let raw = read_utf8(&path)?;
        let stage: StageContractDoc = bijux_dna_infra::formats::yaml::parse_yaml(&raw)
            .map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
        let Some(stage_id) = stage.stage_id else {
            continue;
        };
        if !stage.planned_out_of_scope.is_empty() {
            map.insert(stage_id, stage.planned_out_of_scope.into_iter().collect());
        }
    }
    Ok(map)
}

fn load_tool_registry(path: &Path) -> Result<BTreeMap<String, ToolRegistryEntry>> {
    let raw = read_utf8(path)?;
    let root: TomlValue = raw.parse().with_context(|| format!("parse TOML {}", path.display()))?;
    let mut out = BTreeMap::new();
    for row in root.get("tools").and_then(TomlValue::as_array).cloned().unwrap_or_default() {
        let Some(table) = row.as_table() else {
            continue;
        };
        let Some(tool_id) = table
            .get("tool_id")
            .or_else(|| table.get("id"))
            .and_then(TomlValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let runtimes = table
            .get("runtimes")
            .and_then(TomlValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(TomlValue::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        out.insert(
            tool_id.to_string(),
            ToolRegistryEntry {
                status: table
                    .get("status")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version: table
                    .get("version")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                default_version: table
                    .get("default_version")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version_rule: table
                    .get("version_rule")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                upstream: table
                    .get("upstream")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                citation: table
                    .get("citation")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                license: table
                    .get("license")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                pinned_commit: table
                    .get("pinned_commit")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                pin_strategy: table
                    .get("pin_strategy")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                runtimes,
                container_ref: table
                    .get("container_ref")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                dockerfile: table
                    .get("dockerfile")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                apptainer_def: table
                    .get("apptainer_def")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
        );
    }
    Ok(out)
}

fn binding_claim_evidence_count(loaded: &LoadedSpecs, binding: &BindingSpec) -> usize {
    let mut evidence_ids = BTreeSet::new();
    for claim_id in &binding.claim_ids {
        if let Some(claim) = loaded.claims.get(claim_id.as_str()) {
            for evidence_id in &claim.supports {
                evidence_ids.insert(evidence_id.to_string());
            }
        }
    }
    evidence_ids.len()
}

pub trait SchemaVersion {
    fn schema_version(&self) -> &str;
}

impl SchemaVersion for crate::domain::SourceSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::EvidenceSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ClaimSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::AssumptionSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ReasoningSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::DecisionSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::BindingSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ReleaseManifestSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}
