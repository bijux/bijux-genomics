use super::model::VcfAdapterMissingInputTestRow;
use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
pub(super) fn ensure_required_role_coverage(rows: &[VcfAdapterMissingInputTestRow]) -> Result<()> {
    let expected_roles = [
        "input_bam",
        "input_bam_index",
        "reference_fasta",
        "reference_fai",
        "vcf",
        "vcf_index",
        "target_sites_bed",
        "reference_panel_vcf",
        "genetic_map_tsv",
        "sample_metadata_manifest",
    ];
    let observed_roles =
        rows.iter().map(|row| row.missing_input_role.as_str()).collect::<BTreeSet<_>>();
    let expected_role_set = expected_roles.into_iter().collect::<BTreeSet<_>>();
    if observed_roles != expected_role_set {
        return Err(anyhow!(
            "VCF missing-input test role coverage drifted: expected {expected_role_set:?}, found {observed_roles:?}"
        ));
    }
    Ok(())
}
