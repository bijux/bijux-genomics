use super::models::FastqFilterStagesReadyReport;
use super::{acquisition, admission, collection, report};
use crate::proofs::inputs::SchemaContext;
use anyhow::Result;
use std::path::Path;
pub(super) fn build_fastq_filter_stages_ready_report(
    context: &SchemaContext<'_>,
    output_path: &Path,
) -> Result<FastqFilterStagesReadyReport> {
    let sources = acquisition::acquire(context)?;
    let bindings = admission::admit(sources)?;
    let rows = collection::collect(context, bindings)?;
    Ok(report::summarize(context.repo_root, output_path, rows))
}
