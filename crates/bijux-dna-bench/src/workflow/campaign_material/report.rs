//! Owner: bijux-dna-bench
//! Plan-driven publication across crate-owned benchmark campaign reports.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkCampaignPlan;

use crate::workflow::{
    publish_bam_coverage_stage_report, publish_bam_duplication_metrics_stage_report,
    publish_bam_endogenous_content_stage_report, publish_bam_genotyping_stage_report,
    publish_bam_mapping_summary_stage_report, publish_fastq_detect_adapters_stage_report,
    publish_fastq_profile_read_lengths_stage_report, publish_fastq_profile_reads_stage_report,
    publish_fastq_validate_reads_stage_report, BenchmarkStageReportPublication,
};

type CampaignReportPublisher = fn(
    &Path,
    &Path,
    &Path,
    &BenchmarkCampaignPlan,
    &Path,
) -> Result<BenchmarkStageReportPublication>;

struct BenchmarkCampaignReportAdapter {
    stage_id: &'static str,
    publish: CampaignReportPublisher,
}

const REPORT_ADAPTERS: &[BenchmarkCampaignReportAdapter] = &[
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.genotyping",
        publish: publish_bam_genotyping_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.coverage",
        publish: publish_bam_coverage_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.endogenous_content",
        publish: publish_bam_endogenous_content_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.mapping_summary",
        publish: publish_bam_mapping_summary_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "bam.duplication_metrics",
        publish: publish_bam_duplication_metrics_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.detect_adapters",
        publish: publish_fastq_detect_adapters_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.profile_read_lengths",
        publish: publish_fastq_profile_read_lengths_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.profile_reads",
        publish: publish_fastq_profile_reads_stage_report,
    },
    BenchmarkCampaignReportAdapter {
        stage_id: "fastq.validate_reads",
        publish: publish_fastq_validate_reads_stage_report,
    },
];

/// Immutable publication identity shared by every governed campaign report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCampaignReportPublication {
    pub run_id: String,
    pub stage_id: String,
    pub report_path: PathBuf,
    pub report_sha256: String,
    pub markdown_path: PathBuf,
    pub markdown_sha256: String,
}

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
    let publication =
        (adapter.publish)(repository_root, study_root, benchmark_root, plan, output_root)?;
    Ok(BenchmarkCampaignReportPublication {
        run_id: run_id.to_string(),
        stage_id: stage_id.to_string(),
        report_path: publication.report_path,
        report_sha256: publication.report_sha256,
        markdown_path: publication.markdown_path,
        markdown_sha256: publication.markdown_sha256,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    #[test]
    fn report_adapters_exactly_cover_campaign_and_interface_stages() -> anyhow::Result<()> {
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
        let adapter_stages =
            super::REPORT_ADAPTERS.iter().map(|adapter| adapter.stage_id).collect::<BTreeSet<_>>();
        assert_eq!(adapter_stages.len(), super::REPORT_ADAPTERS.len());
        assert_eq!(adapter_stages, campaign_stages);
        assert_eq!(adapter_stages, interface_stages);
        Ok(())
    }
}
