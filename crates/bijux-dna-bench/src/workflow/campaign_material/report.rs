//! Owner: bijux-dna-bench
//! Plan-driven publication across crate-owned benchmark campaign reports.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::BenchmarkCampaignPlan;

use crate::workflow::{
    publish_bam_coverage_stage_report, publish_bam_duplication_metrics_stage_report,
    publish_bam_endogenous_content_stage_report, publish_bam_genotyping_stage_report,
    publish_bam_mapping_summary_stage_report, publish_fastq_profile_read_lengths_stage_report,
    publish_fastq_profile_reads_stage_report,
};

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
/// Each campaign-specific publisher retains ownership of its scientific and evidence validation;
/// this dispatcher only provides one lifecycle operation and one publication identity shape.
///
/// # Errors
/// Returns an error when the plan mixes run identities, names an unsupported campaign, or the
/// selected campaign report fails source, evidence, science, or no-clobber validation.
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

    macro_rules! publish {
        ($publisher:path) => {{
            let publication =
                $publisher(repository_root, study_root, benchmark_root, plan, output_root)?;
            BenchmarkCampaignReportPublication {
                run_id: run_id.to_string(),
                stage_id: stage_id.to_string(),
                report_path: publication.report_path,
                report_sha256: publication.report_sha256,
                markdown_path: publication.markdown_path,
                markdown_sha256: publication.markdown_sha256,
            }
        }};
    }

    let publication = match run_id {
        "run-2001" => publish!(publish_bam_genotyping_stage_report),
        "run-2002" => publish!(publish_bam_coverage_stage_report),
        "run-2003" => publish!(publish_bam_endogenous_content_stage_report),
        "run-2004" => publish!(publish_bam_mapping_summary_stage_report),
        "run-2011" => publish!(publish_bam_duplication_metrics_stage_report),
        "run-2042" => publish!(publish_fastq_profile_read_lengths_stage_report),
        "run-2043" => publish!(publish_fastq_profile_reads_stage_report),
        _ => bail!("benchmark campaign has no crate-owned report publisher: {run_id}"),
    };
    Ok(publication)
}

#[cfg(test)]
mod tests {
    #[test]
    fn report_dispatch_covers_every_executable_campaign() {
        for run_id in super::super::EXECUTABLE_CAMPAIGN_RUN_IDS {
            assert!(matches!(
                *run_id,
                "run-2001"
                    | "run-2002"
                    | "run-2003"
                    | "run-2004"
                    | "run-2011"
                    | "run-2042"
                    | "run-2043"
            ));
        }
    }
}
