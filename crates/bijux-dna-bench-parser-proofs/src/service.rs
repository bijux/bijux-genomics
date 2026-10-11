use crate::{
    catalog::{family_probe_specs, FamilyProbeSpec},
    materialization::materialize_family_proof,
    model::{RealOutputParserSmokeReport, RealOutputParserSmokeRow},
    ports::ProbeMaterializer,
    reports::{parse_proof, parse_surface_label},
    values::{path_relative_to_repo, top_level_keys},
};
use anyhow::{anyhow, Result};
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};
const REAL_OUTPUT_PARSER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.real_output_parser_smoke.v1";

/// Build governed parser evidence through explicitly composed producers.
///
/// # Errors
/// Reject missing declarations, materialization failures and invalid proof documents.
pub fn build_report(
    repo_root: &Path,
    output_path: &Path,
    probe_root: &Path,
    active_families: Vec<(String, String)>,
    producer: &dyn ProbeMaterializer,
) -> Result<RealOutputParserSmokeReport> {
    let spec_map = family_probe_specs()
        .into_iter()
        .map(|spec| (spec.family_id, spec))
        .collect::<BTreeMap<_, _>>();

    let selected = crate::admission::select_families(active_families, &spec_map)?;
    let mut rows = Vec::with_capacity(selected.len());
    for (family_id, family_summary, spec) in selected {
        rows.push(materialize_row(
            repo_root,
            probe_root,
            family_id,
            family_summary,
            spec,
            producer,
        )?);
    }

    rows.sort_by(|left, right| left.family_id.cmp(&right.family_id));
    let passed_family_count = rows.iter().filter(|row| row.passed).count();
    let failed_family_count = rows.len().saturating_sub(passed_family_count);
    let mut parser_surface_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *parser_surface_counts.entry(row.parser_surface.clone()).or_default() += 1;
    }

    Ok(RealOutputParserSmokeReport {
        schema_version: REAL_OUTPUT_PARSER_SMOKE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        probe_root: path_relative_to_repo(repo_root, probe_root),
        family_count: rows.len(),
        passed_family_count,
        failed_family_count,
        parser_surface_counts,
        rows,
    })
}

/// Build governed parser evidence through explicitly composed producers.
///
/// # Errors
/// Reject missing declarations, materialization failures and invalid proof documents.
pub fn materialize_row(
    repo_root: &Path,
    probe_root: &Path,
    family_id: String,
    family_summary: String,
    spec: FamilyProbeSpec,
    producer: &dyn ProbeMaterializer,
) -> Result<RealOutputParserSmokeRow> {
    if family_id != spec.family_id {
        return Err(anyhow!("parser proof family does not match its governed declaration"));
    }
    crate::reports::require_available(spec.parse_kind)?;
    let materialized = materialize_family_proof(repo_root, probe_root, &spec, producer)?;
    let parsed = parse_proof(&materialized.proof_path, spec.parse_kind, spec.snapshot_keys)?;
    Ok(RealOutputParserSmokeRow {
        family_id,
        family_summary,
        representative_stage_id: spec.representative_stage_id.to_string(),
        representative_tool_id: materialized.observed_tool_id,
        proof_path: path_relative_to_repo(repo_root, &materialized.proof_path),
        parser_surface: parse_surface_label(spec.parse_kind).to_string(),
        parsed_schema_version: parsed
            .0
            .get("schema_version")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        parsed_top_level_key_count: top_level_keys(&parsed.0).len(),
        parsed_top_level_keys: top_level_keys(&parsed.0),
        normalized_snapshot: parsed.1,
        passed: true,
        reason: format!(
            "validated {} output through {}",
            spec.family_id,
            parse_surface_label(spec.parse_kind)
        ),
    })
}
