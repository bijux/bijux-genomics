//! Owner: bijux-dna-bench-vcf
//! phasing adapter fixtures contracts.
use super::{
    anyhow, materialize_local_vcf_panel_fixture, path_relative_to_repo,
    with_local_vcf_panel_fixture_lock, MaterializedPanelInputs, Path, Result,
};

pub(super) fn materialize_panel_inputs(
    repo_root: &Path,
    output_root: &str,
) -> Result<MaterializedPanelInputs> {
    let materialization_root = repo_root.join(output_root).join("artifacts/reference");
    let report = with_local_vcf_panel_fixture_lock(|| {
        materialize_local_vcf_panel_fixture(&materialization_root)
    })?;
    let panel_vcf_path = report
        .materialized_files
        .iter()
        .find(|path| path.ends_with("/panel.vcf.gz"))
        .cloned()
        .ok_or_else(|| anyhow!("local VCF panel fixture did not produce panel.vcf.gz"))?;
    let genetic_map_path = report
        .materialized_files
        .iter()
        .find(|path| path.ends_with(".gmap.gz") || path.ends_with("/recombination_map.tsv.gz"))
        .cloned()
        .ok_or_else(|| {
            anyhow!("local VCF panel fixture did not produce a governed gzip genetic map")
        })?;

    Ok(MaterializedPanelInputs {
        panel_id: report.panel_id,
        map_id: report.map_id,
        panel_vcf_path: path_relative_to_repo(repo_root, Path::new(&panel_vcf_path)),
        genetic_map_path: path_relative_to_repo(repo_root, Path::new(&genetic_map_path)),
    })
}
