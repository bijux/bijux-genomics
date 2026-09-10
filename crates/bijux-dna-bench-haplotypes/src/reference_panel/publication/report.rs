//! Adapt admitted execution and observed evidence to the public report schema.
use crate::publication::ReportContext;
use crate::reference_panel::execution::artifacts::PanelArtifacts;
use crate::reference_panel::execution::inputs::PreparedPanelInputs;
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use crate::reference_panel::model::LocalVcfPrepareReferencePanelSmokeReport;
use crate::reference_panel::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::reference_panel::model::DEFAULT_STAGE_RESULT_NAME;
use crate::reference_panel::model::LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND;
use crate::reference_panel::model::LOCAL_VCF_PREPARE_PANEL_SMOKE_SCHEMA_VERSION;
use crate::reference_panel::observations::evidence::PanelEvidence;

pub(crate) fn build_report(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
    prepared: &PreparedPanelInputs,
    artifacts: &PanelArtifacts,
    evidence: &PanelEvidence,
) -> anyhow::Result<LocalVcfPrepareReferencePanelSmokeReport> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    Ok(LocalVcfPrepareReferencePanelSmokeReport {
        schema_version: LOCAL_VCF_PREPARE_PANEL_SMOKE_SCHEMA_VERSION,
        command: format!("{LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND} --tool-id {}", contract.tool),
        stage_id: contract.stage.clone(),
        tool_id: contract.tool.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        input_vcf_path: context.paths.path(&prepared.input_vcf_path)?,
        raw_panel_path: context.paths.path(&prepared.raw_panel_path)?,
        output_root: context.paths.path(context.paths.execution)?,
        panel_vcf_path: context.paths.path(&artifacts.panel_vcf)?,
        panel_tbi_path: context.paths.path(&artifacts.panel_tbi)?,
        panel_manifest_path: context.paths.path(&artifacts.panel_manifest)?,
        overlap_path: context.paths.path(&artifacts.overlap)?,
        panel_overlap_path: context.paths.path(&artifacts.panel_overlap)?,
        panel_files_path: context.paths.path(&artifacts.panel_files)?,
        overlap_tsv_path: context.paths.path(&artifacts.overlap_tsv)?,
        chunks_path: context.paths.path(&artifacts.chunks)?,
        metrics_path: context.paths.path(&metrics_path)?,
        stage_result_manifest_path: context.paths.path(&stage_result_manifest_path)?,
        started_at: context.started_at.to_owned(),
        finished_at: context.finished_at.to_owned(),
        elapsed_seconds: context.elapsed_seconds,
        exit_code: 0,
        input_variants: evidence.input_variants,
        output_variants: evidence.output_variants,
        sample_count: evidence.sample_count,
        sample_ids: evidence.sample_ids.clone(),
        sample_consistent: evidence.sample_consistent,
        duplicate_sites_removed: evidence.duplicate_sites_removed,
        normalization_status: evidence.normalization_status.clone(),
        index_path: context.paths.path(&artifacts.panel_tbi)?,
        parseable: true,
        validation_checks: evidence.validation.checks.clone(),
        gt_present: evidence.validation.gt_present,
        gl_present: evidence.validation.gl_present,
    })
}
