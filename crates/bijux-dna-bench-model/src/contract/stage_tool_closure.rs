//! Admission rules for the persisted stage/tool closure report.

use std::collections::BTreeSet;

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkStageToolClosureDisposition, BenchmarkStageToolClosureReport,
    BenchmarkStageToolProtocolDisposition,
};
use crate::BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1;

/// Validate identities, dispositions, and aggregate closure.
///
/// # Errors
/// Returns an error when sources, pair identities, dispositions, or summary values drift.
pub fn validate_benchmark_stage_tool_closure(
    report: &BenchmarkStageToolClosureReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1 {
        return Err(invalid("stage/tool closure schema mismatch"));
    }
    let mut source_paths = BTreeSet::new();
    if report.sources.is_empty()
        || report.sources.iter().any(|source| {
            source.path.trim().is_empty()
                || !source_paths.insert(source.path.as_str())
                || source.sha256.len() != 64
                || !source
                    .sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        || report.sources.windows(2).any(|pair| pair[0].path >= pair[1].path)
    {
        return Err(invalid("stage/tool closure sources are invalid"));
    }

    let mut pair_ids = BTreeSet::new();
    let mut tools = BTreeSet::new();
    let mut stages = BTreeSet::new();
    let mut aligned = 0_usize;
    let mut benchmark = 0_usize;
    let mut external = 0_usize;
    for row in &report.rows {
        if row.domain.trim().is_empty()
            || row.stage_id.strip_prefix(&format!("{}.", row.domain)).is_none()
            || row.tool_id.trim().is_empty()
            || row.product_support != "supported"
            || row.artifact_delivery.trim().is_empty()
            || !pair_ids.insert((row.stage_id.as_str(), row.tool_id.as_str()))
        {
            return Err(invalid("stage/tool closure row identity is invalid"));
        }
        tools.insert(row.tool_id.as_str());
        stages.insert(row.stage_id.as_str());
        match (row.domain_declared, row.closure) {
            (true, BenchmarkStageToolClosureDisposition::Aligned) => aligned += 1,
            (false, BenchmarkStageToolClosureDisposition::MissingDomainBinding) => {}
            _ => return Err(invalid("stage/tool closure disposition is inconsistent")),
        }
        match (&row.benchmark_run_id, row.protocol_disposition) {
            (Some(run_id), BenchmarkStageToolProtocolDisposition::BenchmarkRequired)
                if run_id.starts_with("run-") =>
            {
                benchmark += 1;
            }
            (None, BenchmarkStageToolProtocolDisposition::QualificationOnly) => {}
            _ => return Err(invalid("stage/tool protocol disposition is inconsistent")),
        }
        if row.artifact_delivery == "external_artifact" {
            external += 1;
        }
    }

    let row_order = report.rows.iter().map(|row| (&row.stage_id, &row.tool_id)).collect::<Vec<_>>();
    let mut sorted_row_order = row_order.clone();
    sorted_row_order.sort_unstable();
    if row_order != sorted_row_order {
        return Err(invalid("stage/tool closure rows are not ordered"));
    }

    let summary = &report.summary;
    if summary.supported_tool_count < tools.len()
        || summary.stage_count < stages.len()
        || summary.catalog_pair_count != report.rows.len()
        || summary.domain_pair_count < aligned
        || summary.aligned_pair_count != aligned
        || summary.drift_pair_count != report.rows.len() - aligned
        || summary.benchmark_pair_count != benchmark
        || summary.qualification_only_pair_count != report.rows.len() - benchmark
        || summary.external_artifact_pair_count != external
    {
        return Err(invalid("stage/tool closure summary drifted"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
