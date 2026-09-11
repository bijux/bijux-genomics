//! Authored genotype declarations and original scientific fixtures.
use crate::inputs::GenotypeBenchmarkInputs;
use bijux_dna_bench_model::{LocalVcfStageMatrixConfig, VcfStageMatrixRow};
pub(crate) fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates")
        .parent()
        .expect("workspace")
        .to_path_buf()
}
pub(crate) fn rows() -> Vec<VcfStageMatrixRow> {
    let matrix: LocalVcfStageMatrixConfig = toml::from_str(
        &std::fs::read_to_string(repo_root().join("configs/bench/local/vcf-stage-matrix.toml"))
            .expect("matrix"),
    )
    .expect("matrix declaration");
    bijux_dna_bench_model::validate_local_vcf_stage_matrix_config(&matrix).expect("admit matrix");
    matrix
        .rows
        .into_iter()
        .filter(|row| {
            [
                "vcf.call",
                "vcf.call_gl",
                "vcf.call_diploid",
                "vcf.call_pseudohaploid",
                "vcf.gl_propagation",
            ]
            .contains(&row.stage_id.as_str())
        })
        .collect()
}
pub(crate) fn inputs() -> GenotypeBenchmarkInputs {
    GenotypeBenchmarkInputs::new(rows()).expect("admit genotype declarations")
}
