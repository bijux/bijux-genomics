use super::authority::{append_output_rows, VcfAdapterOutputCoverageOwnedRow};
use crate::adapters::angsd::DEFAULT_VCF_ANGSD_ADAPTER_PATH;
use crate::adapters::bcftools::DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH;
use crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH;
use crate::adapters::eigensoft::DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH;
use crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH;
use crate::adapters::observations::VcfAdapterObservations;
use crate::adapters::phasing::{
    DEFAULT_VCF_BEAGLE_ADAPTER_PATH, DEFAULT_VCF_EAGLE_ADAPTER_PATH,
    DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT_ADAPTER_PATH,
};
use crate::adapters::plink::{DEFAULT_VCF_PLINK2_ADAPTER_PATH, DEFAULT_VCF_PLINK_ADAPTER_PATH};
use anyhow::{ensure, Result};
use bijux_dna_bench_vcf_evidence::model::outputs::VcfAdapterOutputCoverageRow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Acquire compatible reports and evaluate declared output coverage for one request.
///
/// # Errors
/// Returns family acquisition/publication, output declaration or coverage contract errors.
pub fn collect_vcf_adapter_output_coverage_rows(
    observations: &VcfAdapterObservations<'_>,
    run_id: &str,
) -> Result<Vec<VcfAdapterOutputCoverageRow>> {
    ensure!(
        !run_id.trim().is_empty()
            && run_id.trim() == run_id
            && !run_id.contains(['/', '\\', '\0'])
            && !matches!(run_id, "." | ".."),
        "VCF output coverage run identity must be one nonempty path component"
    );
    let repo_root = observations.repo_root();
    let tool_serving_map = crate::evidence::serving::render_vcf_tool_serving_map(
        observations.inputs(),
        repo_root,
        Path::new(crate::evidence::serving::DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
    )?;
    let benchmark_ready_pairs = tool_serving_map
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    let owned_rows = collect_family_outputs(observations, run_id)?;

    let mut rows = owned_rows
        .into_values()
        .map(|owned_row| {
            let mut row = owned_row.row;
            row.benchmark_status =
                if benchmark_ready_pairs.contains(&(row.stage_id.clone(), row.tool_id.clone())) {
                    "benchmark_ready".to_string()
                } else {
                    "not_benchmark_ready".to_string()
                };
            row
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.stage_id.cmp(&right.stage_id))
    });
    super::contract::ensure_vcf_adapter_output_coverage_contract(&rows, &benchmark_ready_pairs)?;
    Ok(rows)
}

fn collect_family_outputs(
    observations: &VcfAdapterObservations<'_>,
    run_id: &str,
) -> Result<BTreeMap<(String, String), VcfAdapterOutputCoverageOwnedRow>> {
    let mut owned_rows = BTreeMap::<(String, String), VcfAdapterOutputCoverageOwnedRow>::new();
    append_output_rows(
        &mut owned_rows,
        "bcftools",
        observations.publish_bcftools(Path::new(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "angsd",
        observations.publish_angsd(Path::new(DEFAULT_VCF_ANGSD_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "descent",
        observations.publish_descent(Path::new(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "eigensoft",
        observations.publish_eigensoft(Path::new(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "imputation_family",
        observations
            .publish_imputation(Path::new(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH))?
            .rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "plink",
        observations.publish_plink("plink", Path::new(DEFAULT_VCF_PLINK_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "plink2",
        observations.publish_plink("plink2", Path::new(DEFAULT_VCF_PLINK2_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "shapeit",
        observations.publish_phasing("shapeit", Path::new(DEFAULT_VCF_SHAPEIT_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "shapeit5",
        observations
            .publish_phasing("shapeit5", Path::new(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH))?
            .rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "eagle",
        observations.publish_phasing("eagle", Path::new(DEFAULT_VCF_EAGLE_ADAPTER_PATH))?.rows,
        run_id,
    )?;
    append_output_rows(
        &mut owned_rows,
        "beagle_phasing",
        observations.publish_phasing("beagle", Path::new(DEFAULT_VCF_BEAGLE_ADAPTER_PATH))?.rows,
        run_id,
    )?;

    Ok(owned_rows)
}
