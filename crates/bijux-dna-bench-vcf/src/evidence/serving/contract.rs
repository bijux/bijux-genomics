use super::VcfToolServingMapRow;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;

pub(super) fn ensure_vcf_tool_serving_map_contract(
    rows: &[VcfToolServingMapRow],
    expected_row_count: usize,
) -> Result<()> {
    let expected_rows = [
        ("bcftools", "vcf.call", "assigned"),
        ("bcftools", "vcf.postprocess", "assigned"),
        ("bcftools", "vcf.prepare_reference_panel", "assigned"),
        ("bcftools", "vcf.qc", "assigned"),
        ("plink", "vcf.qc", "assigned"),
        ("plink2", "vcf.qc", "assigned"),
        ("plink2", "vcf.population_structure", "assigned"),
        ("plink2", "vcf.pca", "assigned"),
        ("eigensoft", "vcf.pca", "assigned"),
        ("ibdne", "vcf.demography", "not_required"),
    ];

    for (tool_id, stage_id, asset_status) in expected_rows {
        let row = rows
            .iter()
            .find(|row| row.tool_id == tool_id && row.stage_id == stage_id)
            .ok_or_else(|| anyhow!("VCF tool-serving map is missing `{stage_id}` / `{tool_id}`"))?;
        if row.support_status != "supported"
            || row.adapter_status != "runnable"
            || row.parser_status != "parse_normalized"
            || row.corpus_status != "fixture:vcf_production_regression"
            || row.asset_status != asset_status
            || row.benchmark_status != "benchmark_ready"
        {
            return Err(anyhow!(
                "VCF tool-serving row `{stage_id}` / `{tool_id}` drifted from its governed readiness contract"
            ));
        }
    }

    if rows.len() != expected_row_count {
        return Err(anyhow!(
            "VCF tool-serving map must contain exactly one row per matrix row (expected {}, found {})",
            expected_row_count,
            rows.len()
        ));
    }

    let mut seen_bindings = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen_bindings.insert((row.stage_id.as_str(), row.tool_id.as_str())) {
            return Err(anyhow!(
                "VCF tool-serving map contains duplicate row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    Ok(())
}
