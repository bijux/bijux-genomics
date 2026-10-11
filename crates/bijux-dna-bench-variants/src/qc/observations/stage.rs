//! Admit scientific output evidence and construct the persisted benchmark metrics.
use crate::qc::execution::artifacts::StageArtifacts;
use crate::qc::model::GovernedVcfQcSmokeContract;
use crate::qc::model::LocalVcfQcSmokeMetrics;
use crate::qc::model::LocalVcfQcSummary;
use crate::qc::model::LOCAL_VCF_QC_SMOKE_METRICS_SCHEMA_VERSION;
use crate::qc::observations::reports::read_qc_summary;
use crate::qc::observations::variants::validate_qc_summary;
use anyhow::Result;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfQcSmokeMetrics,
    pub(crate) summary: LocalVcfQcSummary,
}

pub(crate) fn observe_stage(
    contract: &GovernedVcfQcSmokeContract,
    artifacts: &StageArtifacts,
) -> Result<StageEvidence> {
    let qc_summary_path = &artifacts.qc_summary;
    let summary = read_qc_summary(qc_summary_path)?;
    validate_qc_summary(&summary)?;

    let metrics = LocalVcfQcSmokeMetrics {
        schema_version: LOCAL_VCF_QC_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        sample_missingness: summary.sample_missingness.clone(),
        variant_missingness: summary.variant_missingness.clone(),
        maf_summary: summary.maf_summary.clone(),
        heterozygosity: summary.heterozygosity.clone(),
        hwe_summary: summary.hwe_summary.clone(),
        excluded_samples: summary.excluded_samples.clone(),
        excluded_variants: summary.excluded_variants.clone(),
        sample_missingness_exclusion_threshold: summary.sample_missingness_exclusion_threshold,
        variant_missingness_exclusion_threshold: summary.variant_missingness_exclusion_threshold,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, summary })
}
