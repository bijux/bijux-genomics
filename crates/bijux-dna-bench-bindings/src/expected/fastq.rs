use super::{
    index::OutputIndex,
    projection::{
        build_fastq_asset_backed_expected_result_row, build_fastq_expected_result_row,
        ProjectionContext,
    },
    ExpectedBenchmarkResultRow,
};
use crate::observations::{AssignmentKind, CorpusAssignment};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
pub(super) fn collect(
    context: &ProjectionContext<'_, '_>,
    fastq_corpus_rows: Vec<CorpusAssignment>,
    fastq_readiness_kinds: &BTreeMap<String, String>,
    fastq_contract_by_binding: &OutputIndex,
) -> Result<Vec<ExpectedBenchmarkResultRow>> {
    let mut rows = Vec::new();
    for row in fastq_corpus_rows.into_iter().filter(|row| {
        row.benchmark_status == "benchmark_ready"
            && matches!(
                row.assignment_status,
                AssignmentKind::Assigned | AssignmentKind::AssetBacked
            )
    }) {
        let readiness_kind = fastq_readiness_kinds
            .get(&row.stage_id)
            .cloned()
            .ok_or_else(|| anyhow!("missing FASTQ stage readiness kind for `{}`", row.stage_id))?;
        let contract_row = fastq_contract_by_binding
            .get(&(row.stage_id.clone(), row.tool_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "missing FASTQ adapter output contract row for `{}` / `{}`",
                    row.stage_id,
                    row.tool_id
                )
            })?;
        match row.assignment_status {
            AssignmentKind::Assigned => {
                let fixture_id = row.fixture_id.clone().ok_or_else(|| {
                    anyhow!(
                        "FASTQ benchmark-ready row `{}` / `{}` is missing fixture_id",
                        row.stage_id,
                        row.tool_id
                    )
                })?;
                let corpus_family_id = row.corpus_family_id.clone().ok_or_else(|| {
                    anyhow!(
                        "FASTQ benchmark-ready row `{}` / `{}` is missing corpus_family_id",
                        row.stage_id,
                        row.tool_id
                    )
                })?;
                rows.push(build_fastq_expected_result_row(
                    &row.stage_id,
                    &row.tool_id,
                    &readiness_kind,
                    &corpus_family_id,
                    &fixture_id,
                    context,
                    contract_row,
                )?);
            }
            AssignmentKind::AssetBacked => {
                let benchmark_scope_id = row.benchmark_scope_id.clone().ok_or_else(|| {
                    anyhow!(
                        "FASTQ asset-backed row `{}` / `{}` is missing benchmark_scope_id",
                        row.stage_id,
                        row.tool_id
                    )
                })?;
                rows.push(build_fastq_asset_backed_expected_result_row(
                    context,
                    &row.stage_id,
                    &row.tool_id,
                    &readiness_kind,
                    &benchmark_scope_id,
                    contract_row,
                )?);
            }
            AssignmentKind::Excluded => {}
        }
    }

    Ok(rows)
}
