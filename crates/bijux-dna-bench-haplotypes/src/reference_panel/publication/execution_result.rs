//! Publish stage outputs using their retained native execution addresses.
use crate::publication::ReportContext;
use crate::reference_panel::execution::artifacts::PanelArtifacts;
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use crate::reference_panel::model::DEFAULT_CHUNKS_NAME;
use crate::reference_panel::model::DEFAULT_OUTPUT_METRICS_NAME;
use crate::reference_panel::model::DEFAULT_OUTPUT_PANEL_NAME;
use crate::reference_panel::model::DEFAULT_OVERLAP_NAME;
use crate::reference_panel::model::DEFAULT_OVERLAP_TSV_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_FILES_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_MANIFEST_NAME;
use crate::reference_panel::model::DEFAULT_PANEL_OVERLAP_NAME;
use crate::reference_panel::model::DEFAULT_STAGE_RESULT_NAME;
use crate::reference_panel::model::LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND;
use crate::reference_panel::publication::stage_result::build_stage_result_manifest;
use crate::stage_result::validate_stage_result_manifest;
use anyhow::Result;

pub(crate) fn publish_stage_result(
    context: &ReportContext<'_>,
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
    artifacts: &PanelArtifacts,
) -> Result<()> {
    let metrics_path = context.paths.execution.join(DEFAULT_OUTPUT_METRICS_NAME);
    let stage_result_manifest_path = context.paths.execution.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = build_stage_result_manifest(
        context.paths.repository,
        contract,
        &format!("{LOCAL_VCF_PREPARE_PANEL_SMOKE_COMMAND} --tool-id {}", contract.tool),
        &[
            (
                "prepared_panel",
                DEFAULT_OUTPUT_PANEL_NAME.to_string(),
                artifacts.panel_vcf.as_path(),
                "vcf_output",
            ),
            (
                "prepared_panel_tbi",
                format!("{DEFAULT_OUTPUT_PANEL_NAME}.tbi"),
                artifacts.panel_tbi.as_path(),
                "index_output",
            ),
            (
                "panel_manifest_json",
                DEFAULT_PANEL_MANIFEST_NAME.to_string(),
                artifacts.panel_manifest.as_path(),
                "report_output",
            ),
            (
                "overlap_json",
                DEFAULT_OVERLAP_NAME.to_string(),
                artifacts.overlap.as_path(),
                "report_output",
            ),
            (
                "panel_overlap_json",
                DEFAULT_PANEL_OVERLAP_NAME.to_string(),
                artifacts.panel_overlap.as_path(),
                "report_output",
            ),
            (
                "panel_files_json",
                DEFAULT_PANEL_FILES_NAME.to_string(),
                artifacts.panel_files.as_path(),
                "report_output",
            ),
            (
                "overlap_tsv",
                DEFAULT_OVERLAP_TSV_NAME.to_string(),
                artifacts.overlap_tsv.as_path(),
                "table_output",
            ),
            (
                "chunks_json",
                DEFAULT_CHUNKS_NAME.to_string(),
                artifacts.chunks.as_path(),
                "report_output",
            ),
            (
                "metrics_json",
                DEFAULT_OUTPUT_METRICS_NAME.to_string(),
                metrics_path.as_path(),
                "report_output",
            ),
        ],
        context.started_at,
        context.finished_at,
        context.elapsed_seconds,
    );
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(())
}
