//! Pure authored source, claim and cross-reference admission; no filesystem access.
use crate::domain::{ClaimSpec, LoadedSpecs, SourceAccess, SourceKind, SourceSpec};
use anyhow::{anyhow, Result};

pub(crate) fn validate_source(row: &SourceSpec) -> Result<()> {
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
            if row.archive_status.is_some() {
                return Err(anyhow!("repo_path sources must not declare archive_status"));
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
            if row.archive_status.is_none() {
                return Err(anyhow!("manual acquisition sources must declare archive_status"));
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

pub(crate) fn validate_claim(row: &ClaimSpec) -> Result<()> {
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

pub(crate) fn validate_cross_references(loaded: &LoadedSpecs) -> Vec<String> {
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
