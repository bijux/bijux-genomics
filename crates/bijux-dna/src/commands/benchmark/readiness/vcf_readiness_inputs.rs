use anyhow::Result;
pub(crate) use bijux_dna_bench_vcf::fixtures::inputs::{
    load_governed_vcf_fixture_inputs, path_relative_to_repo,
};
use std::path::Path;
pub(crate) fn materialize_reference_fasta_with_index(
    repo_root: &Path,
    source_relative_path: &str,
    staging_root: &Path,
) -> Result<(String, String)> {
    let (fasta, index) = bijux_dna_api::v1::api::bench::materialize_benchmark_reference_fasta(
        &repo_root.join(source_relative_path),
        &repo_root.join(staging_root),
    )?;
    Ok((path_relative_to_repo(repo_root, &fasta), path_relative_to_repo(repo_root, &index)))
}
