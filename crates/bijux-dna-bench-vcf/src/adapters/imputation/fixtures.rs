//! Owner: bijux-dna-bench-vcf
//! imputation adapter fixtures contracts.
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
    let panel_msav_path =
        report.materialized_files.iter().find(|path| path.ends_with("/panel.msav")).cloned();
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
        panel_msav_path: panel_msav_path
            .as_deref()
            .map(|path| path_relative_to_repo(repo_root, Path::new(path))),
        genetic_map_path: path_relative_to_repo(repo_root, Path::new(&genetic_map_path)),
    })
}

pub(super) fn prepare_target_inputs(
    repo_root: &Path,
    tool_id: &str,
    stage_id: &str,
    output_root: &str,
    materialized_inputs: &MaterializedPanelInputs,
) -> Result<(String, Vec<super::VcfImputationFamilyAdapterArtifact>)> {
    use super::commands::artifact;
    use super::{materialize_indexed_vcf_input, PathBuf};
    let (target_vcf_path, target_vcf_index_path) = materialize_indexed_vcf_input(
        repo_root,
        super::commands::governed_target_vcf_path(tool_id)?,
        &PathBuf::from(&output_root).join("artifacts/input"),
        &format!("{}.vcf.gz", stage_id.replace('.', "_")),
    )?;
    let mut required_inputs = vec![
        artifact("vcf", "variant", &target_vcf_path),
        artifact("vcf_index", "index", &target_vcf_index_path),
        artifact("reference_panel_vcf", "variant", &materialized_inputs.panel_vcf_path),
    ];
    if tool_id == "minimac4" {
        required_inputs.push(artifact(
            "reference_panel_msav",
            "variant",
            materialized_inputs.panel_msav_path.as_deref().ok_or_else(|| {
                anyhow!("VCF minimac4 adapter row `{stage_id}` is missing panel.msav")
            })?,
        ));
    }
    required_inputs.push(artifact(
        "genetic_map_tsv",
        "reference",
        &materialized_inputs.genetic_map_path,
    ));

    Ok((target_vcf_path, required_inputs))
}
