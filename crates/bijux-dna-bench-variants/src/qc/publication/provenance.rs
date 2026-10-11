//! Classify internal execution separately from compatible tool-report formatting.
use crate::qc::admission::report_format::QcReportFormat;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct QcExecutionProvenance {
    schema_version: &'static str,
    executor: &'static str,
    requested_report_format: &'static str,
    external_tool_executed: bool,
    report_projection: &'static str,
}

pub(crate) fn publish_provenance(root: &Path, format: QcReportFormat) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        &root.join("execution-provenance.json"),
        &QcExecutionProvenance {
            schema_version: "bijux.bench.qc_execution_provenance.v1",
            executor: "bijux-dna-stages-vcf::pipeline::run_qc_stage",
            requested_report_format: format.tool(),
            external_tool_executed: false,
            report_projection: "internal_observations_to_tool_report_format",
        },
    )?;
    Ok(())
}
