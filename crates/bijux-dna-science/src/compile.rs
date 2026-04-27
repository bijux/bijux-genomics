use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use toml::Value as TomlValue;

use crate::domain::{
    BindingResolutionRow, BindingSpec, ClaimEvidenceRow, ClaimSpec, CompiledScience,
    DecisionReasoningRow, FastqClosureGateRow, FastqClosureSummary, FastqDefaultBindingRiskRow,
    FastqEnvironmentRow, FastqEvidenceSummary, FastqMissingClosurePrerequisiteRow,
    FastqTruthDeltaRow, LoadedSpecs, ScienceIndex, SourceAccess, SourceArchiveGapRow,
    SourceArchiveSummary, SourceId, SourceInventoryRow, SourceKind, SourceSpec,
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
    default_tool: Option<String>,
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

#[derive(Clone, Debug, Default, Deserialize)]
struct FastqToolContractEntry {
    #[serde(default)]
    tool_id: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    default_version: String,
    #[serde(default)]
    versioning_strategy: String,
    #[serde(default)]
    pin_strategy: String,
    #[serde(default)]
    upstream: String,
    #[serde(default)]
    citation: String,
    #[serde(default)]
    license: String,
}

#[derive(Clone, Debug)]
struct ToolEvidenceMapEntry {
    source_id: String,
    tool_id: String,
    archive_path: String,
    paper_root: String,
    acquisition_mode: String,
    primary_locator: String,
}

#[derive(Clone, Debug)]
struct PaperMapEntry {
    paper_id: String,
    tool_id: String,
    paper_root: String,
    paper_status: String,
    open_access_status: String,
    primary_locator: String,
    supporting_locators: String,
    notes: String,
}

#[derive(Clone, Debug)]
struct QaCoverageBlockerEntry {
    stage_id: String,
    blocker: String,
}

/// Load and validate authored science specs from a workspace.
///
/// # Errors
///
/// Returns an error when spec files cannot be listed, read, parsed, or validated.
pub fn load_specs(root: &Path) -> Result<LoadedSpecs> {
    let mut loaded = LoadedSpecs::default();
    let mut errors = Vec::new();

    load_dir::<crate::domain::SourceSpec, _, _>(
        root,
        "science/specs/evidence/sources",
        &mut loaded.sources,
        SOURCE_SCHEMA_VERSION,
        |row| row.source_id.as_str().to_string(),
        validate_source,
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
        validate_claim,
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
        return Err(validation_error(&errors));
    }
    Ok(loaded)
}

/// Compile authored science specs and derived evidence rows for a workspace.
///
/// # Errors
///
/// Returns an error when specs cannot be loaded or derived FASTQ evidence cannot be compiled.
pub fn compile_workspace(root: &Path) -> Result<CompiledScience> {
    let loaded = load_specs(root)?;
    compile_loaded(root, &loaded)
}

/// Compile already loaded science specs into deterministic generated models.
///
/// # Errors
///
/// Returns an error when loaded specs contain unresolved references or required FASTQ evidence
/// sources cannot be resolved.
pub fn compile_loaded(root: &Path, loaded: &LoadedSpecs) -> Result<CompiledScience> {
    let source_inventory = build_source_inventory(root, loaded);
    let source_archive_gaps = build_source_archive_gaps(&source_inventory);
    let source_archive_summary =
        build_source_archive_summary(&source_inventory, &source_archive_gaps);
    let claim_evidence_map = build_claim_evidence_map(loaded);
    let decision_reasoning_map = build_decision_reasoning_map(loaded);
    let binding_resolution = build_binding_resolution(loaded);
    let fastq_environment_rows = build_fastq_environment_rows(root, loaded)?;
    let fastq_container_reference_rows =
        build_fastq_container_reference_rows(root, loaded, &fastq_environment_rows)?;
    let tool_evidence_map = load_fastq_tool_evidence_map(root, loaded)?;
    let paper_map = load_fastq_paper_map(root, loaded)?;
    let qa_coverage_blockers = load_fastq_qa_coverage_blockers(root, loaded)?;
    let fastq_download_backlog_rows = build_fastq_download_backlog_rows(
        root,
        &fastq_container_reference_rows,
        &tool_evidence_map,
        &paper_map,
    );
    let fastq_paper_archive_rows =
        build_fastq_paper_archive_rows(root, &fastq_environment_rows, &paper_map);
    let fastq_closure_gate_rows = build_fastq_closure_gate_rows(
        &fastq_environment_rows,
        &fastq_download_backlog_rows,
        &fastq_paper_archive_rows,
        &qa_coverage_blockers,
    );
    let fastq_truth_delta_rows = build_fastq_truth_delta_rows(&fastq_closure_gate_rows);
    let fastq_missing_closure_prerequisite_rows =
        build_fastq_missing_closure_prerequisite_rows(&fastq_closure_gate_rows);
    let fastq_default_binding_risk_rows =
        build_fastq_default_binding_risk_rows(&fastq_closure_gate_rows);
    let fastq_closure_summary = build_fastq_closure_summary(&fastq_closure_gate_rows);
    let fastq_evidence_summary = build_fastq_evidence_summary(
        &fastq_download_backlog_rows,
        &fastq_paper_archive_rows,
        &fastq_missing_closure_prerequisite_rows,
        &fastq_default_binding_risk_rows,
        &fastq_truth_delta_rows,
    );
    let unresolved_refs = validate_cross_references(loaded);
    if !unresolved_refs.is_empty() {
        return Err(validation_error(&unresolved_refs));
    }
    let index = ScienceIndex {
        sources: loaded.sources.len(),
        source_inventory_rows: source_inventory.len(),
        source_archive_gap_rows: source_archive_gaps.len(),
        source_archive_summary,
        evidences: loaded.evidences.len(),
        claims: loaded.claims.len(),
        assumptions: loaded.assumptions.len(),
        reasonings: loaded.reasonings.len(),
        decisions: loaded.decisions.len(),
        bindings: loaded.bindings.len(),
        releases: loaded.releases.len(),
        fastq_container_reference_rows: fastq_container_reference_rows.len(),
        fastq_download_backlog_rows: fastq_download_backlog_rows.len(),
        fastq_paper_archive_rows: fastq_paper_archive_rows.len(),
        fastq_environment_rows: fastq_environment_rows.len(),
        fastq_closure_gate_rows: fastq_closure_gate_rows.len(),
        fastq_truth_delta_rows: fastq_truth_delta_rows.len(),
        fastq_missing_closure_prerequisite_rows: fastq_missing_closure_prerequisite_rows.len(),
        fastq_default_binding_risk_rows: fastq_default_binding_risk_rows.len(),
        fastq_closure_summary,
        fastq_evidence_summary,
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
        fastq_paper_archive_rows,
        fastq_environment_rows,
        fastq_closure_gate_rows,
        fastq_truth_delta_rows,
        fastq_missing_closure_prerequisite_rows,
        fastq_default_binding_risk_rows,
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

fn build_fastq_closure_summary(rows: &[FastqClosureGateRow]) -> FastqClosureSummary {
    let mut blocking_reason_counts = BTreeMap::new();
    let mut warning_reason_counts = BTreeMap::new();
    for row in rows {
        increment_compound_counts(&mut blocking_reason_counts, &row.blocking_reasons);
        increment_compound_counts(&mut warning_reason_counts, &row.warning_reasons);
    }
    FastqClosureSummary {
        total_rows: rows.len(),
        default_rows: rows.iter().filter(|row| row.is_default).count(),
        world_class_closed_rows: rows.iter().filter(|row| row.world_class_closed).count(),
        declared_closed_with_gaps_rows: rows
            .iter()
            .filter(|row| row.effective_closure_status == "declared_closed_with_gaps")
            .count(),
        not_closed_rows: rows
            .iter()
            .filter(|row| row.effective_closure_status == "not_closed")
            .count(),
        blocking_reason_counts,
        warning_reason_counts,
    }
}

fn build_source_archive_summary(
    inventory_rows: &[SourceInventoryRow],
    gap_rows: &[SourceArchiveGapRow],
) -> SourceArchiveSummary {
    let mut kind_counts = BTreeMap::new();
    let mut access_counts = BTreeMap::new();
    let mut archive_status_counts = BTreeMap::new();
    let mut missing_tool_counts = BTreeMap::new();

    for row in inventory_rows {
        increment_scalar_count(&mut kind_counts, &row.kind);
        increment_scalar_count(&mut access_counts, &row.access);
        increment_scalar_count(&mut archive_status_counts, &row.archive_status);
    }

    for row in gap_rows {
        increment_compound_counts(&mut missing_tool_counts, &row.tool_ids);
    }

    SourceArchiveSummary { kind_counts, access_counts, archive_status_counts, missing_tool_counts }
}

fn build_fastq_evidence_summary(
    backlog_rows: &[crate::domain::FastqDownloadBacklogRow],
    paper_rows: &[crate::domain::FastqPaperArchiveRow],
    prerequisite_rows: &[FastqMissingClosurePrerequisiteRow],
    default_risk_rows: &[FastqDefaultBindingRiskRow],
    truth_delta_rows: &[FastqTruthDeltaRow],
) -> FastqEvidenceSummary {
    let mut backlog_status_counts = BTreeMap::new();
    let mut paper_status_counts = BTreeMap::new();
    let mut paper_archive_status_counts = BTreeMap::new();
    let mut prerequisite_counts = BTreeMap::new();
    let mut default_risk_counts = BTreeMap::new();
    let mut truth_delta_reason_counts = BTreeMap::new();

    for row in backlog_rows {
        increment_scalar_count(&mut backlog_status_counts, &row.backlog_status);
    }
    for row in paper_rows {
        increment_scalar_count(&mut paper_status_counts, &row.paper_status);
        increment_scalar_count(&mut paper_archive_status_counts, &row.archive_status);
    }
    for row in prerequisite_rows {
        increment_scalar_count(&mut prerequisite_counts, &row.prerequisite);
    }
    for row in default_risk_rows {
        increment_scalar_count(&mut default_risk_counts, &row.risk_class);
    }
    for row in truth_delta_rows {
        increment_compound_counts(&mut truth_delta_reason_counts, &row.reason);
    }

    FastqEvidenceSummary {
        backlog_status_counts,
        paper_status_counts,
        paper_archive_status_counts,
        prerequisite_counts,
        default_risk_counts,
        truth_delta_reason_counts,
    }
}

fn increment_scalar_count(counts: &mut BTreeMap<String, usize>, value: &str) {
    let value = value.trim();
    if value.is_empty() || value == "not_applicable" {
        return;
    }
    *counts.entry(value.to_string()).or_insert(0) += 1;
}

fn increment_compound_counts(counts: &mut BTreeMap<String, usize>, value: &str) {
    for label in value.split(';').map(str::trim) {
        increment_scalar_count(counts, label);
    }
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
            reason: "expected archive payload is not present under science/docs".to_string(),
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
        let document = read_utf8(&path)?;
        let spec: T = bijux_dna_infra::formats::yaml::parse_yaml(&document)
            .map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
        if spec.schema_version() != expected_schema {
            errors.push(format!(
                "{} uses schema {} but {} is required",
                path.display(),
                spec.schema_version(),
                expected_schema
            ));
            continue;
        }
        if let Err(err) = validate_fn(&spec) {
            errors.push(format!("{}: {err}", path.display()));
            continue;
        }
        let key = key_fn(&spec);
        if out.insert(key.clone(), spec).is_some() {
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
            if !archive_path.starts_with("science/docs/") {
                return Err(anyhow!(
                    "archive_path must live under science/docs/ for manual acquisition sources"
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
    validate_evidence_references(loaded, &mut errors);
    validate_claim_references(loaded, &mut errors);
    validate_reasoning_references(loaded, &mut errors);
    validate_decision_references(loaded, &mut errors);
    validate_binding_references(loaded, &mut errors);
    validate_release_references(loaded, &mut errors);
    errors.sort();
    errors.dedup();
    errors
}

fn validate_evidence_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
}

fn validate_claim_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
}

fn validate_reasoning_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
}

fn validate_decision_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
}

fn validate_binding_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
}

fn validate_release_references(loaded: &LoadedSpecs, errors: &mut Vec<String>) {
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
        .ok_or_else(|| anyhow!("missing source record {registry_source}"))?;
    let registry = load_tool_registry(&validate_source_path(root, registry_path)?)?;
    let tool_contracts = load_fastq_tool_contracts(&root.join("domain/fastq/tools"))?;

    let mut stage_map = BTreeMap::<String, BTreeSet<String>>::new();
    for row in fastq_environment_rows {
        stage_map.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }

    let mut rows = stage_map
        .into_iter()
        .map(|(tool_id, stage_ids)| {
            let entry = merged_tool_metadata(
                registry.get(&tool_id).cloned().unwrap_or_default(),
                tool_contracts.get(&tool_id),
            );
            let has_container_metadata = !(entry.container_ref.is_empty()
                && entry.dockerfile.is_empty()
                && entry.apptainer_def.is_empty());
            let has_source_metadata =
                !(entry.upstream.trim().is_empty() && entry.citation.trim().is_empty());
            let reference_status = if has_container_metadata {
                "governed".to_string()
            } else if has_source_metadata {
                "tool_contract_metadata".to_string()
            } else {
                "missing_source_metadata".to_string()
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
    evidence_map: &[ToolEvidenceMapEntry],
    paper_map: &[PaperMapEntry],
) -> Vec<crate::domain::FastqDownloadBacklogRow> {
    let evidence_by_tool = evidence_map
        .iter()
        .map(|entry| (entry.tool_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let paper_by_root = paper_map
        .iter()
        .map(|entry| (entry.paper_root.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut backlog = rows
        .iter()
        .map(|row| {
            let tool_entry = evidence_by_tool.get(row.tool_id.as_str()).copied();
            let source_id = tool_entry.map_or_else(
                || format!("source.fastq.tool.{}.upstream", row.tool_id),
                |entry| entry.source_id.clone(),
            );
            let acquisition_mode = tool_entry.map_or_else(
                || infer_acquisition_mode(&row.upstream),
                |entry| entry.acquisition_mode.clone(),
            );
            let archive_path = tool_entry.map_or_else(
                || default_archive_path(&row.tool_id, &acquisition_mode),
                |entry| entry.archive_path.clone(),
            );
            let archive_status = if archive_path.is_empty() {
                "not_applicable".to_string()
            } else if root.join(&archive_path).exists() {
                "present".to_string()
            } else {
                "missing".to_string()
            };
            let locator = tool_entry
                .map_or_else(|| row.upstream.clone(), |entry| entry.primary_locator.clone());
            let paper_root = tool_entry.map(|entry| entry.paper_root.clone()).unwrap_or_default();
            let paper_status = if paper_root.is_empty() {
                "missing_paper_root".to_string()
            } else {
                paper_by_root.get(paper_root.as_str()).map_or_else(
                    || "missing_paper_map".to_string(),
                    |entry| entry.paper_status.clone(),
                )
            };
            let has_source_metadata =
                !(locator.trim().is_empty() && row.citation.trim().is_empty());
            let backlog_status = if tool_entry.is_none() {
                "missing_evidence_map".to_string()
            } else if paper_root.is_empty() {
                "missing_paper_root".to_string()
            } else if paper_status == "missing_paper_map" {
                "missing_paper_map".to_string()
            } else if !has_source_metadata {
                "missing_registry_source".to_string()
            } else if locator.trim().is_empty() {
                "missing_upstream_locator".to_string()
            } else if locator.contains("${") {
                "templated_locator".to_string()
            } else {
                "ready".to_string()
            };
            let notes = backlog_notes(&backlog_status).to_string();
            crate::domain::FastqDownloadBacklogRow {
                source_id,
                tool_id: row.tool_id.clone(),
                stage_ids: row.stage_ids.clone(),
                acquisition_mode,
                backlog_status,
                locator,
                citation: row.citation.clone(),
                archive_path,
                archive_status,
                paper_root,
                paper_status,
                notes,
            }
        })
        .collect::<Vec<_>>();
    backlog.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    backlog
}

fn backlog_notes(backlog_status: &str) -> &'static str {
    match backlog_status {
        "missing_evidence_map" => {
            "tool is on the FASTQ surface but lacks a governed tool evidence map row"
        }
        "missing_paper_root" => {
            "tool evidence row exists but does not yet point at a durable paper root"
        }
        "missing_paper_map" => {
            "tool evidence row points at a paper root that is not registered in the tool paper map"
        }
        "missing_registry_source" => {
            "tool is on the FASTQ surface but lacks governed registry source metadata"
        }
        "missing_upstream_locator" => "tool has registry metadata but no upstream locator to archive yet",
        "templated_locator" => {
            "registry locator contains unresolved template variables; archive the resolved release page or source bundle"
        }
        _ => "archive upstream evidence for later review and claim confirmation",
    }
}

fn infer_acquisition_mode(locator: &str) -> String {
    if locator.trim().is_empty() {
        return String::new();
    }
    if locator.contains("github.com/") || locator.contains("gitlab.") {
        if has_download_archive_suffix(locator)
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

fn has_download_archive_suffix(locator: &str) -> bool {
    let path = Path::new(locator);
    if path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("zip")) {
        return true;
    }
    let Some((archive_stem, extension)) = locator.rsplit_once('.') else {
        return false;
    };
    extension.eq_ignore_ascii_case("gz")
        && archive_stem
            .rsplit_once('.')
            .is_some_and(|(_, inner_extension)| inner_extension.eq_ignore_ascii_case("tar"))
}

fn default_archive_path(tool_id: &str, acquisition_mode: &str) -> String {
    if acquisition_mode.is_empty() {
        String::new()
    } else if acquisition_mode == "manual_clone" {
        format!("science/docs/upstream/fastq/tools/{tool_id}/repo")
    } else {
        format!("science/docs/upstream/fastq/tools/{tool_id}/download")
    }
}

fn build_fastq_paper_archive_rows(
    root: &Path,
    environment_rows: &[FastqEnvironmentRow],
    paper_map: &[PaperMapEntry],
) -> Vec<crate::domain::FastqPaperArchiveRow> {
    let mut stage_map = BTreeMap::<String, BTreeSet<String>>::new();
    for row in environment_rows {
        stage_map.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }
    let mut rows = paper_map
        .iter()
        .map(|entry| crate::domain::FastqPaperArchiveRow {
            paper_id: entry.paper_id.clone(),
            tool_id: entry.tool_id.clone(),
            stage_ids: stage_map
                .get(&entry.tool_id)
                .map(|stage_ids| stage_ids.iter().cloned().collect::<Vec<_>>().join(","))
                .unwrap_or_default(),
            paper_root: entry.paper_root.clone(),
            paper_status: entry.paper_status.clone(),
            open_access_status: entry.open_access_status.clone(),
            primary_locator: entry.primary_locator.clone(),
            supporting_locators: entry.supporting_locators.clone(),
            archive_status: if root.join(&entry.paper_root).exists() {
                "present".to_string()
            } else {
                "missing".to_string()
            },
            notes: entry.notes.clone(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.paper_id.cmp(&right.paper_id))
    });
    rows
}

fn build_fastq_closure_gate_rows(
    environment_rows: &[FastqEnvironmentRow],
    download_rows: &[crate::domain::FastqDownloadBacklogRow],
    paper_rows: &[crate::domain::FastqPaperArchiveRow],
    qa_coverage_blockers: &[QaCoverageBlockerEntry],
) -> Vec<FastqClosureGateRow> {
    let download_by_tool =
        download_rows.iter().map(|row| (row.tool_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let paper_by_tool = paper_rows.iter().fold(
        BTreeMap::<&str, Vec<&crate::domain::FastqPaperArchiveRow>>::new(),
        |mut acc, row| {
            acc.entry(row.tool_id.as_str()).or_default().push(row);
            acc
        },
    );
    let qa_blockers_by_stage = qa_coverage_blockers.iter().fold(
        BTreeMap::<&str, BTreeSet<&str>>::new(),
        |mut acc, blocker| {
            acc.entry(blocker.stage_id.as_str()).or_default().insert(blocker.blocker.as_str());
            acc
        },
    );

    let mut rows = environment_rows
        .iter()
        .map(|row| {
            let blockers =
                closure_blockers(row, &download_by_tool, &paper_by_tool, &qa_blockers_by_stage);
            let warnings = closure_warnings(row);
            let effective_closure_status = if blockers.is_empty() {
                "world_class_closed"
            } else if row.execution_status == "closed" {
                "declared_closed_with_gaps"
            } else {
                "not_closed"
            }
            .to_string();
            FastqClosureGateRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                is_default: row.is_default,
                requested_execution_status: row.execution_status.clone(),
                effective_closure_status,
                world_class_closed: blockers.is_empty(),
                blocking_reasons: blockers.join(";"),
                warning_reasons: warnings.join(";"),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (&left.stage_id, !left.is_default, &left.tool_id).cmp(&(
            &right.stage_id,
            !right.is_default,
            &right.tool_id,
        ))
    });
    rows
}

fn closure_blockers(
    row: &FastqEnvironmentRow,
    download_by_tool: &BTreeMap<&str, &crate::domain::FastqDownloadBacklogRow>,
    paper_by_tool: &BTreeMap<&str, Vec<&crate::domain::FastqPaperArchiveRow>>,
    qa_blockers_by_stage: &BTreeMap<&str, BTreeSet<&str>>,
) -> Vec<String> {
    let mut blockers = Vec::<String>::new();
    add_environment_blockers(row, &mut blockers);
    add_download_blockers(download_by_tool.get(row.tool_id.as_str()).copied(), &mut blockers);
    add_paper_blockers(row, paper_by_tool, &mut blockers);
    if row.benchmark_support == "none" {
        blockers.push("missing_benchmark_support".to_string());
    }
    if let Some(stage_qa_blockers) = qa_blockers_by_stage.get(row.stage_id.as_str()) {
        blockers.extend(stage_qa_blockers.iter().map(|blocker| (*blocker).to_string()));
    }
    blockers
}

fn add_environment_blockers(row: &FastqEnvironmentRow, blockers: &mut Vec<String>) {
    if row.execution_status != "closed" {
        blockers.push("stage_not_marked_closed".to_string());
    }
    if row.registry_status != "production" {
        blockers.push("registry_not_production".to_string());
    }
    if row.container_ref.trim().is_empty() {
        blockers.push("missing_container_ref".to_string());
    }
    if row.container_ref.contains("sha256:pending") {
        blockers.push("pending_container_digest".to_string());
    }
    if row
        .container_ref
        .contains("@sha256:0000000000000000000000000000000000000000000000000000000000000000")
    {
        blockers.push("placeholder_container_digest".to_string());
    }
    if row.runtimes.trim().is_empty() {
        blockers.push("missing_runtime_surface".to_string());
    }
}

fn add_download_blockers(
    download: Option<&crate::domain::FastqDownloadBacklogRow>,
    blockers: &mut Vec<String>,
) {
    match download {
        Some(download) => {
            if download.backlog_status != "ready" {
                blockers.push(format!("source_backlog_{}", download.backlog_status));
            }
            if download.archive_status == "missing" {
                blockers.push("missing_upstream_archive".to_string());
            }
            if matches!(download.paper_status.as_str(), "missing_paper_root" | "missing_paper_map")
            {
                blockers.push(download.paper_status.clone());
            }
        }
        None => blockers.push("missing_download_backlog_row".to_string()),
    }
}

fn add_paper_blockers(
    row: &FastqEnvironmentRow,
    paper_by_tool: &BTreeMap<&str, Vec<&crate::domain::FastqPaperArchiveRow>>,
    blockers: &mut Vec<String>,
) {
    let has_present_paper = paper_by_tool
        .get(row.tool_id.as_str())
        .is_some_and(|rows| rows.iter().any(|paper| paper.archive_status == "present"));
    if !has_present_paper {
        blockers.push("missing_paper_archive".to_string());
    }
}

fn closure_warnings(row: &FastqEnvironmentRow) -> Vec<String> {
    let mut warnings = Vec::new();
    if row.tool_status == "disallowed" {
        warnings.push("planned_binding_not_admitted".to_string());
    }
    if !row.is_default {
        warnings.push("non_default_binding".to_string());
    }
    warnings
}

fn build_fastq_truth_delta_rows(rows: &[FastqClosureGateRow]) -> Vec<FastqTruthDeltaRow> {
    let mut deltas = Vec::new();
    for row in rows {
        if row.requested_execution_status == "closed" && !row.world_class_closed {
            deltas.push(FastqTruthDeltaRow {
                entity_type: "fastq_stage_tool_binding".to_string(),
                entity_id: format!("{}:{}", row.stage_id, row.tool_id),
                layer: "closure_gate".to_string(),
                expected_status: "world_class_closed".to_string(),
                observed_status: row.effective_closure_status.clone(),
                reason: row.blocking_reasons.clone(),
            });
        }
    }
    deltas
}

fn build_fastq_missing_closure_prerequisite_rows(
    rows: &[FastqClosureGateRow],
) -> Vec<FastqMissingClosurePrerequisiteRow> {
    let mut missing = Vec::new();
    for row in rows {
        for reason in row.blocking_reasons.split(';').filter(|reason| !reason.trim().is_empty()) {
            missing.push(FastqMissingClosurePrerequisiteRow {
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                prerequisite: reason.to_string(),
                severity: if row.is_default { "blocking" } else { "advisory" }.to_string(),
                detail: format!(
                    "{} cannot be treated as world-class closed for {} until {} is resolved",
                    row.tool_id, row.stage_id, reason
                ),
            });
        }
    }
    missing.sort_by(|left, right| {
        (&left.stage_id, &left.tool_id, &left.prerequisite).cmp(&(
            &right.stage_id,
            &right.tool_id,
            &right.prerequisite,
        ))
    });
    missing
}

fn build_fastq_default_binding_risk_rows(
    rows: &[FastqClosureGateRow],
) -> Vec<FastqDefaultBindingRiskRow> {
    rows.iter()
        .filter(|row| row.is_default)
        .map(|row| FastqDefaultBindingRiskRow {
            stage_id: row.stage_id.clone(),
            default_tool_id: row.tool_id.clone(),
            requested_execution_status: row.requested_execution_status.clone(),
            effective_closure_status: row.effective_closure_status.clone(),
            risk_class: if row.world_class_closed {
                "world_class_closed".to_string()
            } else if row.blocking_reasons.contains("pending_container_digest") {
                "immutable_container_blocked".to_string()
            } else if row.blocking_reasons.contains("missing_upstream_archive")
                || row.blocking_reasons.contains("missing_paper_archive")
            {
                "archive_evidence_blocked".to_string()
            } else {
                "closure_prerequisite_blocked".to_string()
            },
            blocking_reasons: row.blocking_reasons.clone(),
            warning_reasons: row.warning_reasons.clone(),
        })
        .collect()
}

fn merged_tool_metadata(
    mut registry: ToolRegistryEntry,
    contract: Option<&FastqToolContractEntry>,
) -> ToolRegistryEntry {
    let Some(contract) = contract else {
        return registry;
    };
    if registry.status.is_empty() {
        registry.status.clone_from(&contract.status);
    }
    if registry.default_version.is_empty() {
        registry.default_version.clone_from(&contract.default_version);
    }
    if registry.version_rule.is_empty() {
        registry.version_rule = if contract.versioning_strategy.is_empty() {
            contract.pin_strategy.clone()
        } else {
            contract.versioning_strategy.clone()
        };
    }
    if registry.upstream.is_empty() {
        registry.upstream.clone_from(&contract.upstream);
    }
    if registry.citation.is_empty() || registry.citation == "pending:tool-publication" {
        registry.citation.clone_from(&contract.citation);
    }
    if registry.license.is_empty() {
        registry.license.clone_from(&contract.license);
    }
    if registry.pin_strategy.is_empty() {
        registry.pin_strategy.clone_from(&contract.pin_strategy);
    }
    registry
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
            let is_default = stage.default_tool.as_ref().is_some_and(|default| default == &tool_id);
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

fn load_fastq_tool_contracts(path: &Path) -> Result<BTreeMap<String, FastqToolContractEntry>> {
    let mut out = BTreeMap::new();
    for tool_path in list_yaml_files(path)? {
        let raw = read_utf8(&tool_path)?;
        let entry: FastqToolContractEntry = bijux_dna_infra::formats::yaml::parse_yaml(&raw)
            .map_err(|err| anyhow!("parse {}: {err}", tool_path.display()))?;
        if !entry.tool_id.trim().is_empty() {
            out.insert(entry.tool_id.clone(), entry);
        }
    }
    Ok(out)
}

fn load_fastq_tool_evidence_map(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<ToolEvidenceMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.tool-evidence-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        out.push(ToolEvidenceMapEntry {
            source_id: row_value(&row, "source_id"),
            tool_id: row_value(&row, "tool_id"),
            archive_path: row_value(&row, "archive_path"),
            paper_root: row_value(&row, "paper_root"),
            acquisition_mode: row_value(&row, "acquisition_mode"),
            primary_locator: row_value(&row, "primary_locator"),
        });
    }
    Ok(out)
}

fn load_fastq_paper_map(root: &Path, loaded: &LoadedSpecs) -> Result<Vec<PaperMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.paper-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        out.push(PaperMapEntry {
            paper_id: row_value(&row, "paper_id"),
            tool_id: row_value(&row, "tool_id"),
            paper_root: row_value(&row, "paper_root"),
            paper_status: row_value(&row, "paper_status"),
            open_access_status: row_value(&row, "open_access_status"),
            primary_locator: row_value(&row, "primary_locator"),
            supporting_locators: row_value(&row, "supporting_locators"),
            notes: row_value(&row, "notes"),
        });
    }
    Ok(out)
}

fn load_fastq_qa_coverage_blockers(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<QaCoverageBlockerEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.qa-coverage-blockers") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        if row_value(&row, "status") == "tracked" {
            out.push(QaCoverageBlockerEntry {
                stage_id: row_value(&row, "stage_id"),
                blocker: row_value(&row, "blocker"),
            });
        }
    }
    Ok(out)
}

fn parse_tsv_rows(text: &str) -> Vec<BTreeMap<String, String>> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers = header_line.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        let values = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        let mut parsed_row = BTreeMap::new();
        for (index, header) in headers.iter().enumerate() {
            parsed_row.insert(header.clone(), values.get(index).cloned().unwrap_or_default());
        }
        rows.push(parsed_row);
    }
    rows
}

fn row_value(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
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
