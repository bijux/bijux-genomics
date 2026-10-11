//! Authored science document acquisition with version, identity and reference admission.
use super::{list_yaml_files, read_utf8};
use crate::domain::LoadedSpecs;
use crate::errors::validation_error;
use crate::schema::authored::{validate_claim, validate_cross_references, validate_source};
use crate::schema::{
    SchemaVersion, ASSUMPTION_SCHEMA_VERSION, BINDING_SCHEMA_VERSION, CLAIM_SCHEMA_VERSION,
    DECISION_SCHEMA_VERSION, EVIDENCE_SCHEMA_VERSION, REASONING_SCHEMA_VERSION,
    RELEASE_SCHEMA_VERSION, SOURCE_SCHEMA_VERSION,
};
use anyhow::{anyhow, Result};
use std::{collections::BTreeMap, path::Path};

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
