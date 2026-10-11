use anyhow::{anyhow, Result};
use bijux_dna_domain_fastq::{
    params::qc_post::{QcAggregationEngine, QcAggregationScope},
    GovernedQcContributorV1, GovernedQcManifestContributorV1,
};

pub(super) fn parse_aggregation_engine(value: &str) -> Result<QcAggregationEngine> {
    match value {
        "auto" | "multiqc" => Ok(QcAggregationEngine::Multiqc),
        other => Err(anyhow!("unsupported local fastq.report_qc aggregation_engine `{other}`")),
    }
}

pub(super) fn parse_aggregation_scope(value: &str) -> Result<QcAggregationScope> {
    match value {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        other => Err(anyhow!("unsupported local fastq.report_qc aggregation_scope `{other}`")),
    }
}

pub(super) fn contributor_stage_ids(
    contributors: &[GovernedQcManifestContributorV1],
) -> Vec<String> {
    let mut stage_ids =
        contributors.iter().map(|contributor| contributor.stage_id.clone()).collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

pub(super) fn contributor_tool_ids(
    contributors: &[GovernedQcManifestContributorV1],
) -> Vec<String> {
    let mut tool_ids =
        contributors.iter().map(|contributor| contributor.tool_id.clone()).collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

pub(super) fn report_contributor_from_manifest(
    contributor: &GovernedQcManifestContributorV1,
) -> GovernedQcContributorV1 {
    GovernedQcContributorV1 {
        contributor_id: contributor.contributor_id.clone(),
        stage_id: contributor.stage_id.clone(),
        tool_id: contributor.tool_id.clone(),
        artifact_id: contributor.artifact_id.clone(),
        artifact_role: contributor.artifact_role.as_str().to_string(),
        path: contributor.path.display().to_string(),
    }
}
