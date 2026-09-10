//! Owner: bijux-dna-bench-population
//! Population structure admission upstream ownership.
use crate::admixture::run_local_vcf_admixture_smoke;
use crate::inputs::PopulationBenchmarkInputs;
use crate::pca::run_local_vcf_pca_smoke;
use crate::population_structure::execution::artifacts::ensure_file_exists;
use anyhow::Result;
use std::path::Path;

pub(crate) struct ConsumedPopulationInputs {
    pub(crate) pca_report: crate::pca::LocalVcfPcaSmokeReport,
    pub(crate) admixture_report: crate::admixture::LocalVcfAdmixtureSmokeReport,
    pub(crate) upstream_input_vcf: std::path::PathBuf,
    pub(crate) upstream_population_labels_manifest: std::path::PathBuf,
}

pub(crate) fn acquire_upstream(
    inputs: &PopulationBenchmarkInputs,
    repo_root: &Path,
    contract: &crate::population_structure::model::GovernedVcfPopulationStructureSmokeContract,
) -> Result<ConsumedPopulationInputs> {
    let pca_report =
        run_local_vcf_pca_smoke(inputs, repo_root, &contract.normalization_pca_tool_id)?;
    let admixture_report =
        run_local_vcf_admixture_smoke(inputs, repo_root, &contract.admixture_tool_id)?;

    let upstream_input_vcf = repo_root.join(&pca_report.input_vcf_path);
    let upstream_population_labels_manifest =
        repo_root.join(&pca_report.population_labels_manifest_path);
    ensure_file_exists(&upstream_input_vcf, "population-structure upstream input VCF")?;
    ensure_file_exists(
        &upstream_population_labels_manifest,
        "population-structure upstream metadata manifest",
    )?;

    Ok(ConsumedPopulationInputs {
        pca_report,
        admixture_report,
        upstream_input_vcf,
        upstream_population_labels_manifest,
    })
}
