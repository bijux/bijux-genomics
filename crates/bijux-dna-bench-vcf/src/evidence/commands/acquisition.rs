use super::projection;
use crate::adapters::observations::VcfAdapterObservations;
use anyhow::{anyhow, Result};
use bijux_dna_bench_vcf_evidence::{
    commands::admission::render_script_commands, model::commands::VcfRenderedCommandRow,
};
use std::collections::BTreeMap;
/// Acquire and publish compatible adapter reports, then project one command authority per pair.
///
/// # Errors
/// Returns family acquisition, publication or duplicate command authority errors.
pub fn collect_vcf_adapter_command_row_map(
    observations: &VcfAdapterObservations<'_>,
) -> Result<BTreeMap<(String, String), VcfRenderedCommandRow>> {
    let report_root =
        observations.repo_root().join("artifacts/bench-readiness/vcf-rendered-command-rows");

    let mut rows = Vec::new();

    projection::append_bcftools(
        &mut rows,
        observations.publish_bcftools(&report_root.join("bcftools.adapter.json"))?.rows,
    );
    projection::append_angsd(
        &mut rows,
        observations.publish_angsd(&report_root.join("angsd.adapter.json"))?.rows,
    );
    projection::append_eigensoft(
        &mut rows,
        observations.publish_eigensoft(&report_root.join("eigensoft.adapter.json"))?.rows,
    );
    for tool_id in ["plink", "plink2"] {
        projection::append_plink(
            &mut rows,
            observations
                .publish_plink(tool_id, &report_root.join(format!("{tool_id}.adapter.json")))?
                .rows,
        );
    }
    for tool_id in ["shapeit", "shapeit5", "eagle", "beagle"] {
        projection::append_phasing(
            &mut rows,
            observations
                .publish_phasing(
                    tool_id,
                    &report_root.join(format!("{tool_id}.phasing.adapter.json")),
                )?
                .rows,
        );
    }
    projection::append_imputation(
        &mut rows,
        observations.publish_imputation(&report_root.join("imputation-family.adapter.json"))?.rows,
    );
    projection::append_descent(
        &mut rows,
        observations.publish_descent(&report_root.join("descent-family.adapter.json"))?.rows,
    );
    let mut row_map: BTreeMap<(String, String), VcfRenderedCommandRow> = BTreeMap::new();
    for mut row in rows {
        row.script_commands = render_script_commands(&row.command_steps)?;
        let key = (row.stage_id.clone(), row.tool_id.clone());
        if let Some(existing) = row_map.get(&(key.0.clone(), key.1.clone())) {
            if should_replace_existing_command_source(
                &key.0,
                &key.1,
                &existing.command_source,
                &row.command_source,
            ) {
                row_map.insert(key, row);
                continue;
            }
            return Err(anyhow!(
                "VCF rendered command row map encountered duplicate pair `{}` / `{}`",
                key.0,
                key.1
            ));
        }
        row_map.insert(key, row);
    }
    Ok(row_map)
}

fn should_replace_existing_command_source(
    stage_id: &str,
    tool_id: &str,
    existing_source: &str,
    candidate_source: &str,
) -> bool {
    stage_id == "vcf.roh"
        && tool_id == "plink2"
        && existing_source == "vcf_plink_family_adapter"
        && candidate_source == "vcf_descent_family_adapter"
}
