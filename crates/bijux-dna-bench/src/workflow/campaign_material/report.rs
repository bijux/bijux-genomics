//! Owner: bijux-dna-bench
//! Plan-driven publication across crate-owned benchmark campaign reports.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_campaign_report_publication, BenchmarkCampaignPlan, BenchmarkCaseMatrixV2,
    BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
};

pub use bijux_dna_bench_model::BenchmarkCampaignReportPublication;

use crate::workflow::bam_alignment::publish_bam_alignment_stage_report_with_presentation_authority;
use crate::workflow::bam_coverage::publish_bam_coverage_stage_report_with_presentation_authority;
use crate::workflow::bam_damage::publish_bam_damage_stage_report_with_presentation_authority;
use crate::workflow::bam_duplication_metrics::publish_bam_duplication_metrics_stage_report_with_presentation_authority;
use crate::workflow::bam_endogenous_content::publish_bam_endogenous_content_stage_report_with_presentation_authority;
use crate::workflow::bam_genotyping::publish_bam_genotyping_stage_report_with_presentation_authority;
use crate::workflow::bam_mapping_summary::publish_bam_mapping_summary_stage_report_with_presentation_authority;
use crate::workflow::bam_picard_cohort::{
    publish_bam_gc_bias_stage_report_with_presentation_authority,
    publish_bam_insert_size_stage_report_with_presentation_authority,
};
use crate::workflow::fastq_detect_adapters::publish_fastq_detect_adapters_stage_report_with_presentation_authority;
use crate::workflow::fastq_profile_read_lengths::publish_fastq_profile_read_lengths_stage_report_with_presentation_authority;
use crate::workflow::fastq_profile_reads::publish_fastq_profile_reads_stage_report_with_presentation_authority;
use crate::workflow::fastq_validate_reads::publish_fastq_validate_reads_stage_report_with_presentation_authority;
use crate::workflow::{BenchmarkCampaignPlanSource, BenchmarkStageReportPublication};

type CampaignReportPublisher = fn(
    &Path,
    &Path,
    &Path,
    &Path,
    &BenchmarkCampaignPlan,
    BenchmarkCampaignPlanSource<'_>,
    &Path,
) -> Result<BenchmarkStageReportPublication>;

struct BenchmarkCampaignReportAdapter {
    stage_id: &'static str,
    publish: CampaignReportPublisher,
}

/// Reviewed material-driver adapters retain stage-specific scientific interpretation for the
/// original campaigns. The generic case-aggregation reporter owns every campaign, including the
/// derived stages that intentionally have no reviewed material driver.
const REPORT_ADAPTERS: &[BenchmarkCampaignReportAdapter] = &[
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.align",
        publish: publish_bam_alignment_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.genotyping",
        publish: publish_bam_genotyping_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.coverage",
        publish: publish_bam_coverage_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.damage",
        publish: publish_bam_damage_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.endogenous_content",
        publish: publish_bam_endogenous_content_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.mapping_summary",
        publish: publish_bam_mapping_summary_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.duplication_metrics",
        publish: publish_bam_duplication_metrics_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.gc_bias",
        publish: publish_bam_gc_bias_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.insert_size",
        publish: publish_bam_insert_size_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.detect_adapters",
        publish: publish_fastq_detect_adapters_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.profile_read_lengths",
        publish: publish_fastq_profile_read_lengths_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.profile_reads",
        publish: publish_fastq_profile_reads_stage_report_with_presentation_authority,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.validate_reads",
        publish: publish_fastq_validate_reads_stage_report_with_presentation_authority,
    },
];

/// Derive and publish the report selected by a source-locked campaign plan.
///
/// Each stage adapter retains ownership of its scientific and evidence validation. The validated
/// plan's stable stage identity selects that adapter without coupling lifecycle dispatch to run IDs.
///
/// # Errors
/// Returns an error when the plan mixes identities, names a stage without an adapter, or the
/// selected report fails source, evidence, science, or no-clobber validation.
pub fn publish_benchmark_campaign_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BenchmarkCampaignReportPublication> {
    publish_benchmark_campaign_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        output_root,
    )
}

/// Publish one source-locked campaign using an explicit render-only presentation authority.
///
/// # Errors
/// Returns an error under the same conditions as [`publish_benchmark_campaign_report`], or when
/// the presentation repository cannot complete a historical interface catalog.
pub fn publish_benchmark_campaign_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<BenchmarkCampaignReportPublication> {
    publish_benchmark_campaign_report_with_source_authority(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        BenchmarkCampaignPlanSource::Legacy,
        output_root,
    )
}

/// Derive and publish a case-scoped campaign report against its exact matrix authority.
///
/// # Errors
/// Returns an error when the plan and matrix identities differ, or under the same evidence,
/// science, source, and no-clobber conditions as [`publish_benchmark_campaign_report`].
pub fn publish_case_benchmark_campaign_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    case_matrix: &BenchmarkCaseMatrixV2,
    output_root: &Path,
) -> Result<BenchmarkCampaignReportPublication> {
    publish_case_benchmark_campaign_report_with_presentation_authority(
        repository_root,
        repository_root,
        study_root,
        benchmark_root,
        plan,
        case_matrix,
        output_root,
    )
}

/// Publish a case-scoped campaign using exact matrix and render-only presentation authorities.
///
/// # Errors
/// Returns an error under the same conditions as [`publish_case_benchmark_campaign_report`], or
/// when the presentation repository cannot complete a historical interface catalog.
pub fn publish_case_benchmark_campaign_report_with_presentation_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    case_matrix: &BenchmarkCaseMatrixV2,
    output_root: &Path,
) -> Result<BenchmarkCampaignReportPublication> {
    publish_benchmark_campaign_report_with_source_authority(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        BenchmarkCampaignPlanSource::CaseMatrix(case_matrix),
        output_root,
    )
}

fn publish_benchmark_campaign_report_with_source_authority(
    repository_root: &Path,
    presentation_repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    source_authority: BenchmarkCampaignPlanSource<'_>,
    output_root: &Path,
) -> Result<BenchmarkCampaignReportPublication> {
    let first = plan.cycle.attempts.first().context("benchmark campaign plan has no attempts")?;
    let run_id = first.identity.run.run_id.as_str();
    let stage_id = first.identity.run.stage_id.as_str();
    if plan.cycle.attempts.iter().any(|attempt| {
        attempt.identity.run.run_id != run_id || attempt.identity.run.stage_id != stage_id
    }) {
        bail!("benchmark campaign plan mixes run or stage identities");
    }

    let adapter =
        REPORT_ADAPTERS.iter().find(|adapter| adapter.stage_id == stage_id).with_context(|| {
            format!("benchmark stage has no crate-owned report adapter: {stage_id}")
        })?;
    let publication = (adapter.publish)(
        repository_root,
        presentation_repository_root,
        study_root,
        benchmark_root,
        plan,
        source_authority,
        output_root,
    )?;
    let publication = BenchmarkCampaignReportPublication {
        schema_version: BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1.to_string(),
        run_id: run_id.to_string(),
        stage_id: stage_id.to_string(),
        report_path: publication.report_path,
        report_sha256: publication.report_sha256,
        markdown_path: publication.markdown_path,
        markdown_sha256: publication.markdown_sha256,
        presentation_repository_commit: publication.presentation_authority.repository_commit,
        presentation_catalog_sha256: publication.presentation_authority.catalog_sha256,
        presentation_overlaid_stage_ids: publication.presentation_authority.overlaid_stage_ids,
    };
    validate_benchmark_campaign_report_publication(&publication)
        .context("validate benchmark campaign report publication")?;
    Ok(publication)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    #[test]
    fn specialized_report_adapters_cover_reviewed_material_without_narrowing_generic_campaigns(
    ) -> anyhow::Result<()> {
        let (campaigns, _, interfaces) =
            super::super::super::load_checked_in_benchmark_planning_contracts()?;
        let campaign_stages = campaigns
            .campaigns
            .iter()
            .map(|campaign| campaign.stage_id.as_str())
            .collect::<BTreeSet<_>>();
        let interface_stages = interfaces
            .interfaces
            .iter()
            .map(|interface| interface.stage_id.as_str())
            .collect::<BTreeSet<_>>();
        let reviewed_material_stages = campaigns
            .campaigns
            .iter()
            .filter(|campaign| campaign.material_driver.is_some())
            .map(|campaign| campaign.stage_id.as_str())
            .collect::<BTreeSet<_>>();
        let adapter_stages =
            super::REPORT_ADAPTERS.iter().map(|adapter| adapter.stage_id).collect::<BTreeSet<_>>();
        assert_eq!(adapter_stages.len(), super::REPORT_ADAPTERS.len());
        assert_eq!(adapter_stages, reviewed_material_stages);
        assert_eq!(campaign_stages, interface_stages);
        assert_eq!(campaign_stages.len(), 71);
        assert_eq!(campaign_stages.difference(&adapter_stages).count(), 58);
        Ok(())
    }
}
