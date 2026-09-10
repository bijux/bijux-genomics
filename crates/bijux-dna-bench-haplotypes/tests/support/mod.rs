//! Explicit repository declarations supplied to the haplotype service under test.
use crate::inputs::HaplotypeBenchmarkInputs;
use bijux_dna_bench_model::LocalVcfStageMatrixConfig;
use bijux_dna_core::id_catalog as stage;
pub(crate) fn inputs() -> HaplotypeBenchmarkInputs {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace");
    let matrix: LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(root.join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix"),
    )
    .expect("matrix declaration");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix).expect("admit matrix");
    let rows = matrix
        .rows
        .into_iter()
        .filter(|row| {
            [
                stage::VCF_PREPARE_REFERENCE_PANEL,
                stage::VCF_PHASING,
                stage::VCF_IMPUTE,
                stage::VCF_IMPUTATION_METRICS,
            ]
            .contains(&row.stage_id.as_str())
        })
        .collect();
    HaplotypeBenchmarkInputs::new(
        rows,
        bijux_dna_stage_contract::canonical_tool_bindings().expect("canonical bindings"),
    )
    .expect("admit haplotype declarations")
}
