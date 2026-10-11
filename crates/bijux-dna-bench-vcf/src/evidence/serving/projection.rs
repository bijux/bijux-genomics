use super::VcfToolServingMapRow;
use crate::inputs::VcfAdapterInputs;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{VcfStageCatalogRow, VcfStageMatrixRow};
use std::collections::BTreeMap;

/// Derive serving observations from a single admitted catalog and binding snapshot.
///
/// # Errors
/// Rejects missing catalog coverage, parser/asset contract errors and governed binding drift.
pub fn collect_vcf_tool_serving_map_rows(
    inputs: &VcfAdapterInputs,
) -> Result<Vec<VcfToolServingMapRow>> {
    let catalog_rows = inputs.catalog().to_vec();
    let catalog_by_stage =
        catalog_rows.into_iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let matrix_rows = inputs.bindings().to_vec();

    let mut rows = Vec::with_capacity(matrix_rows.len());
    for matrix_row in matrix_rows {
        let catalog_row = catalog_by_stage.get(matrix_row.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF tool-serving map is missing stage catalog coverage for `{}`",
                matrix_row.stage_id
            )
        })?;
        rows.push(build_tool_serving_row(&matrix_row, catalog_row)?);
    }

    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.stage_id.cmp(&right.stage_id))
    });
    super::contract::ensure_vcf_tool_serving_map_contract(&rows, inputs.bindings().len())?;
    Ok(rows)
}

fn build_tool_serving_row(
    matrix_row: &VcfStageMatrixRow,
    catalog_row: &VcfStageCatalogRow,
) -> Result<VcfToolServingMapRow> {
    let support_status = catalog_row.support_status.clone();
    let adapter_status = adapter_status_label(&support_status, &matrix_row.adapter_id);
    let parser_status = parser_status_label(&matrix_row.parser_id)?;
    let corpus_status = corpus_status_label(&matrix_row.corpus_id);
    let asset_status =
        asset_status_label(&matrix_row.asset_profile_id, &catalog_row.required_assets)?;
    let benchmark_status = benchmark_status_label(&support_status);

    Ok(VcfToolServingMapRow {
        tool_id: matrix_row.tool_id.clone(),
        stage_id: matrix_row.stage_id.clone(),
        support_status,
        adapter_status,
        parser_status,
        corpus_status,
        asset_status,
        benchmark_status,
    })
}

fn adapter_status_label(support_status: &str, adapter_id: &str) -> String {
    if adapter_id.trim().is_empty() {
        "missing".to_string()
    } else if support_status == "supported" {
        "runnable".to_string()
    } else {
        "declared_only".to_string()
    }
}

fn parser_status_label(parser_id: &str) -> Result<String> {
    let label = match parser_id {
        "vcf.parser.vcf_output"
        | "vcf.parser.call_summary"
        | "vcf.parser.qc_report"
        | "vcf.parser.report_json"
        | "vcf.parser.stats_report" => "parse_normalized",
        other => {
            return Err(anyhow!(
                "VCF tool-serving map encountered unknown parser contract `{other}`"
            ));
        }
    };
    Ok(label.to_string())
}

fn corpus_status_label(corpus_id: &str) -> String {
    format!("fixture:{corpus_id}")
}

fn asset_status_label(asset_profile_id: &str, required_assets: &[String]) -> Result<String> {
    if asset_profile_id.trim().is_empty() {
        return Err(anyhow!("VCF tool-serving map encountered an empty asset profile id"));
    }
    if required_assets.is_empty() {
        Ok("not_required".to_string())
    } else {
        Ok("assigned".to_string())
    }
}

fn benchmark_status_label(support_status: &str) -> String {
    match support_status {
        "supported" => "benchmark_ready",
        _ => "not_benchmark_ready",
    }
    .to_string()
}
