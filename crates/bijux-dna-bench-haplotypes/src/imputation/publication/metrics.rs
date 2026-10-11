//! Adapt validated scientific evidence to the persisted metrics schema.
use crate::imputation::model::LocalVcfImputeSmokeMetrics;
use crate::imputation::model::GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID;
use crate::imputation::model::LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION;
use crate::imputation::observations::evidence::ImputationEvidence;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;

pub(crate) fn build_metrics(
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    evidence: &ImputationEvidence,
) -> LocalVcfImputeSmokeMetrics {
    LocalVcfImputeSmokeMetrics {
        schema_version: LOCAL_VCF_IMPUTE_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage.clone(),
        corpus_id: contract.corpus.clone(),
        input_fixture_id: GOVERNED_VCF_IMPUTE_INPUT_FIXTURE_ID.to_string(),
        panel_id: contract.panel.clone(),
        map_id: contract.map.clone(),
        variant_count: evidence.output_summary.variant_count,
        missing_before: evidence.quality.missing_before,
        missing_after: evidence.quality.missing_after,
        imputed_genotypes: evidence.quality.imputed_genotypes,
        low_confidence_count: evidence.quality.low_confidence_count,
        masked_truth_site_count: evidence.quality.masked_truth_site_count,
        masked_truth_match_count: evidence.quality.masked_truth_match_count,
        unresolved_count: evidence.quality.unresolved_count,
        not_imputable_reasons: evidence.quality.not_imputable_reasons.clone(),
        sample_count: evidence.sample_count,
        sample_ids: evidence.output_summary.sample_ids.clone(),
        tool_id: contract.tool.clone(),
        exit_code: 0,
    }
}
