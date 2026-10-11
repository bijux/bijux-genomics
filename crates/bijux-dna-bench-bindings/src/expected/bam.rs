use super::{
    index::OutputIndex,
    projection::{build_bam_expected_result_row, ProjectionContext},
    ExpectedBenchmarkResultRow,
};
use crate::observations::CorpusAssignment;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(super) fn collect(
    context: &ProjectionContext<'_, '_>,
    bam_corpus_rows: Vec<CorpusAssignment>,
    bam_readiness_kinds: &BTreeMap<String, String>,
    bam_contract_by_binding: &OutputIndex,
) -> Result<Vec<ExpectedBenchmarkResultRow>> {
    let mut rows = Vec::new();
    for row in bam_corpus_rows.into_iter().filter(|row| row.benchmark_status == "benchmark_ready") {
        let readiness_kind = bam_readiness_kinds
            .get(&row.stage_id)
            .cloned()
            .ok_or_else(|| anyhow!("missing BAM stage readiness kind for `{}`", row.stage_id))?;
        let contract_row = bam_contract_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing BAM adapter output contract row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        rows.push(build_bam_expected_result_row(
            &row.stage_id,
            &row.tool_id,
            &readiness_kind,
            row.corpus_family_id.as_deref().ok_or_else(|| {
                anyhow!(
                    "BAM benchmark-ready row `{}` / `{}` is missing corpus_family_id",
                    row.stage_id,
                    row.tool_id
                )
            })?,
            row.fixture_id.as_deref().ok_or_else(|| {
                anyhow!(
                    "BAM benchmark-ready row `{}` / `{}` is missing fixture_id",
                    row.stage_id,
                    row.tool_id
                )
            })?,
            context,
            contract_row,
        )?);
    }

    Ok(rows)
}
