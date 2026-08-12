//! Validation for persisted local HPC result-collection simulation reports.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResultCollectionSimulationRow,
    BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
};

const COLLECTION_STATUSES: [&str; 5] =
    ["complete", "failed", "missing", "insufficient", "unavailable"];

/// Validate status distinctions, evidence lineage, exact counters, and behavior claims.
///
/// # Errors
/// Returns an error when persisted result-collection evidence is incomplete or inconsistent.
pub fn validate_benchmark_local_hpc_result_collection_simulation_report(
    report: &BenchmarkLocalHpcResultCollectionSimulationReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local HPC result-collection simulation schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.output_path, "output_path")?;
    require_text(&report.simulation_root, "simulation_root")?;
    require_text(&report.execution_resolver_path, "execution_resolver_path")?;
    if report.row_count != COLLECTION_STATUSES.len() || report.row_count != report.rows.len() {
        return invalid("result-collection simulation must persist exactly five status rows");
    }

    let expected_counts = BTreeMap::from([
        ("complete", report.complete_row_count),
        ("failed", report.failed_row_count),
        ("missing", report.missing_row_count),
        ("insufficient", report.insufficient_row_count),
        ("unavailable", report.unavailable_row_count),
    ]);
    if expected_counts.values().any(|count| *count != 1) {
        return invalid("result-collection simulation must persist one row per governed status");
    }

    let mut record_ids = BTreeSet::new();
    let mut job_ids = BTreeSet::new();
    let mut rows_by_status = BTreeMap::new();
    for row in &report.rows {
        validate_row(row)?;
        if !record_ids.insert(row.record_id.as_str()) {
            return invalid(format!("duplicate result-collection record `{}`", row.record_id));
        }
        if !job_ids.insert(row.job_id_local.as_str()) {
            return invalid(format!("duplicate local job `{}`", row.job_id_local));
        }
        if rows_by_status.insert(row.collection_status.as_str(), row).is_some() {
            return invalid(format!(
                "duplicate result-collection status `{}`",
                row.collection_status
            ));
        }
    }
    if rows_by_status.len() != COLLECTION_STATUSES.len()
        || COLLECTION_STATUSES.iter().any(|status| !rows_by_status.contains_key(status))
    {
        return invalid("result-collection rows must cover each governed status exactly once");
    }
    for (status, expected) in expected_counts {
        let observed = report.rows.iter().filter(|row| row.collection_status == status).count();
        if observed != expected {
            return invalid(format!("{status}_row_count drifted from persisted rows"));
        }
    }

    validate_behavior(report, &rows_by_status)
}

fn validate_row(row: &BenchmarkLocalHpcResultCollectionSimulationRow) -> Result<(), BenchError> {
    require_text(&row.record_id, "rows[].record_id")?;
    require_text(&row.source_surface, "rows[].source_surface")?;
    require_text(&row.job_id_local, "rows[].job_id_local")?;
    require_text(&row.domain, "rows[].domain")?;
    require_text(&row.stage_id, "rows[].stage_id")?;
    require_text(&row.tool_id, "rows[].tool_id")?;
    require_text(&row.evidence_path, "rows[].evidence_path")?;
    require_text(&row.detail, "rows[].detail")?;
    if row.present_output_count > row.declared_output_count {
        return invalid(format!(
            "row `{}` has more present outputs than declared outputs",
            row.record_id
        ));
    }
    match row.collection_status.as_str() {
        "complete" | "failed" => {
            require_result_row(row)?;
            require_optional_text(row.manifest_path.as_deref(), "rows[].manifest_path")?;
            require_absent_reason(row)?;
        }
        "missing" => {
            require_result_row(row)?;
            require_absent(row.manifest_path.as_deref(), "manifest_path", row)?;
            require_absent_reason(row)?;
        }
        "insufficient" => {
            require_result_row(row)?;
            require_optional_text(row.manifest_path.as_deref(), "rows[].manifest_path")?;
            require_optional_text(
                row.insufficient_data_reason.as_deref(),
                "rows[].insufficient_data_reason",
            )?;
            require_absent(row.unavailable_reason.as_deref(), "unavailable_reason", row)?;
        }
        "unavailable" => {
            if row.source_surface != "execution_resolver"
                || row.record_id != format!("unavailable:{}", row.job_id_local)
                || row.declared_output_count != 0
                || row.present_output_count != 0
            {
                return invalid(format!(
                    "unavailable row `{}` has invalid resolver lineage",
                    row.record_id
                ));
            }
            require_absent(row.manifest_path.as_deref(), "manifest_path", row)?;
            require_absent(
                row.insufficient_data_reason.as_deref(),
                "insufficient_data_reason",
                row,
            )?;
            require_optional_text(row.unavailable_reason.as_deref(), "rows[].unavailable_reason")?;
        }
        status => return invalid(format!("unsupported result-collection status `{status}`")),
    }
    Ok(())
}

fn validate_behavior(
    report: &BenchmarkLocalHpcResultCollectionSimulationReport,
    rows_by_status: &BTreeMap<&str, &BenchmarkLocalHpcResultCollectionSimulationRow>,
) -> Result<(), BenchError> {
    let behavior = &report.behavior;
    let identities = [
        ("complete", behavior.complete_result_job_id.as_str()),
        ("failed", behavior.failed_result_job_id.as_str()),
        ("missing", behavior.missing_result_job_id.as_str()),
        ("insufficient", behavior.insufficient_result_job_id.as_str()),
        ("unavailable", behavior.unavailable_result_job_id.as_str()),
    ];
    let mut job_ids = BTreeSet::new();
    for (status, job_id) in identities {
        require_text(job_id, "behavior result job id")?;
        if !job_ids.insert(job_id) {
            return invalid(format!("behavior reuses local job `{job_id}`"));
        }
        let row = rows_by_status
            .get(status)
            .ok_or_else(|| BenchError::InvalidPolicy(format!("missing `{status}` row")))?;
        if row.job_id_local != job_id {
            return invalid(format!("behavior `{status}` job drifted from persisted row"));
        }
    }
    if !behavior.complete_row_present
        || !behavior.failed_row_present
        || !behavior.missing_row_present
        || !behavior.insufficient_row_present
        || !behavior.unavailable_row_present
        || !behavior.failed_distinct_from_missing
        || !behavior.insufficient_distinct_from_unavailable
        || !behavior.proven
    {
        return invalid("behavior must prove all five status distinctions");
    }
    Ok(())
}

fn require_result_row(
    row: &BenchmarkLocalHpcResultCollectionSimulationRow,
) -> Result<(), BenchError> {
    if row.source_surface != "stage_result_manifest"
        || row.record_id != format!("result:{}", row.job_id_local)
    {
        return invalid(format!("result row `{}` has invalid manifest lineage", row.record_id));
    }
    Ok(())
}

fn require_absent_reason(
    row: &BenchmarkLocalHpcResultCollectionSimulationRow,
) -> Result<(), BenchError> {
    require_absent(row.insufficient_data_reason.as_deref(), "insufficient_data_reason", row)?;
    require_absent(row.unavailable_reason.as_deref(), "unavailable_reason", row)
}

fn require_absent(
    value: Option<&str>,
    field: &str,
    row: &BenchmarkLocalHpcResultCollectionSimulationRow,
) -> Result<(), BenchError> {
    if value.is_some() {
        return invalid(format!(
            "row `{}` cannot declare {field} for `{}` status",
            row.record_id, row.collection_status
        ));
    }
    Ok(())
}

fn require_optional_text<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, BenchError> {
    let value = value.ok_or_else(|| {
        BenchError::InvalidPolicy(format!(
            "local HPC result-collection simulation requires {field}"
        ))
    })?;
    require_text(value, field)?;
    Ok(value)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!(
            "local HPC result-collection simulation requires non-empty {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
