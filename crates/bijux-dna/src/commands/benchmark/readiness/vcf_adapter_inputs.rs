use crate::commands::benchmark::local_vcf_stage_catalog::build_vcf_stage_catalog_rows;
use crate::commands::benchmark::vcf_benchmark_bindings::collect_vcf_benchmark_binding_rows_at;
use anyhow::Result;
use bijux_dna_bench_vcf::inputs::VcfAdapterInputs;
use std::path::Path;
pub(super) fn load(repo_root: &Path) -> Result<VcfAdapterInputs> {
    VcfAdapterInputs::new(
        build_vcf_stage_catalog_rows()?,
        collect_vcf_benchmark_binding_rows_at(repo_root)?,
        bijux_dna_api::v1::api::bench::materialize_benchmark_reference_fasta,
    )
}
