use super::outputs::OutputExpectation;
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{VcfStageCatalogRow, VcfStageMatrixRow};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use bijux_dna_domain_vcf::{contracts::stage_metrics_contract, VcfDomainStage};
use std::collections::BTreeMap;

/// Project expected outputs, metrics and sections from admitted bindings and acquired adapters.
///
/// # Errors
/// Rejects unavailable or conflicting adapter observations and governed binding drift.
pub fn collect_vcf_expected_benchmark_result_rows(
    observations: &VcfAdapterObservations<'_>,
) -> Result<Vec<VcfExpectedBenchmarkResultRow>> {
    let inputs = observations.inputs();
    let benchmark_ready_rows = crate::evidence::serving::collect_vcf_tool_serving_map_rows(inputs)?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .collect::<Vec<_>>();
    let matrix_by_pair = inputs
        .bindings()
        .iter()
        .map(|row| ((row.stage_id.clone(), row.tool_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let catalog_by_stage =
        inputs.catalog().iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let adapter_rows = super::outputs::collect_output_expectations(observations)?;
    let expected_row_count = benchmark_ready_rows.len();

    let mut rows = Vec::with_capacity(benchmark_ready_rows.len());
    for serving_row in benchmark_ready_rows {
        let pair_key = (serving_row.stage_id.clone(), serving_row.tool_id.clone());
        let matrix_row = matrix_by_pair.get(&pair_key).ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing matrix coverage for `{}` / `{}`",
                serving_row.stage_id,
                serving_row.tool_id
            )
        })?;
        let catalog_row = catalog_by_stage.get(serving_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing stage catalog coverage for `{}`",
                serving_row.stage_id
            )
        })?;
        let stage = VcfDomainStage::try_from(serving_row.stage_id.as_str())
            .map_err(|error| anyhow!("unknown VCF stage `{}`: {error}", serving_row.stage_id))?;
        let expected_metrics = stage_metrics_contract(stage)
            .required_metrics
            .iter()
            .map(|metric| (*metric).to_string())
            .collect::<Vec<_>>();
        if expected_metrics.is_empty() {
            return Err(anyhow!(
                "VCF expected-result table stage `{}` is missing required metrics",
                serving_row.stage_id
            ));
        }
        let expected_outputs = expected_outputs_for_binding(
            &serving_row.tool_id,
            &serving_row.stage_id,
            matrix_row,
            &adapter_rows,
        )?;
        let report_section = report_section_for_stage(catalog_row)?;

        rows.push(VcfExpectedBenchmarkResultRow {
            domain: "vcf".to_string(),
            stage_id: serving_row.stage_id,
            tool_id: serving_row.tool_id,
            corpus_id: matrix_row.corpus_id.clone(),
            asset_profile_id: matrix_row.asset_profile_id.clone(),
            expected_outputs,
            expected_metrics,
            report_section,
        });
    }

    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });
    super::contract::ensure_vcf_expected_benchmark_result_contract(&rows, expected_row_count)?;
    Ok(rows)
}

fn expected_outputs_for_binding(
    tool_id: &str,
    stage_id: &str,
    matrix_row: &VcfStageMatrixRow,
    adapter_rows: &BTreeMap<(String, String), OutputExpectation>,
) -> Result<Vec<String>> {
    let adapter_row = adapter_rows
        .get(&(stage_id.to_string(), tool_id.to_string()))
        .ok_or_else(|| {
            anyhow!(
                "VCF expected-result table is missing adapter coverage for `{stage_id}` / `{tool_id}`"
            )
        })?;
    let expected_outputs = adapter_row.stage_output_ids.clone();
    if expected_outputs != matrix_row.expected_outputs {
        return Err(anyhow!(
            "VCF expected-result table stage output drifted for `{stage_id}` / `{tool_id}`"
        ));
    }
    if expected_outputs.is_empty() {
        return Err(anyhow!(
            "VCF expected-result table row `{stage_id}` / `{tool_id}` is missing expected outputs"
        ));
    }
    Ok(expected_outputs)
}

fn report_section_for_stage(catalog_row: &VcfStageCatalogRow) -> Result<String> {
    if catalog_row.benchmark_category.trim().is_empty() {
        return Err(anyhow!(
            "VCF expected-result table stage `{}` is missing benchmark_category",
            catalog_row.stage_id
        ));
    }
    Ok(catalog_row.benchmark_category.clone())
}
