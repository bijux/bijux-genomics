use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::Result;

use crate::decision::fastq_closure::{
    build_fastq_closure_gate_rows, build_fastq_default_binding_risk_rows,
    build_fastq_missing_closure_prerequisite_rows, build_fastq_truth_delta_rows,
};
use crate::domain::compilation::{
    CompilationEvidence, FastqToolContractEntry, PaperMapEntry, ToolEvidenceMapEntry,
    ToolRegistryEntry,
};
use crate::domain::{
    ArchiveStatus, BindingResolutionRow, BindingSpec, ClaimEvidenceRow, CompiledScience,
    DecisionReasoningRow, FastqClosureGateRow, FastqClosureSummary, FastqDefaultBindingRiskRow,
    FastqEnvironmentRow, FastqEvidenceSummary, FastqMissingClosurePrerequisiteRow,
    FastqTruthDeltaRow, LoadedSpecs, ScienceIndex, SourceAccess, SourceArchiveGapRow,
    SourceArchiveSummary, SourceInventoryRow, SourceKind,
};
use crate::errors::validation_error;
use crate::io::compilation::load_compilation_evidence;
pub use crate::io::load_specs;
use crate::schema::authored::validate_cross_references;
pub use crate::schema::SchemaVersion;

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
    let evidence = load_compilation_evidence(root, loaded)?;
    compile_evidence(loaded, &evidence)
}

fn compile_evidence(
    loaded: &LoadedSpecs,
    evidence: &CompilationEvidence,
) -> Result<CompiledScience> {
    let source_inventory = build_source_inventory(loaded);
    let source_archive_gaps = build_source_archive_gaps(&source_inventory);
    let source_archive_summary =
        build_source_archive_summary(&source_inventory, &source_archive_gaps);
    let claim_evidence_map = build_claim_evidence_map(loaded);
    let decision_reasoning_map = build_decision_reasoning_map(loaded);
    let binding_resolution = build_binding_resolution(loaded);
    let fastq_environment_rows = build_fastq_environment_rows(loaded, evidence);
    let fastq_container_reference_rows =
        build_fastq_container_reference_rows(evidence, &fastq_environment_rows);
    let tool_evidence_map = &evidence.tool_evidence_map;
    let paper_map = &evidence.paper_map;
    let qa_coverage_blockers = &evidence.qa_coverage_blockers;
    let fastq_download_backlog_rows = build_fastq_download_backlog_rows(
        &fastq_container_reference_rows,
        tool_evidence_map,
        paper_map,
    );
    let fastq_paper_archive_rows =
        build_fastq_paper_archive_rows(&fastq_environment_rows, paper_map);
    let fastq_closure_gate_rows = build_fastq_closure_gate_rows(
        &fastq_environment_rows,
        &fastq_download_backlog_rows,
        &fastq_paper_archive_rows,
        qa_coverage_blockers,
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

fn build_source_inventory(loaded: &LoadedSpecs) -> Vec<SourceInventoryRow> {
    let mut rows = loaded
        .sources
        .values()
        .map(|source| {
            let archive_path = source.archive_path.clone().unwrap_or_default();
            let archive_status = match source.access {
                SourceAccess::RepoPath => "not_applicable".to_string(),
                SourceAccess::ManualDownload | SourceAccess::ManualClone => source
                    .archive_status
                    .as_ref()
                    .map_or_else(|| "missing".to_string(), archive_status_label),
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
    evidence: &CompilationEvidence,
    fastq_environment_rows: &[FastqEnvironmentRow],
) -> Vec<crate::domain::FastqContainerReferenceRow> {
    let registry = &evidence.registry;
    let tool_contracts = &evidence.tool_contracts;
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
    rows
}

fn build_fastq_download_backlog_rows(
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
            } else {
                tool_entry
                    .map_or_else(|| "missing".to_string(), |entry| entry.archive_status.clone())
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
            archive_status: entry.archive_status.clone(),
            notes: entry.notes.clone(),
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.paper_id.cmp(&right.paper_id))
    });
    rows
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
    loaded: &LoadedSpecs,
    evidence: &CompilationEvidence,
) -> Vec<FastqEnvironmentRow> {
    let mut rows = Vec::new();
    for binding in loaded
        .bindings
        .values()
        .filter(|binding| binding.target_type == "fastq_stage_tool_environment_matrix")
    {
        rows.extend(build_fastq_binding_rows(loaded, binding, evidence));
    }
    rows.sort_by(|left, right| {
        (&left.stage_id, !left.is_default, &left.tool_id).cmp(&(
            &right.stage_id,
            !right.is_default,
            &right.tool_id,
        ))
    });
    rows
}

fn build_fastq_binding_rows(
    loaded: &LoadedSpecs,
    binding: &BindingSpec,
    evidence: &CompilationEvidence,
) -> Vec<FastqEnvironmentRow> {
    let execution_support_doc = &evidence.execution_support;
    let planned_tools = &evidence.planned_tools;
    let registry = &evidence.registry;
    let evidence_count = binding_claim_evidence_count(loaded, binding);
    let claim_ids = binding.claim_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");

    let mut rows = Vec::new();
    for stage in &execution_support_doc.stages {
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
    rows
}

fn archive_status_label(status: &ArchiveStatus) -> String {
    match status {
        ArchiveStatus::Missing => "missing",
        ArchiveStatus::Present => "present",
    }
    .to_string()
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

#[cfg(test)]
#[path = "../tests/unit/compilation.rs"]
mod tests;
