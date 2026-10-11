//! Adapt validated scientific evidence to the persisted metrics schema.
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::phasing::model::LocalVcfPhasingSmokeMetrics;
use crate::phasing::model::GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID;
use crate::phasing::model::LOCAL_VCF_PHASING_SMOKE_METRICS_SCHEMA_VERSION;
use crate::phasing::observations::stage::PhasingEvidence;

pub(crate) fn build_metrics(
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    evidence: &PhasingEvidence,
) -> LocalVcfPhasingSmokeMetrics {
    LocalVcfPhasingSmokeMetrics {
        schema_version: LOCAL_VCF_PHASING_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: GOVERNED_VCF_PHASING_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        input_genotypes: evidence.input_summary.genotype_count,
        phased_genotypes: evidence.output_summary.phased_genotypes,
        unphased_genotypes: evidence.output_summary.unphased_genotypes,
        phase_set_count: evidence.output_summary.phase_set_count,
        sample_count: evidence.sample_count,
        sample_ids: evidence.output_summary.sample_ids.clone(),
        tool_id: contract.tool.clone(),
        exit_code: 0,
    }
}
