use super::observations::OutputSource;
use anyhow::{anyhow, bail, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageRow;
use std::collections::BTreeMap;
pub(super) struct VcfAdapterOutputCoverageOwnedRow {
    pub(super) family_label: String,
    pub(super) row: VcfAdapterOutputCoverageRow,
}
pub(super) fn append_output_rows<T: OutputSource>(
    owned_rows: &mut BTreeMap<(String, String), VcfAdapterOutputCoverageOwnedRow>,
    family_label: &str,
    source_rows: Vec<T>,
    run_id: &str,
) -> Result<()> {
    for row in source_rows {
        let normalized_row = super::projection::project_output_observation(
            family_label,
            row.output_observation(),
            run_id,
        )?;
        let key = (normalized_row.stage_id.clone(), normalized_row.tool_id.clone());
        match owned_rows.get(&key) {
            None => {
                owned_rows.insert(
                    key,
                    VcfAdapterOutputCoverageOwnedRow {
                        family_label: family_label.to_string(),
                        row: normalized_row,
                    },
                );
            }
            Some(existing) => {
                let canonical_family = canonical_family_for_pair(
                    normalized_row.stage_id.as_str(),
                    normalized_row.tool_id.as_str(),
                )
                .ok_or_else(|| {
                    anyhow!(
                        "VCF adapter output coverage discovered an unmapped duplicate row `{}` / `{}` between `{}` and `{}`",
                        normalized_row.stage_id,
                        normalized_row.tool_id,
                        existing.family_label,
                        family_label
                    )
                })?;
                if existing.family_label == canonical_family && family_label != canonical_family {
                    continue;
                }
                if family_label == canonical_family && existing.family_label != canonical_family {
                    owned_rows.insert(
                        key,
                        VcfAdapterOutputCoverageOwnedRow {
                            family_label: family_label.to_string(),
                            row: normalized_row,
                        },
                    );
                    continue;
                }
                bail!(
                    "VCF adapter output coverage found conflicting owned rows `{}` / `{}` for canonical family `{}` between `{}` and `{}`",
                    normalized_row.stage_id,
                    normalized_row.tool_id,
                    canonical_family,
                    existing.family_label,
                    family_label
                );
            }
        }
    }
    Ok(())
}

fn canonical_family_for_pair(stage_id: &str, tool_id: &str) -> Option<&'static str> {
    match (stage_id, tool_id) {
        ("vcf.roh", "plink2") => Some("descent"),
        ("vcf.impute" | "vcf.imputation_metrics", "beagle") => Some("imputation_family"),
        _ => None,
    }
}
