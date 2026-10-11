//! Explicit authored processing templates and report-format bindings for service contracts.
use crate::inputs::VariantBenchmarkInputs;
use bijux_dna_bench_model::{LocalVcfStageMatrixConfig, VcfStageMatrixRow};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn rows() -> (Vec<VcfStageMatrixRow>, Vec<VcfStageMatrixRow>) {
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
    let qc_template =
        matrix.rows.iter().find(|row| row.stage_id == stage::VCF_QC).expect("QC template");
    let qc = ["bcftools", "plink", "plink2"]
        .into_iter()
        .map(|tool| {
            let mut row = qc_template.clone();
            row.tool_id = tool.to_owned();
            row
        })
        .collect();
    let governed = matrix
        .rows
        .into_iter()
        .filter(|row| {
            [stage::VCF_FILTER, stage::VCF_DAMAGE_FILTER, stage::VCF_POSTPROCESS]
                .contains(&row.stage_id.as_str())
        })
        .collect();
    (governed, qc)
}

pub(crate) fn inputs() -> VariantBenchmarkInputs {
    let (governed, qc) = rows();
    VariantBenchmarkInputs::new(governed, qc).expect("admit processing declarations")
}
