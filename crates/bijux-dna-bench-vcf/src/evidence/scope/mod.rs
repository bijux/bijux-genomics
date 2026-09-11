//! Retained registry and active binding identities have explicit request ownership.
mod admission;
mod contract;
mod governed;
mod model;
mod projection;
mod publication;
mod registry;
mod status;
pub use projection::collect_vcf_active_stage_tool_matrix_rows_from_expected;
pub use publication::render_vcf_active_stage_tool_matrix;
pub use status::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH;

/// Independent authority for full active stage/tool/corpus/asset binding identities.
pub type ActiveBindingProvider = fn(
    &std::path::Path,
) -> anyhow::Result<
    Vec<bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey>,
>;
