//! Adapt validated scientific evidence to the persisted metrics schema.
use crate::publication::PublishedArtifactPaths;
use crate::reference_panel::execution::artifacts::PanelArtifacts;
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use crate::reference_panel::model::LocalVcfPrepareReferencePanelSmokeMetrics;
use crate::reference_panel::model::LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION;
use crate::reference_panel::observations::evidence::PanelEvidence;

pub(crate) fn build_metrics(
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
    evidence: &PanelEvidence,
    published: &PublishedArtifactPaths<'_>,
    artifacts: &PanelArtifacts,
) -> anyhow::Result<LocalVcfPrepareReferencePanelSmokeMetrics> {
    Ok(LocalVcfPrepareReferencePanelSmokeMetrics {
        schema_version: LOCAL_VCF_PREPARE_PANEL_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: contract.input_fixture.clone(),
        input_variants: evidence.input_variants,
        output_variants: evidence.output_variants,
        sample_count: evidence.sample_count,
        sample_ids: evidence.sample_ids.clone(),
        duplicate_sites_removed: evidence.duplicate_sites_removed,
        normalization_status: evidence.normalization_status.clone(),
        index_path: published.path(&artifacts.panel_tbi)?,
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        tool_id: contract.tool.clone(),
        exit_code: 0,
    })
}
