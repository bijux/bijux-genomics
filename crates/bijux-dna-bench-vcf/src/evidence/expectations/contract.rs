use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::collections::BTreeSet;
pub(crate) fn ensure_vcf_expected_benchmark_result_contract(
    rows: &[VcfExpectedBenchmarkResultRow],
    expected_row_count: usize,
) -> Result<()> {
    let unique_rows = rows
        .iter()
        .map(|row| (&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id))
        .collect::<BTreeSet<_>>();
    if unique_rows.len() != rows.len() {
        return Err(anyhow!(
            "VCF expected-result table must keep one row per benchmark-ready stage-tool-corpus-asset binding"
        ));
    }
    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "VCF expected-result table must retain every benchmark-ready row, expected {expected_row_count}, found {}",
            rows.len(),
        ));
    }
    if rows.iter().any(|row| row.domain != "vcf") {
        return Err(anyhow!(
            "VCF expected-result table must keep the VCF domain label on every row"
        ));
    }
    for &(
        stage_id,
        tool_id,
        corpus_id,
        asset_profile_id,
        expected_outputs,
        expected_metrics,
        report_section,
    ) in super::governed::EXPECTED_BINDINGS
    {
        let row =
            rows.iter().find(|row| row.stage_id == stage_id && row.tool_id == tool_id).ok_or_else(
                || anyhow!("VCF expected-result table is missing `{stage_id}` / `{tool_id}`"),
            )?;
        if row.corpus_id != corpus_id
            || row.asset_profile_id != asset_profile_id
            || !row.expected_outputs.iter().any(|candidate| candidate == expected_outputs)
            || !row.expected_metrics.iter().any(|candidate| candidate == expected_metrics)
            || row.report_section != report_section
        {
            return Err(anyhow!(
                "VCF expected-result table row `{stage_id}` / `{tool_id}` drifted from the governed corpus, asset, output, metrics, or section contract"
            ));
        }
    }
    Ok(())
}
