use super::models::FastqValidateReadsReadyReport;
use super::{acquisition, admission, collection, report};
use crate::proofs::inputs::ValidationContext;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_fastq_validate_reads_ready_report(
    context: &ValidationContext<'_>,
    output_path: &Path,
) -> Result<FastqValidateReadsReadyReport> {
    let sources = acquisition::acquire(context)?;
    let bindings = admission::admit(sources)?;
    let rows = collection::collect(context, bindings)?;
    Ok(report::summarize(context.repo_root, output_path, rows))
}
