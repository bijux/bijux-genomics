use super::all_domain_active_stage_tool_matrix::collect_all_domain_active_stage_tool_matrix_rows;
use anyhow::Result;
use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
pub(crate) use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use bijux_dna_bench_vcf_evidence::{
    model::command_reports::VcfRenderedCommandsReport,
    snapshot::{ActiveVcfBinding, VcfEvidenceSnapshot, VcfStageReadinessBindingKey},
};
use std::path::Path;
pub(crate) fn collect_vcf_stage_readiness_bindings(
    repo_root: &Path,
    stage_id: &str,
) -> Result<(VcfRenderedCommandsReport, Vec<VcfStageReadinessBinding>)> {
    let (commands, evidence) = collect_vcf_readiness_evidence(repo_root)?;
    Ok((commands, evidence.stage_bindings(stage_id)?.to_vec()))
}
pub(crate) fn collect_vcf_readiness_evidence(
    repo_root: &Path,
) -> Result<(VcfRenderedCommandsReport, VcfEvidenceSnapshot)> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    collect_vcf_readiness_evidence_with_observations(&observations)
}
pub(crate) fn collect_vcf_readiness_evidence_with_observations(
    observations: &VcfAdapterObservations<'_>,
) -> Result<(VcfRenderedCommandsReport, VcfEvidenceSnapshot)> {
    bijux_dna_bench_vcf::evidence::readiness::collect_vcf_readiness_evidence_with_observations(
        observations,
        crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID,
        active_evidence,
    )
}
pub(super) fn active_evidence(repo_root: &Path) -> Result<Vec<ActiveVcfBinding>> {
    Ok(collect_all_domain_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.domain == "vcf")
        .map(|row| ActiveVcfBinding {
            key: VcfStageReadinessBindingKey {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                corpus_id: row.corpus_id,
                asset_profile_id: row.asset_profile_id,
            },
            adapter_id: row.adapter_id,
            parser_id: row.parser_id,
            schema_id: row.schema_id,
        })
        .collect())
}
