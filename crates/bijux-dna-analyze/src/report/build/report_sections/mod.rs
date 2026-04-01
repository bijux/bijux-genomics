mod pipeline_overview;
mod key_findings;

pub(super) use key_findings::key_findings_section;
pub(super) use pipeline_overview::pipeline_overview_section;

pub(super) fn data_contract_validation_section(
    completeness: &bijux_dna_runtime::ReportCompletenessV1,
) -> serde_json::Value {
    serde_json::json!({
        "status": completeness.status,
        "missing_metrics": completeness.missing_metrics,
        "missing_reports": completeness.missing_reports,
    })
}
