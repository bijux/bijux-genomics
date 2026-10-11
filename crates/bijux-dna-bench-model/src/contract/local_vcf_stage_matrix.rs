//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted VCF stage matrices and their reports.

use std::collections::BTreeSet;

use crate::{
    model::{
        LocalVcfStageMatrixConfig, LocalVcfStageMatrixReport, ValidateVcfStageMatrixReport,
        VcfStageMatrixRow,
    },
    BenchError,
};

use super::{
    LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1,
    VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1,
};

/// Validate the authored local VCF stage-matrix declaration.
///
/// # Errors
/// Returns an error when schema identity, stage identity, or a row contract is invalid.
pub fn validate_local_vcf_stage_matrix_config(
    config: &LocalVcfStageMatrixConfig,
) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1 {
        return invalid("unsupported local VCF stage-matrix schema");
    }
    validate_rows(&config.rows).map(|_| ())
}

/// Validate the report emitted while rendering the local VCF stage matrix.
///
/// # Errors
/// Returns an error when schema identity or aggregate counts do not close.
pub fn validate_local_vcf_stage_matrix_report(
    report: &LocalVcfStageMatrixReport,
) -> Result<(), BenchError> {
    if report.schema_version != LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1 {
        return invalid("unsupported local VCF stage-matrix report schema");
    }
    require_text(&report.config_path, "config_path")?;
    let counts = validate_rows(&report.rows)?;
    if report.row_count != report.rows.len()
        || report.stage_count != counts.stage_count
        || report.supported_stage_count + report.planned_stage_count != report.stage_count
    {
        return invalid("local VCF stage-matrix report aggregates do not close");
    }
    Ok(())
}

/// Validate the report emitted after checking a persisted local VCF stage matrix.
///
/// # Errors
/// Returns an error when schema, domain, row, or tool-count evidence is inconsistent.
pub fn validate_vcf_stage_matrix_validation_report(
    report: &ValidateVcfStageMatrixReport,
) -> Result<(), BenchError> {
    if report.schema_version != VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1 {
        return invalid("unsupported VCF stage-matrix validation schema");
    }
    if report.domain != "vcf" {
        return invalid("VCF stage-matrix validation report has a foreign domain");
    }
    require_text(&report.matrix_path, "matrix_path")?;
    let counts = validate_rows(&report.rows)?;
    if report.row_count != report.rows.len() || report.stage_count != counts.stage_count {
        return invalid("VCF stage-matrix validation aggregates do not close");
    }
    if report.required_tool_count < counts.tool_count
        || report.registry_tool_count < counts.tool_count
    {
        return invalid("VCF stage-matrix validation tool aggregates omit selected tools");
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct MatrixCounts {
    stage_count: usize,
    tool_count: usize,
}

fn validate_rows(rows: &[VcfStageMatrixRow]) -> Result<MatrixCounts, BenchError> {
    if rows.is_empty() {
        return invalid("VCF stage matrix has no rows");
    }
    let mut stages = BTreeSet::new();
    let mut tools = BTreeSet::new();
    for row in rows {
        for (value, field) in [
            (row.stage_id.as_str(), "stage_id"),
            (row.tool_id.as_str(), "tool_id"),
            (row.corpus_id.as_str(), "corpus_id"),
            (row.asset_profile_id.as_str(), "asset_profile_id"),
            (row.adapter_id.as_str(), "adapter_id"),
            (row.parser_id.as_str(), "parser_id"),
        ] {
            require_text(value, field)?;
        }
        if !stages.insert(row.stage_id.as_str()) {
            return invalid("VCF stage matrix repeats a stage identity");
        }
        tools.insert(row.tool_id.as_str());
        if row.expected_outputs.is_empty()
            || row.expected_outputs.iter().any(|value| value.trim().is_empty())
            || row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                != row.expected_outputs.len()
        {
            return invalid("VCF stage matrix has invalid expected output identities");
        }
    }
    Ok(MatrixCounts { stage_count: stages.len(), tool_count: tools.len() })
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(&format!("VCF stage matrix requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.to_string()))
}
