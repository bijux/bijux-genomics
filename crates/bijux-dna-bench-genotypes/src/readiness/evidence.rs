use crate::inputs::GenotypeBenchmarkInputs;
use anyhow::{ensure, Result};
pub use bijux_dna_bench_vcf_evidence::model::command_reports::VcfCommandPublicationPaths as ReadinessCommandPaths;
use bijux_dna_bench_vcf_evidence::snapshot::{VcfEvidenceSnapshot, VcfStageReadinessBinding};

// Each independent evidence surface has its own observed coverage; combinations are valid incomplete evidence.
pub(super) struct SurfaceAssessment {
    pub result_id: String,
    pub required_metric_names: Vec<String>,
    pub missing_surfaces: Vec<String>,
    pub all_domain_active_row_present: bool,
    pub command_ready: bool,
    pub output_ready: bool,
    pub parser_ready: bool,
    pub expected_result_ready: bool,
    pub report_ready: bool,
    pub smoke_ready: bool,
    pub coverage_status: String,
    pub reason: String,
}
pub(super) fn admit_stage<'a>(
    inputs: &GenotypeBenchmarkInputs,
    snapshot: &'a VcfEvidenceSnapshot,
    commands: &ReadinessCommandPaths,
    stage: &str,
) -> Result<&'a [VcfStageReadinessBinding]> {
    ensure!(
        [&commands.output_path, &commands.argv_output_path]
            .into_iter()
            .all(|path| !path.is_empty() && path.trim() == path),
        "genotype readiness command publication paths must be nonempty and unpadded"
    );
    let declaration = inputs
        .rows()
        .find(|row| row.stage_id == stage)
        .ok_or_else(|| anyhow::anyhow!("genotype readiness is missing its stage declaration"))?;
    let descriptors=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors()?;
    let descriptor = descriptors.iter().find(|row| row.stage_id == stage).ok_or_else(|| {
        anyhow::anyhow!("genotype readiness is missing its normalized stage schema")
    })?;
    let bindings = snapshot.stage_bindings(stage)?;
    for binding in bindings {
        let row = &binding.retained_row;
        ensure!(
            row.corpus_id == declaration.corpus_id
                && row.asset_profile_id == declaration.asset_profile_id
                && row.adapter_id == declaration.adapter_id
                && row.parser_id == declaration.parser_id
                && row.schema_id == descriptor.schema_id,
            "genotype readiness snapshot disagrees with admitted stage inputs"
        );
    }
    Ok(bindings)
}
