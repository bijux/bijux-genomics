//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted benchmark stage/tool scope views.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    model::{
        AllDomainActiveStageToolMatrixReport, AllDomainActiveStageToolMatrixRow,
        AllDomainRetainedToolRow, AllDomainRetainedToolsReport, StageRegistryExtraPairsReport,
    },
    BenchError,
};

use super::{
    ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1, ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1,
    STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
};

const BENCHMARK_READY: &str = "benchmark_ready";
const NOT_BENCHMARK_READY: &str = "not_benchmark_ready";
const LICENSED_BINDINGS: [(&str, &str, &str); 8] = [
    ("bam", "bam.damage", "ngsbriggs"),
    ("bam", "bam.kinship", "angsd"),
    ("bam", "bam.sex", "angsd"),
    ("fastq", "fastq.remove_duplicates", "fastuniq"),
    ("vcf", "vcf.call_gl", "angsd"),
    ("vcf", "vcf.call_pseudohaploid", "angsd"),
    ("vcf", "vcf.imputation_metrics", "impute5"),
    ("vcf", "vcf.impute", "impute5"),
];

/// Validate the persisted all-domain active stage/tool matrix.
///
/// # Errors
/// Returns an error when identity, licensed binding retention, or aggregate closure drifts.
pub fn validate_all_domain_active_stage_tool_matrix(
    report: &AllDomainActiveStageToolMatrixReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1 {
        return invalid("unsupported all-domain active stage/tool matrix schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.rows.is_empty() || report.row_count != report.rows.len() {
        return invalid("active stage/tool matrix row aggregate does not close");
    }
    if !report.rows.windows(2).all(|pair| row_key(&pair[0]) < row_key(&pair[1])) {
        return invalid("active stage/tool matrix rows are not in canonical order");
    }

    let mut keys = BTreeSet::new();
    let mut stages = BTreeSet::new();
    let mut tools = BTreeSet::new();
    let mut domain_counts = BTreeMap::new();
    let mut status_counts = BTreeMap::new();
    for row in &report.rows {
        validate_active_row(row)?;
        let key = row_key(row);
        if !keys.insert(key) {
            return invalid("active stage/tool matrix repeats a binding identity");
        }
        stages.insert((row.domain.as_str(), row.stage_id.as_str()));
        tools.insert(row.tool_id.as_str());
        increment(&mut domain_counts, &row.domain);
        increment(&mut status_counts, &row.status);
    }
    for required in LICENSED_BINDINGS {
        if !keys.contains(&required) {
            return invalid(format!(
                "active stage/tool matrix omits supported licensed binding {}/{}/{}",
                required.0, required.1, required.2
            ));
        }
    }
    if report.stage_count != stages.len()
        || report.tool_count != tools.len()
        || report.domain_counts != domain_counts
        || report.status_counts != status_counts
    {
        return invalid("active stage/tool matrix aggregates do not close");
    }
    Ok(())
}

fn validate_active_row(row: &AllDomainActiveStageToolMatrixRow) -> Result<(), BenchError> {
    for (value, field) in [
        (row.domain.as_str(), "rows[].domain"),
        (row.stage_id.as_str(), "rows[].stage_id"),
        (row.tool_id.as_str(), "rows[].tool_id"),
        (row.corpus_id.as_str(), "rows[].corpus_id"),
        (row.asset_profile_id.as_str(), "rows[].asset_profile_id"),
        (row.adapter_id.as_str(), "rows[].adapter_id"),
        (row.parser_id.as_str(), "rows[].parser_id"),
        (row.schema_id.as_str(), "rows[].schema_id"),
    ] {
        require_text(value, field)?;
    }
    if row.status != BENCHMARK_READY {
        return invalid("active stage/tool matrix contains a non-ready binding");
    }
    Ok(())
}

/// Validate the persisted all-domain retained-tool inventory.
///
/// # Errors
/// Returns an error when tool coverage, licensed pairs, or aggregate closure drifts.
pub fn validate_all_domain_retained_tools(
    report: &AllDomainRetainedToolsReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1 {
        return invalid("unsupported all-domain retained-tool schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.rows.is_empty()
        || report.row_count != report.rows.len()
        || report.active_matrix_tool_count != report.rows.len()
        || !report.rows.windows(2).all(|pair| pair[0].tool_id < pair[1].tool_id)
    {
        return invalid("retained-tool roster or row aggregate is invalid");
    }

    let mut tool_ids = BTreeSet::new();
    let mut domain_tool_counts = BTreeMap::new();
    let mut benchmark_ready_tools = 0;
    let mut mixed_status_tools = 0;
    let mut not_ready_only_tools = 0;
    for row in &report.rows {
        validate_retained_tool(row)?;
        if !tool_ids.insert(row.tool_id.as_str()) {
            return invalid("retained-tool inventory repeats a tool identity");
        }
        benchmark_ready_tools += usize::from(row.benchmark_ready_binding_count > 0);
        mixed_status_tools += usize::from(row.benchmark_statuses.len() > 1);
        not_ready_only_tools += usize::from(row.benchmark_ready_binding_count == 0);
        for domain in &row.domains {
            increment(&mut domain_tool_counts, domain);
        }
    }
    for (domain, stage_id, tool_id) in LICENSED_BINDINGS {
        let row = report.rows.iter().find(|row| row.tool_id == tool_id).ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "retained-tool inventory omits supported licensed tool {tool_id}"
            ))
        })?;
        if !row.domains.iter().any(|candidate| candidate == domain)
            || !row.active_stage_ids.iter().any(|candidate| candidate == stage_id)
        {
            return invalid(format!(
                "retained-tool inventory omits supported licensed binding {domain}/{stage_id}/{tool_id}"
            ));
        }
    }
    if report.benchmark_ready_tool_count != benchmark_ready_tools
        || report.mixed_status_tool_count != mixed_status_tools
        || report.not_benchmark_ready_only_tool_count != not_ready_only_tools
        || report.domain_tool_counts != domain_tool_counts
    {
        return invalid("retained-tool top-level aggregates do not close");
    }
    Ok(())
}

fn validate_retained_tool(row: &AllDomainRetainedToolRow) -> Result<(), BenchError> {
    require_text(&row.tool_id, "rows[].tool_id")?;
    validate_canonical_text(&row.domains, false, "rows[].domains")?;
    validate_canonical_text(&row.benchmark_statuses, false, "rows[].benchmark_statuses")?;
    validate_canonical_text(&row.active_stage_ids, false, "rows[].active_stage_ids")?;
    validate_canonical_text(
        &row.benchmark_ready_stage_ids,
        true,
        "rows[].benchmark_ready_stage_ids",
    )?;
    if row
        .benchmark_statuses
        .iter()
        .any(|status| !matches!(status.as_str(), BENCHMARK_READY | NOT_BENCHMARK_READY))
    {
        return invalid("retained-tool inventory contains an unsupported readiness status");
    }
    if row.active_stage_count != row.active_stage_ids.len()
        || row.benchmark_ready_stage_count != row.benchmark_ready_stage_ids.len()
        || row.active_binding_count < row.active_stage_count
        || row.benchmark_ready_binding_count > row.active_binding_count
        || row.benchmark_ready_stage_count > row.active_stage_count
        || row.benchmark_ready_stage_ids.iter().any(|stage| !row.active_stage_ids.contains(stage))
        || ((row.benchmark_ready_binding_count > 0)
            != row.benchmark_statuses.iter().any(|status| status == BENCHMARK_READY))
    {
        return invalid("retained-tool nested aggregates do not close");
    }
    Ok(())
}

/// Validate the persisted registry-only stage/tool pair report.
///
/// # Errors
/// Returns an error when pair identity, exception disposition, or aggregate closure drifts.
pub fn validate_stage_registry_extra_pairs(
    report: &StageRegistryExtraPairsReport,
) -> Result<(), BenchError> {
    if report.schema_version != STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1 {
        return invalid("unsupported stage-registry extra-pairs schema");
    }
    require_text(&report.output_path, "output_path")?;
    if report.extra_pair_count != report.rows.len()
        || !report.rows.windows(2).all(|pair| {
            (&pair[0].domain, &pair[0].stage_id, &pair[0].tool_id)
                < (&pair[1].domain, &pair[1].stage_id, &pair[1].tool_id)
        })
    {
        return invalid("stage-registry extra-pair roster or aggregate is invalid");
    }
    let mut domain_counts = BTreeMap::new();
    for row in &report.rows {
        for (value, field) in [
            (row.domain.as_str(), "rows[].domain"),
            (row.stage_id.as_str(), "rows[].stage_id"),
            (row.tool_id.as_str(), "rows[].tool_id"),
            (row.reason.as_str(), "rows[].reason"),
        ] {
            require_text(value, field)?;
        }
        if !matches!(
            row.contract_status.as_str(),
            "pair_missing_from_contract" | "tool_missing_contract"
        ) {
            return invalid("stage-registry extra pair has an unsupported contract status");
        }
        validate_unique_text(&row.registry_sources, true, "rows[].registry_sources")?;
        validate_unique_text(&row.registered_stage_ids, true, "rows[].registered_stage_ids")?;
        match row.documented_exception_status.as_str() {
            "none" if row.documented_exception_reason.is_empty() => {}
            "explicit_intent" if !row.documented_exception_reason.trim().is_empty() => {}
            _ => return invalid("stage-registry exception status and reason disagree"),
        }
        increment(&mut domain_counts, &row.domain);
    }
    let observed_ok =
        report.rows.iter().all(|row| row.documented_exception_status == "explicit_intent");
    if report.domain_counts != domain_counts || report.ok != observed_ok {
        return invalid("stage-registry extra-pair aggregates do not close");
    }
    Ok(())
}

fn row_key(row: &AllDomainActiveStageToolMatrixRow) -> (&str, &str, &str) {
    (&row.domain, &row.stage_id, &row.tool_id)
}

fn validate_canonical_text(
    values: &[String],
    allow_empty: bool,
    field: &str,
) -> Result<(), BenchError> {
    validate_unique_text(values, allow_empty, field)?;
    if !values.windows(2).all(|pair| pair[0] < pair[1]) {
        return invalid(format!("scope-view {field} is not in canonical order"));
    }
    Ok(())
}

fn validate_unique_text(
    values: &[String],
    allow_empty: bool,
    field: &str,
) -> Result<(), BenchError> {
    if (!allow_empty && values.is_empty())
        || values.iter().any(|value| value.trim().is_empty())
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
    {
        return invalid(format!("scope-view {field} contains invalid or duplicate values"));
    }
    Ok(())
}

fn increment(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("scope-view requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
